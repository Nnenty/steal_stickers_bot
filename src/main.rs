use middlewares::ClientApplication;
use serde::Deserialize;
use telegram_application::{authorize, client_connect};
use telers::{
    client::Reqwest,
    enums::{ChatType as ChatTypeEnum, ContentType as ContentTypeEnum},
    errors::HandlerError,
    event::ToServiceProvider as _,
    filters::{ChatType, Command, ContentType, State as StateFilter},
    fsm::{MemoryStorage, Strategy},
    methods::SetMyCommands,
    middlewares::outer::FSMContext,
    types::{BotCommand, BotCommandScopeAllPrivateChats},
    Bot, Dispatcher, Filter as _, Router,
};

use tracing::debug;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

pub mod core;
pub mod middlewares;
pub mod states;
mod telegram_application;
pub use states::{AddStickerState, StealStickerSetState};
mod handlers;
use handlers::{
    add_stickers::get_stickers_to_add, add_stickers_handler,
    add_stickers_to_user_owned_sticker_set, cancel_handler, create_new_sticker_set,
    get_stolen_sticker_set, process_wrong_sticker as process_wrong_sticker_handler, source_handler,
    start_handler, steal_sticker_set_handler, steal_sticker_set_name_handler,
};

async fn set_commands(bot: Bot) -> Result<(), HandlerError> {
    let help = BotCommand::new("help", "Show help message");
    let source = BotCommand::new("source", "Show the source of the bot");
    let src = BotCommand::new("src", "Show the source of the bot");
    let steal = BotCommand::new("steal_pack", "Steal sticker pack");
    let steal_sticker = BotCommand::new(
        "add_stickers",
        "Add sticker to a sticker pack stolen by this bot",
    );
    let cancel = BotCommand::new("cancel", "Cancel last command");

    let private_chats = [help, source, src, steal, steal_sticker, cancel];

    bot.send(SetMyCommands::new(private_chats.clone()).scope(BotCommandScopeAllPrivateChats {}))
        .await?;

    Ok(())
}

#[derive(Deserialize)]
pub struct ConfigToml {
    pub application: Application,
    pub auth: Auth,
}
#[derive(Deserialize)]
pub struct Application {
    pub api_id: i32,
    pub api_hash: String,
}
#[derive(Deserialize)]
pub struct Auth {
    pub phone_number: String,
    pub password: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::new("debug")
                .add_directive("hyper=warn".parse().expect("Invalid directive"))
                .add_directive("reqwest=warn".parse().expect("Invalid directive")),
        )
        .init();

    let config = std::fs::read_to_string("config.toml").expect("wrong path");
    let ConfigToml { application, auth } = toml::from_str(&config).unwrap();

    let client = client_connect(application.api_id, application.api_hash)
        .await
        .expect("error connect to Telegram");

    authorize(&client, auth.phone_number.as_str(), auth.password.as_str())
        .await
        .expect("error to authorize");

    let bot = Bot::from_env_by_key("BOT_TOKEN");

    let mut main_router: Router<Reqwest> = Router::new("main");

    // only private because in channels may be many errors
    let mut router = Router::new("private");

    let storage = MemoryStorage::new();
    router
        .update
        .outer_middlewares
        .register(FSMContext::new(storage).strategy(Strategy::UserInChat));
    router
        .message
        .outer_middlewares
        .register(ClientApplication::new(client));

    // if user dont specify one of thats commands, send him help message
    void_command(
        &mut router,
        &[
            "source",
            "src",
            "steal_pack",
            "add_stickers",
            "help",
            "cancel",
        ],
    )
    .await;

    start_command(&mut router, &["start", "help"]).await;

    source_command(&mut router, &["src", "source"]).await;

    cancel_command(&mut router, &["cancel"]).await;

    add_stickers_command(&mut router, "add_stickers", "done").await;

    steal_sticker_set_command(&mut router, "steal_pack").await;

    process_wrong_sticker(&mut router).await;

    main_router.include(router);

    main_router.startup.register(set_commands, (bot.clone(),));

    let dispatcher = Dispatcher::builder()
        .bot(bot)
        .allowed_updates(main_router.resolve_used_update_types())
        .router(main_router)
        .build();

    match dispatcher
        .to_service_provider_default()
        .unwrap()
        .run_polling()
        .await
    {
        Ok(()) => debug!("Bot stopped"),
        Err(err) => debug!("Bot stopped with error: {err}"),
    }
}

/// If the user simply writes to the bot without calling any commands, the bot will send a /help message
async fn void_command(router: &mut Router<Reqwest>, not_void_commands: &'static [&str]) {
    router
        .message
        .register(start_handler::<MemoryStorage>)
        .filter(StateFilter::none())
        .filter(Command::many(not_void_commands.iter().map(ToOwned::to_owned)).invert());
}

/// Executes Telegram commands `/start` and `/help`
async fn start_command(router: &mut Router<Reqwest>, commands: &'static [&str]) {
    router
        .message
        .register(start_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(commands.iter().map(ToOwned::to_owned)));
}

/// Executes Telegram commands `/src` and `/source`
async fn source_command(router: &mut Router<Reqwest>, commands: &'static [&str]) {
    router
        .message
        .register(source_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(commands.iter().map(ToOwned::to_owned)));
}

/// Executes Telegram command `/cancel`
async fn cancel_command(router: &mut Router<Reqwest>, commands: &'static [&str]) {
    router
        .message
        .register(cancel_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(commands.iter().map(ToOwned::to_owned)));
}

/// Executes Telegram command `/add_stickers`
async fn add_stickers_command(
    router: &mut Router<Reqwest>,
    command: &'static str,
    done_command: &'static str,
) {
    router
        .message
        .register(add_stickers_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::one(command))
        .filter(ContentType::one(ContentTypeEnum::Text));

    router
        .message
        .register(get_stolen_sticker_set::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(AddStickerState::GetStolenStickerSet));

    router
        .message
        .register(get_stickers_to_add::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(AddStickerState::GetStickersToAdd));

    router
        .message
        .register(add_stickers_to_user_owned_sticker_set::<MemoryStorage>)
        .filter(Command::one(done_command))
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::one(AddStickerState::GetStickersToAdd));
}

/// Executes Telegram command `/steal_pack`
async fn steal_sticker_set_command(router: &mut Router<Reqwest>, command: &'static str) {
    router
        .message
        .register(steal_sticker_set_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::one(command))
        .filter(ContentType::one(ContentTypeEnum::Text));

    router
        .message
        .register(steal_sticker_set_name_handler::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(StealStickerSetState::StealStickerSetName));

    router
        .message
        .register(create_new_sticker_set::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::one(StealStickerSetState::CreateNewStickerSet));
}

/// If user enter wrong content type, but the request type is Sticker, this handler will process it
async fn process_wrong_sticker(router: &mut Router<Reqwest>) {
    router
        .message
        .register(process_wrong_sticker_handler)
        .filter(ContentType::one(ContentTypeEnum::Sticker).invert())
        .filter(
            StateFilter::one(StealStickerSetState::StealStickerSetName).or(StateFilter::many([
                AddStickerState::GetStolenStickerSet,
                AddStickerState::GetStickersToAdd,
            ])),
        );
}

use telers::{
    enums::ContentType as ContentTypeEnum,
    errors::HandlerError,
    event::ToServiceProvider as _,
    filters::{Command, ContentType, State as StateFilter},
    fsm::{Context, MemoryStorage, Storage, Strategy},
    methods::SetMyCommands,
    middlewares::outer::FSMContext,
    types::{BotCommand, BotCommandScopeAllPrivateChats},
    Bot, Dispatcher, Filter as _, Router,
};
use tracing::debug;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

mod handlers;
use handlers::{start_handler, steal_handler, sticker_handler, wrong_content_type};
pub mod states;
use states::State;

async fn set_commands(bot: Bot) -> Result<(), HandlerError> {
    let help = BotCommand::new("help", "Show help message");
    let steal = BotCommand::new("steal", "Steal sticker pack");
    let my_stickers = BotCommand::new("my_stickers", "Your current sticker packs you was steal");

    let private_chats = [help, steal, my_stickers];

    bot.send(SetMyCommands::new(private_chats.clone()).scope(BotCommandScopeAllPrivateChats {}))
        .await?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::new("debug"))
        .init();

    let bot = Bot::from_env_by_key("BOT_TOKEN");

    let mut main_router = Router::new("main");

    let mut router = Router::new("router");

    router
        .message
        .register(start_handler)
        .filter(Command::many(["start", "help"]));

    let storage = MemoryStorage::new();
    router
        .update
        .outer_middlewares
        .register(FSMContext::new(storage).strategy(Strategy::UserInChat));

    router
        .message
        .register(sticker_handler::<MemoryStorage>)
        .filter(Command::one("steal"))
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::none());

    router
        .message
        .register(steal_handler::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(State::Sticker));

    router
        .message
        .register(wrong_content_type)
        .filter(ContentType::one(ContentTypeEnum::Sticker).invert());

    main_router.include(router);

    // DDDDONNNNNNNNT FOOOOOORRRGEEEEETTT DEEELLL LIIIINEEEE BELLOOWW!!!! this line is so i can make imaginary code blocks
    // ----------------------------------------------------------------------------------

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

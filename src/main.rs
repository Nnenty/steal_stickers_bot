use telers::{
    enums::ContentType as ContentTypeEnum,
    errors::HandlerError,
    event::ToServiceProvider as _,
    filters::{Command, ContentType, State as StateFilter},
    fsm::{MemoryStorage, Strategy},
    methods::SetMyCommands,
    middlewares::outer::FSMContext,
    types::{BotCommand, BotCommandScopeAllPrivateChats},
    Bot, Dispatcher, Filter as _, Router,
};
use tracing::debug;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

mod handlers;
use handlers::{
    get_new_sticker_set_name, get_new_sticker_set_title, process_wrong_sticker, start_handler,
    steal_handler, steal_sticker_set_handler,
};
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

    let storage = MemoryStorage::new();
    router
        .update
        .outer_middlewares
        .register(FSMContext::new(storage).strategy(Strategy::UserInChat));

    // router to execute commands `/start` and `/help`
    router
        .message
        .register(start_handler)
        .filter(Command::many(["start", "help"]))
        .filter(StateFilter::none());

    // router to execute command `/steal`
    router
        .message
        .register(steal_handler::<MemoryStorage>)
        .filter(Command::one("steal"))
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::none());

    // router to get sticker pack that user wants to steal
    router
        .message
        .register(steal_sticker_set_handler::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(State::StealStickerSetName));

    // router that processed wrong content type (required type is sticker)
    router
        .message
        .register(process_wrong_sticker)
        .filter(ContentType::one(ContentTypeEnum::Sticker).invert())
        .filter(StateFilter::one(State::StealStickerSetName));

    // router to get name for new sticker set
    router
        .message
        .register(get_new_sticker_set_name::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::one(State::NewStickerSetName));

    router
        .message
        .register(get_new_sticker_set_title::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::one(State::NewStickerSetTitle));

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

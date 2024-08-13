use telers::{
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

// export modules
pub mod core;
pub mod states;
// export states
pub use states::{AddStickerState, StealStickerSetState};

mod handlers;
use handlers::{
    add_sticker_to_user_owned_sticker_set, cancel_handler, create_new_sticker_set,
    get_stolen_sticker_set, process_wrong_sticker, source_handler, start_handler,
    steal_sticker_handler, steal_sticker_set_handler, steal_sticker_set_name_handler,
};

async fn set_commands(bot: Bot) -> Result<(), HandlerError> {
    let help = BotCommand::new("help", "Show help message");
    let source = BotCommand::new("source", "Show the source of the bot");
    let src = BotCommand::new("src", "Show the source of the bot");
    let steal = BotCommand::new("steal_pack", "Steal sticker pack");
    let steal_sticker = BotCommand::new(
        "add_sticker",
        "Add sticker to a sticker pack stolen by this bot",
    );
    let cancel = BotCommand::new("cancel", "Cancel last command");

    let private_chats = [help, source, src, steal, steal_sticker, cancel];

    bot.send(SetMyCommands::new(private_chats.clone()).scope(BotCommandScopeAllPrivateChats {}))
        .await?;

    Ok(())
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

    let bot = Bot::from_env_by_key("BOT_TOKEN");

    let mut main_router = Router::new("main");

    let mut router = Router::new("private");

    let storage = MemoryStorage::new();
    router
        .update
        .outer_middlewares
        .register(FSMContext::new(storage).strategy(Strategy::UserInChat));

    // if user just send messages without request, send him help message
    router
        .message
        .register(start_handler::<MemoryStorage>)
        .filter(StateFilter::none())
        .filter(Command::many(["steal_pack", "add_sticker", "source", "src", "cancel"]).invert());

    // router to execute commands `/start` and `/help`
    router
        .message
        .register(start_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(["start", "help"]));

    // router to execute commands `/src` and `/source`
    router
        .message
        .register(source_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(["src", "source"]));

    // router to cancel last command
    router
        .message
        .register(cancel_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(["cancel"]));

    //                                COMMAND    `/steal_sticker`
    // ------------------------------------------------------------------------------------------------------

    // router to execute command `/steal_sticker`
    router
        .message
        .register(steal_sticker_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::one("add_sticker"))
        .filter(ContentType::one(ContentTypeEnum::Text));

    // router to get sticker set
    router
        .message
        .register(get_stolen_sticker_set::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(AddStickerState::GetStolenStickerSet));

    router
        .message
        .register(add_sticker_to_user_owned_sticker_set::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(
            AddStickerState::AddStickerToStolenStickerSet,
        ));

    // ------------------------------------------------------------------------------------------------------

    //                                  COMMAND    `/steal_pack`
    // ------------------------------------------------------------------------------------------------------

    // router to execute command `/steal_pack`
    router
        .message
        .register(steal_sticker_set_handler::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::one("steal_pack"))
        .filter(ContentType::one(ContentTypeEnum::Text));

    // router to get sticker pack that user wants to steal
    router
        .message
        .register(steal_sticker_set_name_handler::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(StealStickerSetState::StealStickerSetName));

    // router to create new sticker set
    router
        .message
        .register(create_new_sticker_set::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::one(StealStickerSetState::CreateNewStickerSet));

    // ------------------------------------------------------------------------------------------------------

    // router that processed wrong content type (required type is sticker)
    router
        .message
        .register(process_wrong_sticker)
        .filter(ContentType::one(ContentTypeEnum::Sticker).invert())
        .filter(
            StateFilter::many([StealStickerSetState::StealStickerSetName]).or(StateFilter::many([
                AddStickerState::GetStolenStickerSet,
                AddStickerState::AddStickerToStolenStickerSet,
            ])),
        );

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

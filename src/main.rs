use telers::enums::UpdateType;
use telers::errors::HandlerError;
use telers::event::ToServiceProvider;
use telers::filters::Command;
use telers::methods::SetMyCommands;
use telers::types::{BotCommand, BotCommandScopeAllPrivateChats};
use telers::Bot;
use telers::Dispatcher;
use telers::Router;

use tracing::debug;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

mod handlers;
use handlers::start_handler;

async fn set_commands(bot: Bot) -> Result<(), HandlerError> {
    let help = BotCommand::new("help", "Show help message");

    let private_chats = [help];

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

    main_router
        .message
        .register(start_handler)
        .filter(Command::many(["start", "help"]));

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

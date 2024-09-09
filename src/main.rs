use std::process;

use telers::{
    client::Reqwest,
    enums::ContentType as ContentTypeEnum,
    errors::HandlerError,
    event::ToServiceProvider as _,
    fsm::{MemoryStorage, Strategy},
    methods::SetMyCommands,
    middlewares::outer::FSMContext,
    types::{BotCommand, BotCommandScopeAllPrivateChats},
    Bot, Dispatcher, Router,
};

use tracing::{debug, error};
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

use clap::{Parser, Subcommand};
use serde::Deserialize;
use toml;

pub mod bot_commands;
pub mod core;
pub mod middlewares;
pub mod states;
mod telegram_application;
use bot_commands::{
    add_stickers_command, cancel_command, my_stickers, process_non_command, process_non_sticker,
    source_command, start_command, steal_sticker_set_command,
};
pub use core::{common, texts};
use middlewares::ClientApplication;
use telegram_application::{client_authorize, client_connect};

async fn set_commands(bot: Bot) -> Result<(), HandlerError> {
    let help = BotCommand::new("help", "Show help message");
    let source = BotCommand::new("source", "Show the source of the bot");
    let src = BotCommand::new("src", "Show the source of the bot");
    let steal = BotCommand::new("steal_pack", "Steal sticker pack");
    let steal_sticker = BotCommand::new(
        "add_stickers",
        "Add stickers to a sticker pack stolen by this bot",
    );
    let my_stickers = BotCommand::new("my_stickers", "List of your stolen stickers");
    let cancel = BotCommand::new("cancel", "Cancel last command");

    let private_chats = [help, source, src, steal, steal_sticker, cancel, my_stickers];

    bot.send(SetMyCommands::new(private_chats).scope(BotCommandScopeAllPrivateChats {}))
        .await?;

    Ok(())
}

#[derive(Deserialize)]
struct ConfigToml {
    bot: BotConfig,
    application: Application,
    auth: AuthCredentials,
}
#[derive(Deserialize)]
struct BotConfig {
    bot_token: String,
}
#[derive(Deserialize)]
struct Application {
    api_id: i32,
    api_hash: String,
}
#[derive(Deserialize)]
struct AuthCredentials {
    phone_number: String,
    password: String,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, PartialEq)]
enum Commands {
    /// Authorize client and exit
    Auth,
    /// Run programm (exit if client not authorized)
    Run,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::new("debug")
                .add_directive("hyper=warn".parse().expect("Invalid directive"))
                .add_directive("reqwest=warn".parse().expect("Invalid directive"))
                .add_directive("grammers=warn".parse().expect("Invalid directive")),
        )
        .init();

    let config = std::fs::read_to_string("configs/config.toml").expect("wrong path");
    let ConfigToml {
        application: Application { api_id, api_hash },
        auth: AuthCredentials {
            phone_number,
            password,
        },
        bot: BotConfig { bot_token },
    } = toml::from_str(&config).unwrap();

    let cli = Cli::parse();

    let client = client_connect(api_id, api_hash.clone())
        .await
        .expect("error connect to Telegram");

    if Commands::Auth == cli.command {
        client_authorize(&client, phone_number.as_str(), password.as_str())
            .await
            .expect("error to authorize");

        debug!("Client sucessfully authorized! Now run programm using command:\ndocker compose up --build");

        process::exit(0);
    }
    if Commands::Run == cli.command && !client.is_authorized().await.expect("error to authorize") {
        error!("Client is not authorized! Run programm with command auth:\njust auth");

        process::exit(1);
    }

    let bot = Bot::new(bot_token);

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
        .register(ClientApplication::new(client, api_id, api_hash));

    // all bot commands
    // -------------------------------------------------------------------------------

    process_non_command(
        &mut router,
        &[
            "source",
            "src",
            "steal_pack",
            "add_stickers",
            "help",
            "cancel",
            "my_stickers",
        ],
    )
    .await;
    start_command(&mut router, &["start", "help"]).await;
    source_command(&mut router, &["src", "source"]).await;
    cancel_command(&mut router, &["cancel"]).await;
    add_stickers_command(&mut router, "add_stickers", "done").await;
    steal_sticker_set_command(&mut router, "steal_pack").await;
    process_non_sticker(&mut router, ContentTypeEnum::Sticker).await;
    my_stickers(&mut router, "my_stickers").await;

    // -------------------------------------------------------------------------------

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

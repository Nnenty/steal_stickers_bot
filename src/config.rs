use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfigToml {
    pub bot: BotConfig,
    pub tg_app: Application,
    pub auth: AuthCredentials,
    pub tracing: Tracing,
    pub database: DatabaseConfig,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub db_url: String,
}

#[derive(Deserialize)]
pub struct BotConfig {
    pub bot_token: String,
}

#[derive(Deserialize)]
pub struct Application {
    pub api_id: i32,
    pub api_hash: String,
}

#[derive(Deserialize)]
pub struct AuthCredentials {
    pub phone_number: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Tracing {
    pub log_level: String,
}

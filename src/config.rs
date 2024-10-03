use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ConfigToml {
    pub bot: BotConfig,
    pub tg_app: Application,
    pub auth: AuthCredentials,
    pub tracing: Tracing,
    pub postgres: DatabaseConfig,
}

impl ConfigToml {
    pub fn get_postgres_url(&self) -> String {
        let postgres = &self.postgres;

        format!(
            "postgres://{}:{}@{}:{}/{}",
            postgres.username, postgres.password, postgres.host, postgres.port, postgres.db
        )
    }
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub db: String,
}

#[derive(Deserialize, Clone)]
pub struct BotConfig {
    pub bot_token: String,
}

#[derive(Deserialize, Clone)]
pub struct Application {
    pub api_id: i32,
    pub api_hash: String,
}

#[derive(Deserialize, Clone)]
pub struct AuthCredentials {
    pub phone_number: String,
    pub password: String,
}

#[derive(Deserialize, Clone)]
pub struct Tracing {
    pub log_level: String,
}

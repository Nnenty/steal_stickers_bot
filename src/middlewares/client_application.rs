use async_trait::async_trait;
use chrono::{NaiveTime, Utc};
use grammers_client::Client as ClientGrammers;
use telers::{
    errors::EventErrorKind,
    event::EventReturn,
    middlewares::{outer::MiddlewareResponse, OuterMiddleware},
    router::Request,
    FromContext,
};
use tokio::sync::Mutex;
use tracing::debug;

use crate::telegram_application::client_connect;

#[derive(Debug, Clone, FromContext)]
#[context(key = "client", from = ClientGrammers)]
pub struct Client(pub ClientGrammers);

impl From<ClientGrammers> for Client {
    fn from(value: ClientGrammers) -> Self {
        Self(value)
    }
}

pub struct ClientApplication {
    pub key: &'static str,
    pub client: Mutex<ClientGrammers>,
    pub last_update_time: Mutex<NaiveTime>,
    pub api_id: i32,
    pub api_hash: String,
}

impl ClientApplication {
    pub fn new(client: ClientGrammers, api_id: i32, api_hash: String) -> Self {
        Self {
            key: "client",
            client: Mutex::new(client),
            last_update_time: Mutex::new(Utc::now().time()),
            api_id,
            api_hash,
        }
    }
}

#[async_trait]
impl OuterMiddleware for ClientApplication {
    async fn call(&self, request: Request) -> Result<MiddlewareResponse, EventErrorKind> {
        let now = Utc::now().time();

        let mut lock = self.last_update_time.lock().await;

        if (now - *lock).num_minutes() >= 10 {
            debug!("Update client");

            *lock = now;

            let client = client_connect(self.api_id, self.api_hash.clone())
                .await
                .unwrap();

            let mut lock = self.client.lock().await;
            *lock = client.clone();

            request.context.insert(self.key, Box::new(client));
        } else {
            let client = (*self.client.lock().await).clone();

            request.context.insert(self.key, Box::new(client));
        }

        Ok((request, EventReturn::default()))
    }
}

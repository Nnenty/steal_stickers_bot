use async_trait::async_trait;
use grammers_client::Client as ClientGrammers;
use telers::{
    errors::EventErrorKind,
    event::EventReturn,
    middlewares::{outer::MiddlewareResponse, OuterMiddleware},
    router::Request,
    FromContext,
};

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
    pub data: ClientGrammers,
}

impl ClientApplication {
    pub const fn new(client: ClientGrammers) -> ClientApplication {
        Self {
            key: "client",
            data: client,
        }
    }
}

#[async_trait]
impl OuterMiddleware for ClientApplication {
    async fn call(&self, request: Request) -> Result<MiddlewareResponse, EventErrorKind> {
        request
            .context
            .insert(self.key, Box::new(self.data.clone()));

        Ok((request, EventReturn::default()))
    }
}

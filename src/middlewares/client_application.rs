use async_trait::async_trait;
use grammers_client::Client as ClientGrammers;
use telers::{event::EventReturn, middlewares::OuterMiddleware, FromContext};

#[derive(Debug, Clone, FromContext)]
#[context(key = "client")]
pub struct Client(pub ClientGrammers);

pub struct ClientApplication {
    pub key: &'static str,
    pub data: Client,
}

impl Client {
    fn new(client: ClientGrammers) -> Client {
        Self { 0: client }
    }
}

impl ClientApplication {
    pub fn new(client: ClientGrammers) -> ClientApplication {
        Self {
            key: "client",
            data: Client::new(client),
        }
    }
}

#[async_trait]
impl OuterMiddleware for ClientApplication {
    async fn call(
        &self,
        request: telers::router::Request<telers::client::Reqwest>,
    ) -> Result<
        telers::middlewares::outer::MiddlewareResponse<telers::client::Reqwest>,
        telers::errors::EventErrorKind,
    > {
        request
            .context
            .insert(self.key, Box::new(self.data.clone()));

        Ok((request, EventReturn::default()))
    }
}

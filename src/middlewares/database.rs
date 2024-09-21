use async_trait::async_trait;
use telers::{
    errors::EventErrorKind,
    event::EventReturn,
    middlewares::{outer::MiddlewareResponse, OuterMiddleware},
    router::Request,
};

use crate::application::common::traits::uow::UoWFactory;

#[derive(Debug)]
pub struct Database<UoWF> {
    uow_factory: UoWF,
}

impl<UoWF> Database<UoWF> {
    pub fn new(uow_factory: UoWF) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl<UoWF> OuterMiddleware for Database<UoWF>
where
    UoWF: Send + Sync + UoWFactory + Clone + 'static,
{
    async fn call(&self, request: Request) -> Result<MiddlewareResponse, EventErrorKind> {
        request
            .context
            .insert("uow_factory", Box::new(self.uow_factory.clone()));

        Ok((request, EventReturn::default()))
    }
}

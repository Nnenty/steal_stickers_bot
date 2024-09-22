use async_trait::async_trait;
use telers::{
    errors::EventErrorKind,
    event::EventReturn,
    middlewares::{outer::MiddlewareResponse, OuterMiddleware},
    router::Request,
};

use crate::application::common::traits::uow::UoWFactory as UoWFactoryTrait;

#[derive(Debug)]
pub struct Database<UoWFactory> {
    uow_factory: UoWFactory,
}

impl<UoWFactory> Database<UoWFactory> {
    pub const fn new(uow_factory: UoWFactory) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl<UoWFactory> OuterMiddleware for Database<UoWFactory>
where
    UoWFactory: Send + Sync + UoWFactoryTrait + Clone + 'static,
{
    async fn call(&self, request: Request) -> Result<MiddlewareResponse, EventErrorKind> {
        println!("{:?}", request.context.get("uow_factory"));

        request
            .context
            .insert("uow_factory", Box::new(self.uow_factory.clone()));

        println!("{:?}", request.context.get("uow_factory"));

        Ok((request, EventReturn::default()))
    }
}

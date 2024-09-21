// use async_trait::async_trait;
// use sqlx::{PgConnection, PgPool, Pool, Postgres};
// use telers::{
//     errors::EventErrorKind,
//     event::EventReturn,
//     middlewares::{outer::MiddlewareResponse, OuterMiddleware},
//     router::Request,
// };

// #[derive(Debug)]
// pub struct Database<Pool> {
//     pool: Pool,
// }

// impl<PgPool> Database<PgPool> {
//     pub fn new(pool: PgPool) -> Self {
//         Self { pool }
//     }
// }

// // #[async_trait]
// // impl OuterMiddleware for Database<UoW<&mut PgConnection>> {
// //     async fn call(&self, request: Request) -> Result<MiddlewareResponse, EventErrorKind> {
// //         let pool = Pool::<Postgres>::connect("postgres://admin:admin@localhost/test_db")
// //             .await
// //             .unwrap();

// //         // request.context.insert("uow", Box::new(self.pool.clone()));

// //         Ok((request, EventReturn::default()))
// //     }
// // }

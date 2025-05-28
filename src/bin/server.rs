use actix_cors::Cors;
use actix_web::{App, HttpServer, http::header, web};
use dotenv::dotenv;
use log::{LevelFilter, error, info, warn};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use sqlx::Executor;
use sqlx_postgres::PgPool;
use std::env;
use std::error::Error;
use std::str::FromStr;
use tonic::{Request, Response, Status, transport::Server};

use mail_test::mail_proxy_server::{MailProxy, MailProxyServer};
use mail_test::{MailRequest, MailResponse};

mod handlers;

use handlers::mail;

pub mod mail_test {
    tonic::include_proto!("mail");
}

// #[derive(Debug)]
// pub struct MyMailProxy {
//     connection: PgPool,
// }
//
// impl MyMailProxy {
//     pub fn new(connection: PgPool) -> Self {
//         Self { connection }
//     }
// }

// #[tonic::async_trait]
// impl MailProxy for MyMailProxy {
//     async fn send_mail(
//         &self,
//         request: Request<MailRequest>,
//     ) -> Result<Response<MailResponse>, Status> {
//         println!("Got a request: {:?}", request);
//         let pool = &self.connection;
//
//         let query = sqlx::query("INSERT INTO mail ( from, to ) VALUES ( ?, ? )")
//             .bind("from")
//             .bind("to");
//
//         let result = pool.execute(query).await.unwrap();
//
//         let response = if result.rows_affected() > 1 {
//             MailResponse { is_success: true }
//         } else {
//             MailResponse { is_success: false }
//         };
//
//         Ok(Response::new(response))
//     }
// }

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    // let addr = "[::1]:50051".parse().unwrap();
    let db_url = env::var("DB_URL").expect("Cant find DB_URL in .env");

    let pool = PgPool::connect(&db_url).await.unwrap();
    // let greeter = MyMailProxy::new(pool.clone());

    // let mailboxes_type = sqlx::query(include_str!("../sql/create_mailbox_status_type.sql"));
    let mailboxes_table = sqlx::query(include_str!("../sql/create_mailboxes_table.sql"));
    let messages_table = sqlx::query(include_str!("../sql/create_messages_table.sql"));
    let domains_table = sqlx::query(include_str!("../sql/create_domains_table.sql"));

    // pool.execute(mailboxes_type).await.unwrap();
    pool.execute(mailboxes_table).await.unwrap();
    pool.execute(messages_table).await.unwrap();
    pool.execute(domains_table).await.unwrap();

    let app_state = web::Data::new(pool);

    // tokio::spawn(async move {
    //     Server::builder()
    //         .add_service(MailProxyServer::new(greeter))
    //         .serve(addr)
    //         .await
    //         .unwrap();
    // });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET"])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .app_data(app_state.clone())
            .service(web::scope("/api").configure(mail::mail::mail_config))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

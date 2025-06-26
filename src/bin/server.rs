use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::{App, HttpMessage, HttpServer, cookie::Key, dev::Service as _, http::header, web};
use dotenv::dotenv;
use futures_util::future::FutureExt;
use log::LevelFilter;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use sqlx::Executor;
use sqlx_postgres::PgPool;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot::Sender;
use uuid::Uuid;

type MailId = Uuid;

type ChannelsMap = Arc<Mutex<HashMap<MailId, Sender<Mail>>>>;

mod handlers;

use handlers::mail::mail_handler;
use mail::models::Mail;

struct State {
    pool: PgPool,
    channels_map: ChannelsMap,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let db_url = env::var("DB_URL").expect("Cant find DB_URL in .env");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Create Redis session store
    let redis_store = RedisSessionStore::new(redis_url)
        .await
        .expect("Failed to create Redis session store");

    let pool = PgPool::connect(&db_url).await.unwrap();

    // let mailboxes_type = sqlx::query(include_str!("../sql/create_mailbox_status_type.sql"));
    let mailboxes_table = sqlx::query(include_str!("../sql/create_mailboxes_table.sql"));
    let messages_table = sqlx::query(include_str!("../sql/create_messages_table.sql"));
    let domains_table = sqlx::query(include_str!("../sql/create_domains_table.sql"));

    // pool.execute(mailboxes_type).await.unwrap();
    pool.execute(mailboxes_table).await.unwrap();
    pool.execute(messages_table).await.unwrap();
    pool.execute(domains_table).await.unwrap();

    // Secret key for session encryption
    let secret_key = Key::generate();

    let channels_map = Arc::new(Mutex::new(HashMap::new()));
    let state = State { pool, channels_map };

    let app_state = web::Data::new(state);

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .supports_credentials()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                    .cookie_secure(false) // Set to true in production
                    .build(),
            )
            .app_data(app_state.clone())
            .service(web::scope("/api").configure(mail_handler::mail_config))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

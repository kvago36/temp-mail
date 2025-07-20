use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::{App, HttpMessage, HttpServer, cookie::Key, dev::Service as _, http::header, web};
use dotenv::dotenv;
use futures_util::future::FutureExt;
use futures_util::stream::{self, StreamExt};
use log::{LevelFilter, info};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use sqlx::{Executor, QueryBuilder, Row};
// use sqlx::Executor;
use sqlx_postgres::PgPool;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::fs::read_to_string;
use tokio::sync::oneshot::Sender;
use uuid::Uuid;

type MailId = Uuid;

type ChannelsMap = Arc<Mutex<HashMap<MailId, Sender<Mail>>>>;

mod handlers;
mod middlewares;

use handlers::email::email_handler;
use handlers::mail::mail_handler;
use handlers::store::store_handler;

use mail::models::Mail;

#[derive(Debug)]
struct DnsEntry {
    domain: String,
    ip: Option<String>,
    mx_host: Option<String>,
    mx_priority: Option<u32>,
}

#[derive(Debug)]
struct Domains {
    domains: Vec<DnsEntry>,
}

impl IntoIterator for Domains {
    type Item = DnsEntry;
    type IntoIter = std::vec::IntoIter<DnsEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.domains.into_iter()
    }
}
impl Domains {
    async fn from_file() -> Self {
        let content = read_to_string("dnsmasq.conf").await.unwrap();

        let mut addresses: HashMap<String, String> = HashMap::new();
        let mut mx_records: Vec<(String, String, u32)> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("address=") {
                // Формат: address=/domain/ip
                if let Some(rest) = line.strip_prefix("address=") {
                    let parts: Vec<&str> = rest.trim_matches('/').split('/').collect();
                    if parts.len() == 2 {
                        let domain = parts[0].to_string();
                        let ip = parts[1].to_string();
                        addresses.insert(domain, ip);
                    }
                }
            } else if line.starts_with("mx-host=") {
                // Формат: mx-host=domain,mx_host,priority
                if let Some(rest) = line.strip_prefix("mx-host=") {
                    let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                    if parts.len() == 3 {
                        let domain = parts[0].to_string();
                        let mx_host = parts[1].to_string();
                        let priority = parts[2].parse::<u32>().unwrap_or(10);
                        mx_records.push((domain, mx_host, priority));
                    }
                }
            }
        }

        let mut results = Vec::new();
        for (domain, mx_host, priority) in mx_records {
            let ip = addresses.get(&mx_host).cloned();
            results.push(DnsEntry {
                domain,
                ip,
                mx_host: Some(mx_host),
                mx_priority: Some(priority),
            });
        }

        Domains { domains: results }
    }
}

struct State {
    pool: PgPool,
    channels_map: ChannelsMap,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let domains = Domains::from_file().await;

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
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

    let mut builder = QueryBuilder::new("INSERT INTO domains (name) ");

    builder.push_values(domains, |mut b, entry| {
        b.push_bind(entry.domain);
    });
    builder.push(" ON CONFLICT (name) DO NOTHING");
    builder.build().execute(&pool).await.unwrap();

    info!("Batch insert done");

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
            .service(
                web::scope("/api")
                    .configure(store_handler::store_config)
                    .configure(mail_handler::mail_config)
                    .configure(email_handler::email_config),
            )
    })
    .bind((host, 8000))?
    .run()
    .await
}

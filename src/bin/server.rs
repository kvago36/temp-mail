use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::{App, HttpMessage, HttpServer, cookie::Key, dev::Service as _, http::header, web};
use clap::Parser;
use futures_util::future::FutureExt;
use futures_util::stream::{self, StreamExt};
use log::{LevelFilter, info};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use sqlx::{Executor, QueryBuilder, Row, migrate::Migrator};
use sqlx_postgres::PgPool;
use std::collections::HashMap;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::path::Path;
use sqlx::sqlx_macros::migrate;
use tokio::fs::read_to_string;
use tokio::sync::oneshot::Sender;
use uuid::Uuid;

type MailId = Uuid;

type ChannelsMap = Arc<Mutex<HashMap<MailId, Sender<Mail>>>>;

mod config;
mod handlers;
mod middlewares;

use handlers::email::email_handler;
use handlers::mail::mail_handler;
use handlers::store::store_handler;

use config::config::Args;

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
    let args = Args::parse();

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let domains = Domains::from_file().await;

    let ip_from_str = args.host.parse().unwrap_or(Ipv4Addr::new(127, 0, 0, 1));
    let socket = SocketAddr::new(IpAddr::V4(ip_from_str), args.port);

    let redis_store = RedisSessionStore::new(&args.redis_url)
        .await
        .expect("Failed to create Redis session store");

    let migrator = migrate!("./migrations");

    let pool = PgPool::connect(&args.db_url).await.unwrap();

    migrator.run(&pool).await.unwrap();

    let mut builder = QueryBuilder::new("INSERT INTO domains (name) ");

    builder.push_values(domains, |mut b, entry| {
        b.push_bind(entry.domain);
    });
    builder.push(" ON CONFLICT (name) DO NOTHING");
    builder.build().execute(&pool).await.unwrap();

    // Secret key for session encryption
    let secret_key = Key::generate();
    let is_prod = !cfg!(debug_assertions); // true в release, false в debug

    let channels_map = Arc::new(Mutex::new(HashMap::new()));
    let state = State { pool, channels_map };

    let app_state = web::Data::new(state);

    println!("is_prod: {}, origin: {}", is_prod, &args.frontend_origin);

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin(&args.frontend_origin)
                    .supports_credentials()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                    .cookie_secure(is_prod) // Set to true in production
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
        .bind(socket)?
        .run()
        .await
}
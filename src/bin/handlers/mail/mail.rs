use actix_web::{HttpResponse, Responder, get, web, HttpRequest};
use actix_web::cookie::Cookie;
use rand::prelude::*;
use serde_json::json;
use sqlx::{Executor, Row};
use sqlx_postgres::PgPool;

use chrono::{DateTime, Utc};
use log::{error, info};
use tokio::time::{Duration, sleep};
use tokio::sync::oneshot;
use uuid::{uuid, Uuid};

use mail::models::{Mail, MailboxStatus};
use crate::{ChannelsMap, ClientId, State};

#[get("/{id}")]
async fn get_messages(data: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let pool = data.get_ref();
    let id = path.into_inner();
    let mut messages: Vec<Mail> = vec![];

    let rows = sqlx::query(include_str!("../../../sql/get_messages.sql"))
        .bind(&id)
        .fetch_all(pool)
        .await
        .unwrap();

    for row in &rows {
        let subject: String = row.get("subject");
        let message: String = row.get("body");
        let sender: String = row.get("sender");
        let received_at: DateTime<Utc> = row.get("received_at");

        messages.push(Mail::new(subject, sender, message, received_at.to_string()));
    }

    let json = json!({ "messages": messages });

    HttpResponse::Ok().json(json)
}

#[get("/new")]
async fn await_for_new_mail(data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut channels = data.channels_map.lock().unwrap();
    let cookie = req.cookie("user_id");
    let (tx, rx) = oneshot::channel();

    match cookie {
        None => {
            unreachable!();
        }
        Some(c) => {
            let id = c.value();
            let uuid = Uuid::parse_str(id).unwrap();

            channels.entry(uuid).or_insert_with(|| tx);
        }
    };

    // TODO: CHANGE TIMEOUT AFTER TESTS
    let sleep = sleep(Duration::from_secs(5));

    drop(channels);

    tokio::pin!(sleep);

    tokio::select! {
        _ = &mut sleep => {
            info!("operation timed out");
            HttpResponse::Ok().json(json!({ "status": "ok", "n": null }))
        }
        res = rx => {
            if let Ok(n) = res {
                info!("operation completed {}", n);
                HttpResponse::Ok().json(json!({ "status": "ok", "n": n }))
            } else {
                error!("operation error");
                HttpResponse::Ok().json(json!({ "status": "error", "n": null }))
            }
        }
    }
}

#[get("/")]
async fn get_mail(data: web::Data<PgPool>) -> impl Responder {
    let pool = data.get_ref();
    let mut rng = thread_rng();
    let mut nums: Vec<i32> = (1..100).collect();

    nums.shuffle(&mut rng);

    let random_slug = nums.choose(&mut rng).unwrap();
    let random_email = format!("email{}@test.com", random_slug);

    sleep(Duration::from_millis(100)).await;

    let domains_query = sqlx::query(include_str!("../../../sql/get_domains.sql"))
        .bind(&random_email)
        .fetch_all(pool)
        .await
        .unwrap();

    let mail_query = sqlx::query(include_str!("../../../sql/save_user_email.sql"))
        .bind(&random_email)
        .fetch_one(pool)
        .await
        .unwrap();

    let id: Uuid = mail_query.get("id");
    let mail_status: MailboxStatus = mail_query.get("status");
    let user_json = json!({ "id": id, "email": random_email, "status": mail_status.to_string() });

    // TODO: fix error on status type
    // println!("{:?}", mail_query);

    HttpResponse::Ok().json(user_json)
}

pub fn mail_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/mail").service(get_messages).service(get_mail));
}

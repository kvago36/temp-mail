use actix_session::Session;
use actix_web::{HttpResponse, Responder, get, web};
use chrono::Utc;
use mail::models::{MailboxStatus, UserSession};
use rand::prelude::*;
use serde_json::json;
use sqlx::{Executor, Row};
use tokio::time::{Duration, sleep};
use uuid::Uuid;

use crate::State;

use mail::utils::random::generate_email_local_part;

#[get("/")]
async fn get_mail(session: Session, data: web::Data<State>) -> impl Responder {
    let pool = &data.pool;
    let mut rng = thread_rng();

    let random_slug = generate_email_local_part();

    sleep(Duration::from_millis(100)).await;

    let domains_query = sqlx::query(include_str!("../../../sql/get_domains.sql"))
        .fetch_all(pool)
        .await
        .unwrap();

    let mut domains: Vec<String> = domains_query
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    domains.shuffle(&mut rng);

    let random_domain = domains.choose(&mut rng).unwrap();

    let random_email = format!("{}@{}", random_slug, random_domain);

    let mail_query = sqlx::query(include_str!("../../../sql/save_user_email.sql"))
        .bind(&random_email)
        .fetch_one(pool)
        .await
        .unwrap();

    let mail_id: Uuid = mail_query.get("id");
    let mail_status: MailboxStatus = mail_query.get("status");
    let mail_json =
        json!({ "id": mail_id, "email": random_email, "status": mail_status.to_string() });

    let user_session = UserSession {
        mail_id,
        email: random_email,
        created_ad: Utc::now().timestamp(),
    };

    session
        .insert("user", user_session)
        .expect("Cant serialize userSession");

    // TODO: fix error on status type
    // println!("{:?}", mail_query);

    HttpResponse::Ok().json(mail_json)
}

pub fn email_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/email").service(get_mail));
}

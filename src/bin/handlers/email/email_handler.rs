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

#[get("/")]
async fn get_mail(session: Session, data: web::Data<State>) -> impl Responder {
    let pool = &data.pool;
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

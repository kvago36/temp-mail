use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Executor, Row};
use sqlx_postgres::PgPool;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use tokio::sync::oneshot;
use tokio::time::{Duration, sleep};
use uuid::{Uuid, uuid};

use mail::models::{Mail, MailboxStatus};

use crate::{ChannelsMap, MailId, State};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSession {
    pub mail_id: Uuid,
    pub email: String,
    pub created_ad: i64,
}

#[get("/{id}")]
async fn get_messages(data: web::Data<State>, path: web::Path<Uuid>) -> impl Responder {
    let pool = &data.pool;
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

        messages.push(Mail {
            subject,
            receivers: vec![],
            sender,
            message,
            timestamp: received_at.timestamp(),
            body: "test_body".to_string(),
            attachments: vec![],
            domain: "test".to_string(),
        });
    }

    let json = json!({ "messages": messages });

    HttpResponse::Ok().json(json)
}

#[post("/")]
async fn new_message(
    mail: web::Json<Mail>,
    data: web::Data<State>,
) -> impl Responder {
    let mail = mail.into_inner();
    let pool = &data.pool;

    let receiver = mail
        .receivers
        .first()
        .expect("should be at least one receiver");

    let mailbox_query =
        sqlx::query("SELECT * from mailboxes where email = $1 and status != 'expired'")
            .bind(receiver)
            .fetch_one(pool)
            .await;

    if let Ok(row) = mailbox_query {
        let mailbox_id: Uuid = row.get("id");

        let query = sqlx::query(
            "INSERT INTO messages ( mailbox_id, sender, subject, body ) VALUES ( $1, $2, $3, $4 )",
        )
        .bind(mailbox_id)
        .bind(&mail.sender)
        .bind(&mail.subject)
        .bind(&mail.message);

        let result = pool.execute(query).await.unwrap();

        if result.rows_affected() < 1 {
            error!("Error while saving mail to mailbox: {}", mailbox_id);
        } else {
            let mut channels = data.channels_map.lock().unwrap();

            if let Some(channel) = channels.remove(&mailbox_id) {
                if let Err(_) = channel.send(mail) {
                    error!("The receiver for mail: {} dropped", mailbox_id);
                } else {
                    info!("Message sent to mail {}", mailbox_id);
                }
            }

            info!("Message saved to mailbox: {}", mailbox_id);
        }
    } else {
        warn!("No such email: {}", receiver)
    }

    HttpResponse::Ok()
}

#[get("/new")]
async fn await_for_new_mail(session: Session, data: web::Data<State>) -> impl Responder {
    let mut channels = data.channels_map.lock().unwrap();
    let (tx, rx) = oneshot::channel();

    match session.get::<UserSession>("user") {
        Ok(s) => {
            if let Some(user_session) = s {
                // TODO: CHANGE TIMEOUT AFTER TESTS
                let sleep = sleep(Duration::from_secs(60));
                let mail_id = user_session.mail_id;

                channels.insert(mail_id, tx);

                drop(channels);

                tokio::pin!(sleep);

                tokio::select! {
                    _ = &mut sleep => {
                        HttpResponse::RequestTimeout().json(json!({
                            "status": "ok",
                            "mail": null,
                            "message": "Operation timed out"
                        }))
                    }
                    res = rx => {
                        match res {
                            Ok(mail) => {
                                HttpResponse::Ok().json(json!({
                                    "status": "ok",
                                    "mail": mail,
                                }))
                            },
                            Err(e) => {
                                error!("{}", e);
                                HttpResponse::InternalServerError().json(json!({
                                    "status": "error",
                                    "mail": null,
                                    "message": "Cant receive new message"
                                }))
                            }
                        }
                    }
                }
            } else {
                HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "mail": null,
                    "message": "Get user session failed"
                }))
            }
        }
        Err(_) => HttpResponse::ServiceUnavailable().json(json!({
            "status": "error",
            "mail": null,
            "message": "User session storage isn't responding"
        })),
    }
}

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
    let mail_json = json!({ "id": mail_id, "email": random_email, "status": mail_status.to_string() });

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

pub fn mail_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/mail")
            .service(new_message)
            .service(await_for_new_mail)
            .service(get_messages)
            .service(get_mail),
    );
}

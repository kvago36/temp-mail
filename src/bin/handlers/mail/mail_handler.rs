use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, HttpMessage};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Executor, Row};
use sqlx_postgres::PgPool;
use tokio::sync::oneshot;
use tokio::time::{Duration, sleep};
use uuid::{Uuid, uuid};

use mail::models::{Mail, MailboxStatus, UserSession};

use crate::{ChannelsMap, MailId, State};

use crate::middlewares::my_middleware::{RateLimit};

#[get("/")]
async fn get_messages(
    req: HttpRequest,
    data: web::Data<State>,
) -> impl Responder {
    let pool = &data.pool;
    let mut messages: Vec<Mail> = vec![];
    let extensions = req.extensions();
    let user_session = extensions.get::<UserSession>().expect("Should have session from middleware");

    let rows = sqlx::query(include_str!("../../../sql/get_messages.sql"))
        .bind(user_session.mail_id)
        .fetch_all(pool)
        .await
        .unwrap();

    for row in &rows {
        let id: Uuid = row.get("id");
        let subject: String = row.get("subject");
        let message: String = row.get("body");
        let sender: String = row.get("sender");
        let received_at: DateTime<Utc> = row.get("received_at");

        messages.push(Mail {
            id: id.to_string(),
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

pub fn mail_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/mail").wrap(RateLimit)
            .service(await_for_new_mail)
            .service(get_messages),
    );
}

use actix_web::{HttpResponse, Responder, post, web};
use log::{error, info, warn};
use mail::models::Mail;
use sqlx::{Executor, Row};
use uuid::Uuid;

use crate::State;

#[post("/")]
async fn add_message(mail: web::Json<Mail>, data: web::Data<State>) -> impl Responder {
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
            "INSERT INTO messages ( mailbox_id, domain, client_ip, sender, subject, message, body, attachments ) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )",
        )
        .bind(mailbox_id)
        .bind(&mail.domain)
        .bind(&mail.client_ip)
        .bind(&mail.sender)
        .bind(&mail.subject)
        .bind(&mail.message)
        .bind(&mail.body)
        .bind(&mail.attachments);

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

pub fn store_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/store").service(add_message));
}

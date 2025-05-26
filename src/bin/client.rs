use dotenv::dotenv;
use mail_proto::mail_proxy_client::MailProxyClient;
use mail_proto::{Mail, MailRequest};
use sqlx::{Executor, Row};
use sqlx_postgres::PgPool;
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use uuid::Uuid;
use log::{info, warn, error, LevelFilter};
use simple_logger::SimpleLogger;

pub mod mail_proto {
    tonic::include_proto!("mail");
}

mod client_modules;

use client_modules::request::Request;
use client_modules::state::State;

use crate::mail_proto::MailResponse;
use mail::error::MyError;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    dotenv().ok();

    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();


    let listener = TcpListener::bind("localhost:4000").await?;

    // let addr = "[::1]:50051".parse().unwrap();
    let db_url = env::var("DB_URL").expect("Cant find DB_URL in .env");

    let pool = PgPool::connect(&db_url).await.unwrap();

    let (tx, mut rx) = mpsc::channel::<MailRequest>(32);

    tokio::spawn(async move {
        // let mut client = MailProxyClient::connect("http://[::1]:50051")
        //     .await
        //     .unwrap();

        while let Some(mail_request) = rx.recv().await {
            if let Some(mail) = mail_request.mail {
                let receiver = mail
                    .receivers
                    .first()
                    .expect("should be at least one receiver");

                let mailbox_query =
                    sqlx::query("SELECT * from mailboxes where email = $1 and status != 'expired'")
                        .bind(receiver)
                        .fetch_one(&pool)
                        .await;

                if let Ok(row) = mailbox_query {
                    let mailbox_id: Uuid = row.get("id");

                    let query =
                        sqlx::query("INSERT INTO messages ( mailbox_id, sender, subject, body ) VALUES ( $1, $2, $3, $4 )")
                            .bind(mailbox_id)
                            .bind(mail.sender)
                            .bind(mail.subject)
                            .bind(mail.message);

                    let result = pool.execute(query).await.unwrap();

                    if result.rows_affected() < 1 {
                        error!("Error");
                    } else {
                        info!("Success");
                    }
                } else {
                    warn!("Cant find email: {}", receiver)
                }
            }

            // client.send_mail(mail).await.unwrap();
        }
    });

    loop {
        let (mut socket, _) = listener.accept().await?;
        let mut state = State::new(tx.clone());

        info!("New connection: {}", socket.peer_addr()?);

        socket
            .write_all(b"220 smtp.example.com ESMTP ready\r\n")
            .await
            .unwrap();

        let mut buf = [0; 4096];

        tokio::spawn(async move {
            loop {
                let n = socket.read(&mut buf[..]).await.unwrap();

                if n == 0 {
                    break;
                }

                let request = Request::from_bytes(&buf[..n]);

                if let Ok(req) = request {
                    state.handle_request(req).await;

                    socket.write_all(b"250 Ok\r\n").await.unwrap();
                } else {
                    socket.write_all(b"500 Ok\r\n").await.unwrap();
                }
            }
        });
    }
}

use dotenv::dotenv;
use log::{LevelFilter, error, info, warn};
use reqwest;
use serde_json::json;
use simple_logger::SimpleLogger;
use sqlx::{Executor, Row};
use sqlx_postgres::PgPool;
use std::env;
use mailparse::parse_mail;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use uuid::Uuid;

mod client_modules;

use client_modules::request::Request;
use client_modules::state::State;

use mail::error::MyError;
use mail::models::Mail;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    dotenv().ok();

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let listener = TcpListener::bind("localhost:4000").await?;

    let (tx, mut rx) = mpsc::channel::<Mail>(32);

    tokio::spawn(async move {
        let client = reqwest::Client::new();

        while let Some(mail) = rx.recv().await {
            info!("{:?}", mail);

            let res = client
                .post("http://localhost:8000/api/mail")
                .json(&mail)
                .send()
                .await;

            if let Err(_) = res {
                // TODO store somewhere and send it after success
                error!("Failed to send mail to server");
            }
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

                info!("Bytes read from socket{}", n);

                let request = if state.get_data() {
                    match parse_mail(&buf) {
                        Ok(parsed) => Ok(Request::Payload(parsed)),
                        Err(_) => Err(MyError::ParseError),
                    }
                } else {
                    Request::from_bytes(&buf[..n])
                };

                // info!("Request: {:?}", request.);

                match request {
                    Ok(r) => {
                        match r {
                            Request::Hello(domain) => {
                                state.add_domain(domain);
                                socket.write_all(b"250 Ok\r\n").await.unwrap();
                            }
                            Request::Mail(sender) => {
                                state.add_sender(sender);
                                socket.write_all(b"250 Ok\r\n").await.unwrap();
                            }
                            Request::Reset => {
                                state.reset();
                                socket.write_all(b"250 Ok\r\n").await.unwrap();
                            }
                            Request::Recipient(recipient) => {
                                state.add_recipient(recipient);
                                socket.write_all(b"250 Ok\r\n").await.unwrap();
                            }
                            Request::Data => {
                                state.set_data(true);
                                socket
                                    .write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
                                    .await
                                    .unwrap();
                            }
                            Request::Payload(mail) => {
                                let result = state.handle_data(mail).await;

                                match result {
                                    Ok(_) => {
                                        info!("Payload handled successfully");
                                        state.set_data(false);
                                        socket.write_all(b"250 Ok\r\n").await.unwrap();
                                    },
                                    Err(e) => {
                                        error!("Error while in payload message {}", e);

                                    },
                                }
                            }
                            Request::Quit => {
                                socket.write_all(b"221 Bye\r\n").await.unwrap();
                            }
                        }
                    }
                    Err(_) => {
                        socket.write_all(b"500 Ok\r\n").await.unwrap();
                    }
                }
            }
        });
    }
}

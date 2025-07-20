use log::{LevelFilter, error, info, warn};
use mailparse::parse_mail;
use reqwest;
use reqwest::{Response, StatusCode};
use serde_json::json;
use simple_logger::SimpleLogger;
use sqlx::{Executor, Row};
use sqlx_postgres::PgPool;
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::{Utc};

mod client_modules;

use client_modules::request::Request;
use client_modules::state::State;

use mail::error::MyError;
use mail::models::Mail;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let host = env::var("HOST_ADDRESS").unwrap_or_else(|_| "localhost".into());
    let host_port = env::var("HOST_PORT").unwrap_or_else(|_| "4000".into());
    let server = env::var("SERVER_ADDRESS").unwrap_or_else(|_| "localhost".into());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8000".into());

    let listener = TcpListener::bind(format!("{}:{}", &host, &host_port)).await?;

    let (tx, mut rx) = mpsc::channel::<Mail>(32);

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/api/store/", &server, &server_port);

        info!("Sending to host: {}", &url);

        while let Some(mail) = rx.recv().await {
            let res = client
                .post(&url)
                .json(&mail)
                .send()
                .await;

            match res {
                Ok(res) => {
                    if res.status() != StatusCode::OK {
                        error!(
                            "Response failed: {}, {}",
                            res.status(),
                            res.text().await.unwrap()
                        );
                    } else {
                        info!("Mail delivered successfully");
                    }
                }
                Err(e) => {
                    error!("Failed to send mail to server: {}", e);
                }
            }
        }
    });

    loop {
        let (mut socket, client_addr) = listener.accept().await?;
        let mut state = State::new(tx.clone(), client_addr);

        info!("New connection: {}", socket.peer_addr()?);

        socket
            .write_all(b"220 mail.temp.local\r\n")
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
                    Ok(r) => match r {
                        Request::Hello(domain) => {
                            state.add_domain(domain);
                            // TODO send supported features
                            // socket.write_all(b"250 mail.temp.local\r\n").await.unwrap();

                            socket.write_all(b"250-mail.temp.local\r\n").await.unwrap();
                            socket.write_all(b"250-PIPELINING\r\n").await.unwrap();
                            socket.write_all(b"250-SIZE 10485760\r\n").await.unwrap();
                            socket.write_all(b"250-8BITMIME\r\n").await.unwrap();
                            // socket.write_all(b"250-ENHANCEDSTATUSCODES\r\n").await.unwrap();
                            socket.write_all(b"250 SMTPUTF8\r\n").await.unwrap();
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
                            // TODO check if recipient valid
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
                                }
                                Err(e) => {
                                    error!("Error while in payload message {}", e);
                                }
                            }
                        }
                        Request::Quit => {
                            socket.write_all(b"221 Bye\r\n").await.unwrap();
                        }
                    },
                    Err(_) => {
                        socket.write_all(b"500 Ok\r\n").await.unwrap();
                    }
                }
            }
        });
    }
}

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use mail_proto::MailRequest;
use mail_proto::mail_proxy_client::MailProxyClient;

pub mod mail_proto {
    tonic::include_proto!("mail");
}

mod client_modules;

use client_modules::request::Request;
use client_modules::state::State;

use mail::error::MyError;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let listener = TcpListener::bind("localhost:4000").await?;

    let (tx, mut rx) = mpsc::channel(32);

    tokio::spawn(async move {
        let mut client = MailProxyClient::connect("http://[::1]:50051")
            .await
            .unwrap();

        while let Some(mail) = rx.recv().await {
            println!("{:?}", mail);
            client.send_mail(mail).await.unwrap();
        }
    });

    loop {
        let (mut socket, _) = listener.accept().await?;
        let mut state = State::new(tx.clone());

        println!("New connection: {}", socket.peer_addr()?);

        socket
            .write_all(b"220 smtp.example.com ESMTP ready\r\n")
            .await
            .unwrap();

        let mut buf = [0; 4096];

        tokio::spawn(async move {
            loop {
                let n = socket.read(&mut buf[..]).await.unwrap();

                println!("Received: {}", n);

                if n == 0 {
                    break;
                }

                let request = Request::from_bytes(&buf[..n]);

                println!("raw request from client: {:?}", n);

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

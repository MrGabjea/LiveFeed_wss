use crate::config::{IP_ADDRESS, PORT};
use crate::state::StdEvent;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};
use tokio_tungstenite::accept_async;

pub async fn start_websocket_server(tx: broadcast::Sender<StdEvent>) {
    let addr = format!("{}:{}", IP_ADDRESS, PORT);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("WebSocket server running on ws://{}", addr);

    loop {
        if let Ok((stream, addr)) = listener.accept().await {
            println!("Client connected: {}", addr);
            let rx = tx.subscribe();
            tokio::spawn(handle_client(stream, rx, addr));
        }
    }
}

async fn handle_client(
    stream: tokio::net::TcpStream,
    mut rx: broadcast::Receiver<StdEvent>,
    addr: SocketAddr,
) {
    let ws_stream = accept_async(stream).await.unwrap();
    let (ws_sender, mut ws_receiver) = ws_stream.split();

    let ws_sender = Arc::new(Mutex::new(ws_sender));

    // Task to send events to client
    let send_sender = ws_sender.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            let mut sender = send_sender.lock().await;
            if sender
                .send(tokio_tungstenite::tungstenite::Message::Text(json))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Task to handle incoming messages (including Ping)
    let recv_sender = ws_sender.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Ping(payload)) => {
                    // println!("Received Ping, sending Pong");
                    let mut sender = recv_sender.lock().await;
                    if sender
                        .send(tokio_tungstenite::tungstenite::Message::Pong(payload))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(tokio_tungstenite::tungstenite::Message::Close(_frame)) => {
                    println!("Connexion closed with {:?}", addr);
                }
                Ok(_) => println!("Unknown message sent by client {:?}", addr),
                Err(_) => break,
            }
        }
    });
    let (res_send, res_recv) = tokio::join!(send_task, recv_task);
    if let Err(e) = res_send {
        eprintln!("send_task for {} panicked: {:?}", addr, e);
    }
    if let Err(e) = res_recv {
        eprintln!("recv_task for {} panicked: {:?}", addr, e);
    }
}

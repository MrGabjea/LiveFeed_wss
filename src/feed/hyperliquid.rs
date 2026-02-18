use super::LiveEvent;
use crate::config::{COIN, WSS_HYPERLIQUID};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use serde_json::json;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

pub async fn hyperliquid_feed(sender: Sender<LiveEvent>) -> anyhow::Result<()> {
    loop {
        match connect_async(WSS_HYPERLIQUID).await {
            Ok(ws) => {
                let (ws_stream, _) = ws;
                println!("CONNECTED TO: {}", WSS_HYPERLIQUID);
                let (mut write, mut read) = ws_stream.split();
                let subscribe_msg = json!({
                    "method": "subscribe",
                    "subscription": {
                        "type": "l2Book",
                        "coin": COIN
                    }
                });
                if let Err(e) = write.send(Message::Text(subscribe_msg.to_string())).await {
                    eprintln!("ERROR IN SUBSCRIPTION: {}", e);

                    continue;
                } else {
                    println!("SUBSCRIBED TO HYPERLIQUID ORDERBOOK")
                }

                // Loop Message Treatment
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Ok(res) = serde_json::from_str::<Value>(&text) {
                                if let (Some(time_u64), Some(Value::Array(l2_book))) = (
                                    res.get("data")
                                        .and_then(|d| d.get("time"))
                                        .and_then(|t| t.as_u64()),
                                    res.get("data").and_then(|k| k.get("levels")),
                                ) {
                                    let event = LiveEvent::Hyperliquid {
                                        timestamp: time_u64,
                                        levels: l2_book.clone(),
                                    };
                                    if sender.send(event).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            println!("CONNEXION CLOSED");
                            break;
                        }
                        Err(e) => {
                            eprintln!("WEBSOCKET ERROR: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("CANNOT CONNECT TO WEBSOCKET: {}", e);
            }
        }

        // Waiting before trying to reconnect
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("HYPERLIQUID FEED RESTARTING {}", WSS_HYPERLIQUID);
    }
}

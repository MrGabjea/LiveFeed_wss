mod config;
mod feed;
use feed::LiveEvent;
mod state;
use state::{StdEvent, create_std_event};
mod websocket;
use websocket::start_websocket_server;

use tokio::sync::{broadcast, mpsc};

#[tokio::main]
async fn main() {
    let mut market = state::market_state::MarketState::default();

    // Initialize & Run LiveFeed
    let (tx, mut rx) = mpsc::channel::<LiveEvent>(1024);
    tokio::spawn(feed::start_livefeed(tx));

    // Initialize & Run Webscoket
    let (ws_tx, _) = broadcast::channel::<StdEvent>(1024);
    let ws_tx_clone = ws_tx.clone();
    tokio::spawn(async move {
        start_websocket_server(ws_tx_clone).await;
    });

    // Central Loop : Triggered when Event received
    while let Some(event) = rx.recv().await {
        if market.update(&event) {
            let event_to_broadcast = create_std_event(&market, event);
            let _ = ws_tx.send(event_to_broadcast);
        }
    }
}

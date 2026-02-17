mod feed;
mod config;
use feed::LiveEvent;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Initialize & Run LiveFeed
    let (tx, mut rx) = mpsc::channel::<LiveEvent>(1024);
    tokio::spawn(feed::start_livefeed(tx));
    
    // Central Loop : Triggered when Event received
    while let Some(event) = rx.recv().await {
        
        match event {
            LiveEvent::Uniswap {
                block_number,
                amount0 : amo,
                .. } => {
                println!("[UNI]: {}, {:?}",block_number,amo);
            }
            LiveEvent::Hyperliquid { 
                timestamp,
                .. } => {
                println!("[HYPER] {}",timestamp);
            }
        }
    }
}

use ethers::types::{I256, U256};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

mod hyperliquid;
mod uniswap;

#[derive(Debug, Clone)]
pub enum LiveEvent {
    Uniswap {
        block_number: u64,
        log_index: u64,
        amount0: I256,
        amount1: I256,
        sqrt_price_x96: U256,
        liquidity: U256,
    },
    Hyperliquid {
        timestamp: u64,
        levels: Vec<Value>,
    },
}

pub async fn start_livefeed(sender: Sender<LiveEvent>) -> anyhow::Result<()> {
    // Start Feeds
    tokio::spawn(uniswap::uniswap_feed(sender.clone()));
    tokio::spawn(hyperliquid::hyperliquid_feed(sender.clone()));
    Ok(())
}

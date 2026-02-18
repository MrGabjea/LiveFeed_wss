pub mod market_state;
use crate::feed::LiveEvent;
use market_state::MarketState;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub enum Event {
    Uniswap,
    Hyperliquid,
}

#[derive(Debug, Clone, Serialize)]
pub struct StdEvent {
    pub timestamp: u128,
    pub event: Event,
    pub data: MarketState,
}

pub fn create_std_event(market_state: &MarketState, event: LiveEvent) -> StdEvent {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    StdEvent {
        timestamp: time,
        event: match event {
            LiveEvent::Uniswap { .. } => Event::Uniswap,
            LiveEvent::Hyperliquid { .. } => Event::Hyperliquid,
        },
        data: *market_state,
    }
}

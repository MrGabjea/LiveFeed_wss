use crate::feed::LiveEvent;
use ethers::types::U256;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct MarketState {
    bid: f64,
    ask: f64,
    #[serde(serialize_with = "u256_to_decimal_string")]
    sqrt_price_x96: U256,
    block_number: u64,
    log_index: u64,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            bid: 0.0,
            ask: 0.0,
            sqrt_price_x96: U256::zero(),
            block_number: 0,
            log_index: 0,
        }
    }
}

impl MarketState {
    pub fn update(&mut self, event: &LiveEvent) -> bool {
        let change: bool;
        match event {
            LiveEvent::Uniswap {
                block_number,
                log_index,
                sqrt_price_x96: sqrtprice,
                ..
            } => {
                if self.block_number < *block_number
                    || (self.block_number == *block_number && self.log_index < *log_index)
                {
                    self.sqrt_price_x96 = *sqrtprice;
                    self.block_number = *block_number;
                    self.log_index = *log_index;
                    change = true;
                } else {
                    change = false;
                }
            }
            LiveEvent::Hyperliquid { levels: lev, .. } => {
                let (new_bid, new_ask) = get_bid_ask(lev);
                if new_ask != self.ask || new_bid != self.bid {
                    self.ask = new_ask;
                    self.bid = new_bid;
                    change = true
                } else {
                    change = false;
                }
            }
        }
        change
    }
}

fn get_bid_ask(levels: &Vec<Value>) -> (f64, f64) {
    let bid: f64 = (&levels[0][0]["px"].as_str().unwrap()).parse().unwrap();
    let ask: f64 = (&levels[1][0]["px"].as_str().unwrap()).parse().unwrap();
    return (bid, ask);
}

fn u256_to_decimal_string<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

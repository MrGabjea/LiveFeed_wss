use super::LiveEvent;
use ethers::prelude::*;
use ethers::types::{Filter, H160, H256, I256, U256};
use std::str::FromStr;
use tokio::sync::mpsc::Sender;
use crate::config::{WSS_BLOCKCHAIN,POOL_ADDRESS,SWAP_TOPIC};


pub async fn uniswap_feed(
    sender: Sender<LiveEvent>
) -> anyhow::Result<()> {
    loop {
        match Ws::connect(WSS_BLOCKCHAIN).await {
            Ok(ws) => {
                let provider = Provider::new(ws);
                println!("CONNECTED TO: {}", WSS_BLOCKCHAIN);

                if let Ok(mut stream) = provider.subscribe_logs(
                    &Filter::new()
                        .address(POOL_ADDRESS.parse::<H160>().unwrap())
                        .topic0(H256::from_str(SWAP_TOPIC).unwrap())
                ).await {
                    println!("SUBSCRIBED TO SWAP EVENTS");

                    // Loop Message Treatment
                    while let Some(log) = stream.next().await {
                        let data = log.data.as_ref();
                        let block_number = log.block_number.map(|b| b.as_u64()).unwrap_or(0);
                        let log_index = log.log_index.map(|i| i.as_u64()).unwrap_or(0);

                        if data.len() >= 128 {
                            let amount0 = parse_int256(&data[0..32]);
                            let amount1 = parse_int256(&data[32..64]);
                            let sqrt_price_x96 = U256::from_big_endian(&data[64..96]);
                            let liquidity = U256::from_big_endian(&data[96..128]);

                            let event = LiveEvent::Uniswap {
                                block_number,
                                log_index,
                                amount0,
                                amount1,
                                sqrt_price_x96,
                                liquidity,
                            };

                            if sender.send(event).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("CANNOT CONNECT: {}s", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("UNISWAP FEED RESTARTING");
    }
}


// Helper pour décoder un int256 (complément à deux)
fn parse_int256(bytes: &[u8]) -> I256 {
    I256::from_raw(U256::from_big_endian(bytes))
}
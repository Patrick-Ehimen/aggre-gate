//! Coinbase Exchange Connector (placeholder)

use crate::OrderBookService;
use aggregator_core::{PriceLevelUpdate, Result};
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub struct Coinbase;

impl Coinbase {
    pub fn new() -> Self {
        Coinbase
    }
}

#[async_trait]
impl OrderBookService for Coinbase {
    async fn spawn_order_book_service(
        &self,
        _pair: [&str; 2],
        _order_book_depth: usize,
        _exchange_stream_buffer: usize,
        _price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        // TODO: Implement Coinbase WebSocket connection
        Ok(vec![])
    }
}

impl Default for Coinbase {
    fn default() -> Self {
        Self::new()
    }
}

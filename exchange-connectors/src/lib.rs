//! Exchange connectors for supported cryptocurrency exchanges

pub mod binance;
pub mod bitstamp;
pub mod bybit;
pub mod coinbase;
pub mod kraken;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use aggregator_core::{AggregatorError, PriceLevelUpdate, Result};

#[async_trait]
pub trait OrderBookService {
    /// Spawns an order book service to stream order book data and handle stream events for a specified pair.
    async fn spawn_order_book_service(
        &self,
        pair: [&str; 2],
        order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<()>>>>;
}

// Re-export exchange implementations
pub use binance::Binance;
pub use bitstamp::Bitstamp;
pub use bybit::Bybit;
pub use coinbase::Coinbase;
pub use kraken::Kraken;

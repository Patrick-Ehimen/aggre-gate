//! HashMap-based order book implementation
//! Optimized for fast lookups and updates

use crate::OrderBook;
use aggregator_core::{Ask, Bid, Exchange};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// HashMap-based order book implementation
#[derive(Debug, Clone)]
pub struct HashMapOrderBook {
    bids: Arc<RwLock<HashMap<String, Bid>>>, // key: price_exchange
    asks: Arc<RwLock<HashMap<String, Ask>>>, // key: price_exchange
    bid_prices: Arc<RwLock<Vec<f64>>>,       // sorted bid prices (descending)
    ask_prices: Arc<RwLock<Vec<f64>>>,       // sorted ask prices (ascending)
}

impl HashMapOrderBook {
    /// Create a new HashMap-based order book
    pub fn new() -> Self {
        Self {
            bids: Arc::new(RwLock::new(HashMap::new())),
            asks: Arc::new(RwLock::new(HashMap::new())),
            bid_prices: Arc::new(RwLock::new(Vec::new())),
            ask_prices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Generate key for price level
    fn generate_key(price: f64, exchange: &Exchange) -> String {
        format!("{:.8}_{}", price, exchange)
    }

    /// Sort and maintain bid prices (descending)
    async fn sort_bid_prices(&self) {
        let bids = self.bids.read().await;
        let mut prices: Vec<f64> = bids.values().map(|b| b.price).collect();
        prices.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        prices.dedup();

        let mut bid_prices = self.bid_prices.write().await;
        *bid_prices = prices;
    }

    /// Sort and maintain ask prices (ascending)
    async fn sort_ask_prices(&self) {
        let asks = self.asks.read().await;
        let mut prices: Vec<f64> = asks.values().map(|a| a.price).collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        prices.dedup();

        let mut ask_prices = self.ask_prices.write().await;
        *ask_prices = prices;
    }
}

impl Default for HashMapOrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OrderBook for HashMapOrderBook {
    async fn update_bids(&mut self, bids: Vec<Bid>, max_depth: usize) {
        let mut bid_map = self.bids.write().await;

        for bid in bids {
            let key = Self::generate_key(bid.price, &bid.exchange);
            if bid.quantity > 0.0 {
                bid_map.insert(key, bid);
            } else {
                bid_map.remove(&key);
            }
        }

        // Trim to max depth by removing lowest prices
        if bid_map.len() > max_depth {
            let mut prices: Vec<f64> = bid_map.values().map(|b| b.price).collect();
            prices.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            prices.truncate(max_depth);
            let min_price = prices.last().copied().unwrap_or(0.0);

            bid_map.retain(|_, bid| bid.price >= min_price);
        }

        drop(bid_map);
        self.sort_bid_prices().await;
    }

    async fn update_asks(&mut self, asks: Vec<Ask>, max_depth: usize) {
        let mut ask_map = self.asks.write().await;

        for ask in asks {
            let key = Self::generate_key(ask.price, &ask.exchange);
            if ask.quantity > 0.0 {
                ask_map.insert(key, ask);
            } else {
                ask_map.remove(&key);
            }
        }

        // Trim to max depth by removing highest prices
        if ask_map.len() > max_depth {
            let mut prices: Vec<f64> = ask_map.values().map(|a| a.price).collect();
            prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            prices.truncate(max_depth);
            let max_price = prices.last().copied().unwrap_or(f64::MAX);

            ask_map.retain(|_, ask| ask.price <= max_price);
        }

        drop(ask_map);
        self.sort_ask_prices().await;
    }

    async fn get_best_bid(&self) -> Option<Bid> {
        let bid_prices = self.bid_prices.read().await;
        let best_price = bid_prices.first().copied()?;

        let bids = self.bids.read().await;
        bids.values().find(|b| b.price == best_price).cloned()
    }

    async fn get_best_ask(&self) -> Option<Ask> {
        let ask_prices = self.ask_prices.read().await;
        let best_price = ask_prices.first().copied()?;

        let asks = self.asks.read().await;
        asks.values().find(|a| a.price == best_price).cloned()
    }

    async fn get_best_n_bids(&self, n: usize) -> Vec<Bid> {
        let bid_prices = self.bid_prices.read().await;
        let bids = self.bids.read().await;

        bid_prices
            .iter()
            .take(n)
            .filter_map(|&price| bids.values().find(|b| b.price == price).cloned())
            .collect()
    }

    async fn get_best_n_asks(&self, n: usize) -> Vec<Ask> {
        let ask_prices = self.ask_prices.read().await;
        let asks = self.asks.read().await;

        ask_prices
            .iter()
            .take(n)
            .filter_map(|&price| asks.values().find(|a| a.price == price).cloned())
            .collect()
    }

    async fn get_spread(&self) -> Option<f64> {
        let best_bid = self.get_best_bid().await?;
        let best_ask = self.get_best_ask().await?;
        Some(best_ask.price - best_bid.price)
    }

    async fn clear(&mut self) {
        let mut bids = self.bids.write().await;
        let mut asks = self.asks.write().await;
        let mut bid_prices = self.bid_prices.write().await;
        let mut ask_prices = self.ask_prices.write().await;

        bids.clear();
        asks.clear();
        bid_prices.clear();
        ask_prices.clear();
    }

    async fn bid_depth(&self) -> usize {
        let bids = self.bids.read().await;
        bids.len()
    }

    async fn ask_depth(&self) -> usize {
        let asks = self.asks.read().await;
        asks.len()
    }
}

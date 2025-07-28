//! # BTreeSet-based Order Book Implementation
//!
//! This module provides an order book implementation using Rust's `BTreeSet` data structure.
//! The implementation is optimized for maintaining sorted order automatically and provides
//! efficient lookups and range queries.
//!
//! ## Performance Characteristics
//!
//! - **Insertion**: O(log n) - Maintains sorted order automatically
//! - **Lookup**: O(log n) - Binary search through sorted structure  
//! - **Best Price**: O(1) - First element in sorted set
//! - **Range Queries**: O(log n + k) - Efficient for getting top N orders
//! - **Memory**: Moderate overhead due to tree structure
//!
//! ## Use Cases
//!
//! - General purpose order book where sorted access is important
//! - Scenarios requiring frequent range queries (top N orders)
//! - Applications where insertion order doesn't matter
//! - Systems that benefit from automatic sorting
//!
//! ## Thread Safety
//!
//! All operations are protected by async RwLocks, allowing multiple concurrent readers
//! or a single writer. The Arc<RwLock<>> pattern enables safe sharing across async tasks.

use crate::{BuySide, OrderBook, SellSide};
use aggregator_core::{Ask, Bid, Exchange};
use async_trait::async_trait;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// BTreeSet-based order book implementation
///
/// Uses `BTreeSet` to maintain automatically sorted bid and ask orders.
/// Bids are sorted in descending price order (highest first), while
/// asks are sorted in ascending price order (lowest first).
///
/// # Examples
///
/// ```rust
/// use orderbook_implementations::BTreeOrderBook;
/// use aggregator_core::{Bid, Exchange};
/// use chrono::Utc;
///
/// #[tokio::main]
/// async fn main() {
///     let mut orderbook = BTreeOrderBook::new();
///     
///     let bid = Bid {
///         price: 100.0,
///         quantity: 10.0,
///         exchange: Exchange::Binance,
///         timestamp: Utc::now(),
///     };
///     
///     orderbook.update_bids(vec![bid], 100).await;
///     let best = orderbook.get_best_bid().await;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BTreeOrderBook {
    /// Bid orders sorted by price descending (highest first)
    bids: Arc<RwLock<BTreeSet<Bid>>>,
    /// Ask orders sorted by price ascending (lowest first)  
    asks: Arc<RwLock<BTreeSet<Ask>>>,
}

impl BTreeOrderBook {
    /// Creates a new empty BTreeSet-based order book
    ///
    /// # Returns
    ///
    /// A new `BTreeOrderBook` instance with empty bid and ask sides
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orderbook_implementations::BTreeOrderBook;
    ///
    /// let orderbook = BTreeOrderBook::new();
    /// ```
    pub fn new() -> Self {
        Self {
            bids: Arc::new(RwLock::new(BTreeSet::new())),
            asks: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }

    /// Creates a bid-side only view of this order book
    ///
    /// Returns a `BTreeBidSide` that shares the same underlying bid data
    /// but only exposes bid-related operations.
    ///
    /// # Returns
    ///
    /// A `BTreeBidSide` instance sharing the bid data
    pub fn bid_side(&self) -> BTreeBidSide {
        BTreeBidSide {
            bids: self.bids.clone(),
        }
    }

    /// Creates an ask-side only view of this order book
    ///
    /// Returns a `BTreeAskSide` that shares the same underlying ask data
    /// but only exposes ask-related operations.
    ///
    /// # Returns
    ///
    /// A `BTreeAskSide` instance sharing the ask data
    pub fn ask_side(&self) -> BTreeAskSide {
        BTreeAskSide {
            asks: self.asks.clone(),
        }
    }

    /// Trims the bid side to the specified maximum depth
    ///
    /// Keeps only the best (highest price) bids up to `max_depth`.
    /// This is an internal helper method used during updates.
    ///
    /// # Arguments
    ///
    /// * `max_depth` - Maximum number of bid levels to retain
    async fn trim_bids(&self, max_depth: usize) {
        let mut bids = self.bids.write().await;
        if bids.len() > max_depth {
            // Keep only the top max_depth bids (highest prices)
            let mut new_bids = BTreeSet::new();
            for bid in bids.iter().take(max_depth) {
                new_bids.insert(bid.clone());
            }
            *bids = new_bids;
        }
    }

    /// Trims the ask side to the specified maximum depth
    ///
    /// Keeps only the best (lowest price) asks up to `max_depth`.
    /// This is an internal helper method used during updates.
    ///
    /// # Arguments
    ///
    /// * `max_depth` - Maximum number of ask levels to retain
    async fn trim_asks(&self, max_depth: usize) {
        let mut asks = self.asks.write().await;
        if asks.len() > max_depth {
            // Keep only the top max_depth asks (lowest prices)
            let mut new_asks = BTreeSet::new();
            for ask in asks.iter().take(max_depth) {
                new_asks.insert(ask.clone());
            }
            *asks = new_asks;
        }
    }
}

impl Default for BTreeOrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OrderBook for BTreeOrderBook {
    async fn update_bids(&mut self, bids: Vec<Bid>, max_depth: usize) {
        let mut bid_set = self.bids.write().await;

        for bid in bids {
            if bid.quantity > 0.0 {
                // Remove any existing bid at the same price and exchange
                bid_set.retain(|b| !(b.price == bid.price && b.exchange == bid.exchange));
                // Insert the new bid
                bid_set.insert(bid);
            } else {
                // Remove bid if quantity is 0
                bid_set.retain(|b| !(b.price == bid.price && b.exchange == bid.exchange));
            }
        }

        // Trim to max depth
        if bid_set.len() > max_depth {
            let mut new_bids = BTreeSet::new();
            for bid in bid_set.iter().take(max_depth) {
                new_bids.insert(bid.clone());
            }
            *bid_set = new_bids;
        }
    }

    async fn update_asks(&mut self, asks: Vec<Ask>, max_depth: usize) {
        let mut ask_set = self.asks.write().await;

        for ask in asks {
            if ask.quantity > 0.0 {
                // Remove any existing ask at the same price and exchange
                ask_set.retain(|a| !(a.price == ask.price && a.exchange == ask.exchange));
                // Insert the new ask
                ask_set.insert(ask);
            } else {
                // Remove ask if quantity is 0
                ask_set.retain(|a| !(a.price == ask.price && a.exchange == ask.exchange));
            }
        }

        // Trim to max depth
        if ask_set.len() > max_depth {
            let mut new_asks = BTreeSet::new();
            for ask in ask_set.iter().take(max_depth) {
                new_asks.insert(ask.clone());
            }
            *ask_set = new_asks;
        }
    }

    async fn get_best_bid(&self) -> Option<Bid> {
        let bids = self.bids.read().await;
        bids.iter().next().cloned()
    }

    async fn get_best_ask(&self) -> Option<Ask> {
        let asks = self.asks.read().await;
        asks.iter().next().cloned()
    }

    async fn get_best_n_bids(&self, n: usize) -> Vec<Bid> {
        let bids = self.bids.read().await;
        bids.iter().take(n).cloned().collect()
    }

    async fn get_best_n_asks(&self, n: usize) -> Vec<Ask> {
        let asks = self.asks.read().await;
        asks.iter().take(n).cloned().collect()
    }

    async fn get_spread(&self) -> Option<f64> {
        let best_bid = self.get_best_bid().await?;
        let best_ask = self.get_best_ask().await?;
        Some(best_ask.price - best_bid.price)
    }

    async fn clear(&mut self) {
        let mut bids = self.bids.write().await;
        let mut asks = self.asks.write().await;
        bids.clear();
        asks.clear();
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

/// BTreeSet-based bid side implementation
///
/// Provides bid-only operations on a BTreeSet-backed order book.
/// This is useful when you only need to work with the buy side of the market.
///
/// # Thread Safety
///
/// Uses Arc<RwLock<>> for safe concurrent access across async tasks.
#[derive(Debug, Clone)]
pub struct BTreeBidSide {
    /// Shared bid orders sorted by price descending
    bids: Arc<RwLock<BTreeSet<Bid>>>,
}

impl BTreeBidSide {
    /// Creates a new empty bid-side order book
    ///
    /// # Returns
    ///
    /// A new `BTreeBidSide` instance with no bid orders
    pub fn new() -> Self {
        Self {
            bids: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

#[async_trait]
impl BuySide for BTreeBidSide {
    async fn update_bids(&mut self, bids: Vec<Bid>, max_depth: usize) {
        let mut bid_set = self.bids.write().await;

        for bid in bids {
            if bid.quantity > 0.0 {
                bid_set.retain(|b| !(b.price == bid.price && b.exchange == bid.exchange));
                bid_set.insert(bid);
            } else {
                bid_set.retain(|b| !(b.price == bid.price && b.exchange == bid.exchange));
            }
        }

        if bid_set.len() > max_depth {
            let mut new_bids = BTreeSet::new();
            for bid in bid_set.iter().take(max_depth) {
                new_bids.insert(bid.clone());
            }
            *bid_set = new_bids;
        }
    }

    async fn get_best_bid(&self) -> Option<Bid> {
        let bids = self.bids.read().await;
        bids.iter().next().cloned()
    }

    async fn get_best_n_bids(&self, n: usize) -> Vec<Bid> {
        let bids = self.bids.read().await;
        bids.iter().take(n).cloned().collect()
    }

    async fn bid_depth(&self) -> usize {
        let bids = self.bids.read().await;
        bids.len()
    }

    async fn clear_bids(&mut self) {
        let mut bids = self.bids.write().await;
        bids.clear();
    }
}

/// BTreeSet-based ask side implementation
///
/// Provides ask-only operations on a BTreeSet-backed order book.
/// This is useful when you only need to work with the sell side of the market.
///
/// # Thread Safety
///
/// Uses Arc<RwLock<>> for safe concurrent access across async tasks.
#[derive(Debug, Clone)]
pub struct BTreeAskSide {
    /// Shared ask orders sorted by price ascending
    asks: Arc<RwLock<BTreeSet<Ask>>>,
}

impl BTreeAskSide {
    /// Creates a new empty ask-side order book
    ///
    /// # Returns
    ///
    /// A new `BTreeAskSide` instance with no ask orders
    pub fn new() -> Self {
        Self {
            asks: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

#[async_trait]
impl SellSide for BTreeAskSide {
    async fn update_asks(&mut self, asks: Vec<Ask>, max_depth: usize) {
        let mut ask_set = self.asks.write().await;

        for ask in asks {
            if ask.quantity > 0.0 {
                ask_set.retain(|a| !(a.price == ask.price && a.exchange == ask.exchange));
                ask_set.insert(ask);
            } else {
                ask_set.retain(|a| !(a.price == ask.price && a.exchange == ask.exchange));
            }
        }

        if ask_set.len() > max_depth {
            let mut new_asks = BTreeSet::new();
            for ask in ask_set.iter().take(max_depth) {
                new_asks.insert(ask.clone());
            }
            *ask_set = new_asks;
        }
    }

    async fn get_best_ask(&self) -> Option<Ask> {
        let asks = self.asks.read().await;
        asks.iter().next().cloned()
    }

    async fn get_best_n_asks(&self, n: usize) -> Vec<Ask> {
        let asks = self.asks.read().await;
        asks.iter().take(n).cloned().collect()
    }

    async fn ask_depth(&self) -> usize {
        let asks = self.asks.read().await;
        asks.len()
    }

    async fn clear_asks(&mut self) {
        let mut asks = self.asks.write().await;
        asks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_btree_orderbook_basic_operations() {
        let mut orderbook = BTreeOrderBook::new();

        // Test adding bids
        let bid1 = Bid {
            price: 100.0,
            quantity: 10.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        };
        let bid2 = Bid {
            price: 99.0,
            quantity: 5.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        };

        orderbook
            .update_bids(vec![bid1.clone(), bid2.clone()], 10)
            .await;

        // Test best bid (should be highest price)
        let best_bid = orderbook.get_best_bid().await.unwrap();
        assert_eq!(best_bid.price, 100.0);

        // Test adding asks
        let ask1 = Ask {
            price: 101.0,
            quantity: 8.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        };
        let ask2 = Ask {
            price: 102.0,
            quantity: 3.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        };

        orderbook
            .update_asks(vec![ask1.clone(), ask2.clone()], 10)
            .await;

        // Test best ask (should be lowest price)
        let best_ask = orderbook.get_best_ask().await.unwrap();
        assert_eq!(best_ask.price, 101.0);

        // Test spread
        let spread = orderbook.get_spread().await.unwrap();
        assert_eq!(spread, 1.0); // 101.0 - 100.0

        // Test depth
        assert_eq!(orderbook.bid_depth().await, 2);
        assert_eq!(orderbook.ask_depth().await, 2);
    }

    #[tokio::test]
    async fn test_btree_orderbook_update_existing() {
        let mut orderbook = BTreeOrderBook::new();

        // Add initial bid
        let bid1 = Bid {
            price: 100.0,
            quantity: 10.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        };
        orderbook.update_bids(vec![bid1.clone()], 10).await;

        // Update with new quantity at same price
        let bid2 = Bid {
            price: 100.0,
            quantity: 20.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        };
        orderbook.update_bids(vec![bid2.clone()], 10).await;

        // Should have only one bid with updated quantity
        assert_eq!(orderbook.bid_depth().await, 1);
        let best_bid = orderbook.get_best_bid().await.unwrap();
        assert_eq!(best_bid.quantity, 20.0);
    }

    #[tokio::test]
    async fn test_btree_orderbook_remove_zero_quantity() {
        let mut orderbook = BTreeOrderBook::new();

        // Add initial bid
        let bid1 = Bid {
            price: 100.0,
            quantity: 10.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        };
        orderbook.update_bids(vec![bid1.clone()], 10).await;
        assert_eq!(orderbook.bid_depth().await, 1);

        // Remove by setting quantity to 0
        let bid2 = Bid {
            price: 100.0,
            quantity: 0.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        };
        orderbook.update_bids(vec![bid2.clone()], 10).await;

        // Should have no bids
        assert_eq!(orderbook.bid_depth().await, 0);
        assert!(orderbook.get_best_bid().await.is_none());
    }
}

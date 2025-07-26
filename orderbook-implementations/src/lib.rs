//! # Order Book Implementations
//!
//! This crate provides different order book data structure implementations optimized for various use cases.
//! Each implementation provides the same interface through the `OrderBook` trait but uses different
//! underlying data structures for optimal performance characteristics.
//!
//! ## Available Implementations
//!
//! - **BTreeSet**: Maintains sorted order automatically, good for general use
//! - **HashMap**: Fast lookups and updates, requires manual sorting for best prices
//! - **AVL Tree**: Balanced tree implementation (placeholder)
//! - **Red-Black Tree**: Self-balancing binary search tree (placeholder)
//!
//! ## Usage
//!
//! use orderbook_implementations::{BTreeOrderBook, OrderBook};
//! use aggregator_core::{Bid, Ask, Exchange};
//! use chrono::Utc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut orderbook = BTreeOrderBook::new();
//!     
//!     let bid = Bid {
//!         price: 100.0,
//!         quantity: 10.0,
//!         exchange: Exchange::Binance,
//!         timestamp: Utc::now(),
//!     };
//!     
//!     orderbook.update_bids(vec![bid], 100).await;
//!     let best_bid = orderbook.get_best_bid().await;
//! }

pub mod avl_tree;
pub mod btree_set;
pub mod hashmap;
pub mod rb_tree;

use aggregator_core::{Ask, Bid, PriceLevel, Result};
use async_trait::async_trait;

/// Core trait for order book implementations
///
/// This trait defines the standard interface that all order book implementations must provide.
/// It supports both bid and ask operations, depth management, and spread calculations.
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to support concurrent access across async tasks.
///
/// # Performance Considerations
///
/// - `update_bids`/`update_asks`: Should handle batch updates efficiently
/// - `get_best_*`: Should be O(1) or O(log n) for optimal performance
/// - `max_depth`: Limits memory usage and maintains only the most relevant price levels
#[async_trait]
pub trait OrderBook: Send + Sync {
    /// Updates the bid side of the order book with new data
    ///
    /// # Arguments
    ///
    /// * `bids` - Vector of bid orders to update
    /// * `max_depth` - Maximum number of price levels to maintain
    ///
    /// # Behavior
    ///
    /// - Orders with quantity > 0.0 are inserted/updated
    /// - Orders with quantity = 0.0 are removed
    /// - Duplicate price/exchange combinations are replaced
    /// - Only the best `max_depth` levels are kept
    async fn update_bids(&mut self, bids: Vec<Bid>, max_depth: usize);

    /// Updates the ask side of the order book with new data
    ///
    /// # Arguments
    ///
    /// * `asks` - Vector of ask orders to update  
    /// * `max_depth` - Maximum number of price levels to maintain
    ///
    /// # Behavior
    ///
    /// - Orders with quantity > 0.0 are inserted/updated
    /// - Orders with quantity = 0.0 are removed
    /// - Duplicate price/exchange combinations are replaced
    /// - Only the best `max_depth` levels are kept
    async fn update_asks(&mut self, asks: Vec<Ask>, max_depth: usize);

    /// Returns the best (highest price) bid order
    ///
    /// # Returns
    ///
    /// `Some(Bid)` if bids exist, `None` if the bid side is empty
    async fn get_best_bid(&self) -> Option<Bid>;

    /// Returns the best (lowest price) ask order
    ///
    /// # Returns
    ///
    /// `Some(Ask)` if asks exist, `None` if the ask side is empty
    async fn get_best_ask(&self) -> Option<Ask>;

    /// Returns the best N bid orders sorted by price (highest first)
    ///
    /// # Arguments
    ///
    /// * `n` - Maximum number of bids to return
    ///
    /// # Returns
    ///
    /// Vector of up to `n` best bids, may be shorter if fewer bids exist
    async fn get_best_n_bids(&self, n: usize) -> Vec<Bid>;

    /// Returns the best N ask orders sorted by price (lowest first)
    ///
    /// # Arguments
    ///
    /// * `n` - Maximum number of asks to return
    ///
    /// # Returns
    ///
    /// Vector of up to `n` best asks, may be shorter if fewer asks exist
    async fn get_best_n_asks(&self, n: usize) -> Vec<Ask>;

    /// Calculates the current bid-ask spread
    ///
    /// # Returns
    ///
    /// `Some(f64)` representing the spread (best_ask - best_bid),
    /// `None` if either side is empty
    async fn get_spread(&self) -> Option<f64>;

    /// Clears all orders from both sides of the order book
    async fn clear(&mut self);

    /// Returns the number of bid price levels
    async fn bid_depth(&self) -> usize;

    /// Returns the number of ask price levels  
    async fn ask_depth(&self) -> usize;
}

/// Trait for buy-side only order book operations
///
/// This trait allows for specialized implementations that only handle the bid side,
/// useful for scenarios where bid and ask sides are managed separately.
#[async_trait]
pub trait BuySide: Send + Sync {
    /// Updates bid orders with depth management
    async fn update_bids(&mut self, bids: Vec<Bid>, max_depth: usize);

    /// Gets the best (highest price) bid
    async fn get_best_bid(&self) -> Option<Bid>;

    /// Gets the best N bids sorted by price descending
    async fn get_best_n_bids(&self, n: usize) -> Vec<Bid>;

    /// Returns the number of bid price levels
    async fn bid_depth(&self) -> usize;

    /// Clears all bid orders
    async fn clear_bids(&mut self);
}

/// Trait for sell-side only order book operations
///
/// This trait allows for specialized implementations that only handle the ask side,
/// useful for scenarios where bid and ask sides are managed separately.
#[async_trait]
pub trait SellSide: Send + Sync {
    /// Updates ask orders with depth management
    async fn update_asks(&mut self, asks: Vec<Ask>, max_depth: usize);

    /// Gets the best (lowest price) ask
    async fn get_best_ask(&self) -> Option<Ask>;

    /// Gets the best N asks sorted by price ascending
    async fn get_best_n_asks(&self, n: usize) -> Vec<Ask>;

    /// Returns the number of ask price levels
    async fn ask_depth(&self) -> usize;

    /// Clears all ask orders
    async fn clear_asks(&mut self);
}

// Re-export implementations
pub use btree_set::BTreeOrderBook;
pub use hashmap::HashMapOrderBook;

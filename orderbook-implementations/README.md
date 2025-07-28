# Order Book Implementations

This crate provides different order book data structure implementations optimized for various use cases in cryptocurrency trading systems. Each implementation provides the same interface through the `OrderBook` trait but uses different underlying data structures for optimal performance characteristics.

## Features

- **Multiple Implementations**: BTreeSet, HashMap, AVL Tree, Red-Black Tree
- **Async/Await Support**: All operations are async-compatible
- **Thread Safety**: Built with Arc<RwLock<>> for safe concurrent access
- **Depth Management**: Automatic trimming to configurable maximum depth
- **Exchange Support**: Handle orders from multiple exchanges at same price levels
- **Comprehensive Testing**: Extensive test suite covering edge cases and performance

## Available Implementations

### BTreeSet Implementation (`BTreeOrderBook`)

**Best for**: General purpose use, sorted access patterns

- **Insertion**: O(log n) - Maintains sorted order automatically
- **Lookup**: O(log n) - Binary search through sorted structure
- **Best Price**: O(1) - First element in sorted set
- **Range Queries**: O(log n + k) - Efficient for getting top N orders
- **Memory**: Moderate overhead due to tree structure

```rust
use orderbook_implementations::BTreeOrderBook;

let mut orderbook = BTreeOrderBook::new();
```

### HashMap Implementation (`HashMapOrderBook`)

**Best for**: High-frequency updates, fast lookups

- **Insertion**: O(1) average - Direct hash table access
- **Lookup**: O(1) average - Hash table lookup
- **Best Price**: O(n) - Requires scanning sorted price list
- **Range Queries**: O(n) - Must filter through price levels
- **Memory**: Lower overhead, separate price sorting

```rust
use orderbook_implementations::HashMapOrderBook;

let mut orderbook = HashMapOrderBook::new();
```

### Future Implementations

- **AVL Tree**: Self-balancing binary search tree (placeholder)
- **Red-Black Tree**: Alternative balanced tree implementation (placeholder)

## Usage

### Basic Operations

```rust
use orderbook_implementations::{BTreeOrderBook, OrderBook};
use aggregator_core::{Bid, Ask, Exchange};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut orderbook = BTreeOrderBook::new();

    // Add some bids
    let bids = vec![
        Bid {
            price: 100.0,
            quantity: 10.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        },
        Bid {
            price: 99.5,
            quantity: 5.0,
            exchange: Exchange::Coinbase,
            timestamp: Utc::now(),
        },
    ];

    orderbook.update_bids(bids, 100).await;

    // Add some asks
    let asks = vec![
        Ask {
            price: 101.0,
            quantity: 8.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        },
    ];

    orderbook.update_asks(asks, 100).await;

    // Get best prices
    if let Some(best_bid) = orderbook.get_best_bid().await {
        println!("Best bid: {} @ {}", best_bid.quantity, best_bid.price);
    }

    if let Some(best_ask) = orderbook.get_best_ask().await {
        println!("Best ask: {} @ {}", best_ask.quantity, best_ask.price);
    }

    // Calculate spread
    if let Some(spread) = orderbook.get_spread().await {
        println!("Spread: {}", spread);
    }

    // Get top 5 bids and asks
    let top_bids = orderbook.get_best_n_bids(5).await;
    let top_asks = orderbook.get_best_n_asks(5).await;

    println!("Market depth: {} bids, {} asks",
             orderbook.bid_depth().await,
             orderbook.ask_depth().await);

    Ok(())
}
```

### Side-Only Operations

```rust
use orderbook_implementations::{BTreeOrderBook, BuySide, SellSide};

#[tokio::main]
async fn main() {
    let orderbook = BTreeOrderBook::new();

    // Get side-specific views
    let mut bid_side = orderbook.bid_side();
    let mut ask_side = orderbook.ask_side();

    // Work with bids only
    bid_side.update_bids(bids, 100).await;
    let best_bid = bid_side.get_best_bid().await;

    // Work with asks only
    ask_side.update_asks(asks, 100).await;
    let best_ask = ask_side.get_best_ask().await;
}
```

### Order Updates and Removal

```rust
// Update existing order (same price + exchange)
let updated_bid = Bid {
    price: 100.0,
    quantity: 20.0,  // New quantity
    exchange: Exchange::Binance,
    timestamp: Utc::now(),
};
orderbook.update_bids(vec![updated_bid], 100).await;

// Remove order (set quantity to 0)
let remove_bid = Bid {
    price: 100.0,
    quantity: 0.0,  // Zero quantity removes the order
    exchange: Exchange::Binance,
    timestamp: Utc::now(),
};
orderbook.update_bids(vec![remove_bid], 100).await;
```

### Depth Management

```rust
// Limit to top 50 price levels
let max_depth = 50;
orderbook.update_bids(large_bid_list, max_depth).await;
orderbook.update_asks(large_ask_list, max_depth).await;

// Only the best 50 levels will be kept
assert!(orderbook.bid_depth().await <= max_depth);
assert!(orderbook.ask_depth().await <= max_depth);
```

## Performance Considerations

### Choosing an Implementation

- **BTreeOrderBook**: Choose when you need sorted access and don't mind O(log n) operations
- **HashMapOrderBook**: Choose when you need fastest updates and can tolerate O(n) for best price queries
- **Future implementations**: Will provide specialized performance characteristics

### Optimization Tips

1. **Batch Updates**: Update multiple orders in a single call when possible
2. **Appropriate Depth**: Set `max_depth` to limit memory usage and improve performance
3. **Concurrent Access**: Use the shared Arc<RwLock<>> pattern for multiple readers
4. **Avoid Frequent Clearing**: Prefer selective updates over full clears

## Testing

The crate includes comprehensive tests covering:

- **Integration Tests**: Cross-implementation compatibility
- **Unit Tests**: Implementation-specific behavior
- **Edge Cases**: Extreme values, error conditions
- **Performance Benchmarks**: Comparative performance analysis

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration_tests
cargo test --test btree_tests
cargo test --test edge_cases

# Run benchmarks
cargo bench
```

### Test Coverage

- Basic CRUD operations
- Order updates and replacements
- Depth limiting functionality
- Multi-exchange handling
- Concurrent access patterns
- Edge cases (extreme values, empty states)
- Performance under load

## Thread Safety

All implementations are designed for concurrent access:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let orderbook = Arc::new(Mutex::new(BTreeOrderBook::new()));

// Multiple tasks can safely access the orderbook
let orderbook_clone = orderbook.clone();
tokio::spawn(async move {
    let mut ob = orderbook_clone.lock().await;
    ob.update_bids(bids, 100).await;
});
```

## Error Handling

The implementations handle various edge cases gracefully:

- Empty order books return `None` for best price queries
- Zero quantities automatically remove orders
- Depth limiting keeps only the best N levels
- Invalid inputs are handled without panicking

## Contributing

When adding new implementations:

1. Implement the `OrderBook` trait
2. Add comprehensive tests in the `tests/` directory
3. Include benchmarks in `benches/`
4. Update this README with performance characteristics
5. Ensure thread safety with appropriate synchronization

## Dependencies

- `aggregator-core`: Core types (Bid, Ask, Exchange)
- `tokio`: Async runtime and synchronization primitives
- `async-trait`: Async trait support
- `chrono`: Timestamp handling
- `criterion`: Benchmarking framework (dev dependency)

## License

This project is licensed under the same terms as the parent aggregator project.

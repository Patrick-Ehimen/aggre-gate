//! Performance benchmarks for order book implementations
//!
//! These benchmarks compare the performance characteristics of different
//! order book implementations under various workloads.

use aggregator_core::{Ask, Bid, Exchange};
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use orderbook_implementations::{BTreeOrderBook, HashMapOrderBook, OrderBook};
use std::time::Duration;

/// Helper function to create a test bid
fn create_bid(price: f64, quantity: f64, exchange: Exchange) -> Bid {
    Bid {
        price,
        quantity,
        exchange,
        timestamp: Utc::now(),
    }
}

/// Helper function to create a test ask
fn create_ask(price: f64, quantity: f64, exchange: Exchange) -> Ask {
    Ask {
        price,
        quantity,
        exchange,
        timestamp: Utc::now(),
    }
}

/// Benchmark single order insertion
fn bench_single_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_insertion");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("btree", size), size, |b, &size| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut orderbook = BTreeOrderBook::new();
                    for i in 0..size {
                        let bid = create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance);
                        orderbook.update_bids(vec![bid], 1000).await;
                    }
                    black_box(orderbook);
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("hashmap", size), size, |b, &size| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut orderbook = HashMapOrderBook::new();
                    for i in 0..size {
                        let bid = create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance);
                        orderbook.update_bids(vec![bid], 1000).await;
                    }
                    black_box(orderbook);
                });
            });
        });
    }

    group.finish();
}

/// Benchmark batch order insertion
fn bench_batch_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_insertion");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Prepare test data
        let bids: Vec<Bid> = (0..*size)
            .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
            .collect();

        group.bench_with_input(BenchmarkId::new("btree", size), &bids, |b, bids| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut orderbook = BTreeOrderBook::new();
                    orderbook.update_bids(bids.clone(), 1000).await;
                    black_box(orderbook);
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("hashmap", size), &bids, |b, bids| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut orderbook = HashMapOrderBook::new();
                    orderbook.update_bids(bids.clone(), 1000).await;
                    black_box(orderbook);
                });
            });
        });
    }

    group.finish();
}

/// Benchmark getting best price
fn bench_get_best_price(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_best_price");

    for size in [10, 100, 1000].iter() {
        // Prepare orderbooks with data
        let rt = tokio::runtime::Runtime::new().unwrap();

        let btree_orderbook = rt.block_on(async {
            let mut orderbook = BTreeOrderBook::new();
            let bids: Vec<Bid> = (0..*size)
                .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
                .collect();
            orderbook.update_bids(bids, 1000).await;
            orderbook
        });

        let hashmap_orderbook = rt.block_on(async {
            let mut orderbook = HashMapOrderBook::new();
            let bids: Vec<Bid> = (0..*size)
                .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
                .collect();
            orderbook.update_bids(bids, 1000).await;
            orderbook
        });

        group.bench_with_input(
            BenchmarkId::new("btree", size),
            &btree_orderbook,
            |b, orderbook| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let result = orderbook.get_best_bid().await;
                        black_box(result);
                    });
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hashmap", size),
            &hashmap_orderbook,
            |b, orderbook| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let result = orderbook.get_best_bid().await;
                        black_box(result);
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark getting top N orders
fn bench_get_top_n(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_top_n");

    let orderbook_size = 1000;
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Prepare orderbooks with data
    let btree_orderbook = rt.block_on(async {
        let mut orderbook = BTreeOrderBook::new();
        let bids: Vec<Bid> = (0..orderbook_size)
            .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
            .collect();
        orderbook.update_bids(bids, 1000).await;
        orderbook
    });

    let hashmap_orderbook = rt.block_on(async {
        let mut orderbook = HashMapOrderBook::new();
        let bids: Vec<Bid> = (0..orderbook_size)
            .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
            .collect();
        orderbook.update_bids(bids, 1000).await;
        orderbook
    });

    for n in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*n as u64));

        group.bench_with_input(BenchmarkId::new("btree", n), n, |b, &n| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let result = btree_orderbook.get_best_n_bids(n).await;
                    black_box(result);
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("hashmap", n), n, |b, &n| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let result = hashmap_orderbook.get_best_n_bids(n).await;
                    black_box(result);
                });
            });
        });
    }

    group.finish();
}

/// Benchmark order updates (replacing existing orders)
fn bench_order_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_updates");

    for size in [10, 100, 1000].iter() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Prepare initial data
        let initial_bids: Vec<Bid> = (0..*size)
            .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
            .collect();

        // Prepare update data (same prices, different quantities)
        let update_bids: Vec<Bid> = (0..*size)
            .map(|i| create_bid(100.0 - i as f64 * 0.01, 20.0, Exchange::Binance))
            .collect();

        group.bench_with_input(BenchmarkId::new("btree", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let mut orderbook = BTreeOrderBook::new();
                    orderbook.update_bids(initial_bids.clone(), 1000).await;
                    orderbook.update_bids(update_bids.clone(), 1000).await;
                    black_box(orderbook);
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("hashmap", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    let mut orderbook = HashMapOrderBook::new();
                    orderbook.update_bids(initial_bids.clone(), 1000).await;
                    orderbook.update_bids(update_bids.clone(), 1000).await;
                    black_box(orderbook);
                });
            });
        });
    }

    group.finish();
}

/// Benchmark depth limiting performance
fn bench_depth_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("depth_limiting");

    let input_size = 1000;
    let bids: Vec<Bid> = (0..input_size)
        .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
        .collect();

    for max_depth in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("btree", max_depth),
            max_depth,
            |b, &max_depth| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut orderbook = BTreeOrderBook::new();
                        orderbook.update_bids(bids.clone(), max_depth).await;
                        black_box(orderbook);
                    });
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hashmap", max_depth),
            max_depth,
            |b, &max_depth| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut orderbook = HashMapOrderBook::new();
                        orderbook.update_bids(bids.clone(), max_depth).await;
                        black_box(orderbook);
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark mixed read/write workload
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");
    group.measurement_time(Duration::from_secs(10));

    for size in [100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("btree", size), size, |b, &size| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut orderbook = BTreeOrderBook::new();

                    // Initial population
                    let initial_bids: Vec<Bid> = (0..size)
                        .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
                        .collect();
                    orderbook.update_bids(initial_bids, 1000).await;

                    // Mixed operations
                    for i in 0..50 {
                        // Update some orders
                        let update_bid =
                            create_bid(100.0 - i as f64 * 0.01, 15.0, Exchange::Binance);
                        orderbook.update_bids(vec![update_bid], 1000).await;

                        // Read operations
                        let _ = orderbook.get_best_bid().await;
                        let _ = orderbook.get_best_n_bids(10).await;
                        let _ = orderbook.bid_depth().await;
                    }

                    black_box(orderbook);
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("hashmap", size), size, |b, &size| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut orderbook = HashMapOrderBook::new();

                    // Initial population
                    let initial_bids: Vec<Bid> = (0..size)
                        .map(|i| create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance))
                        .collect();
                    orderbook.update_bids(initial_bids, 1000).await;

                    // Mixed operations
                    for i in 0..50 {
                        // Update some orders
                        let update_bid =
                            create_bid(100.0 - i as f64 * 0.01, 15.0, Exchange::Binance);
                        orderbook.update_bids(vec![update_bid], 1000).await;

                        // Read operations
                        let _ = orderbook.get_best_bid().await;
                        let _ = orderbook.get_best_n_bids(10).await;
                        let _ = orderbook.bid_depth().await;
                    }

                    black_box(orderbook);
                });
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_insertion,
    bench_batch_insertion,
    bench_get_best_price,
    bench_get_top_n,
    bench_order_updates,
    bench_depth_limiting,
    bench_mixed_workload
);

criterion_main!(benches);

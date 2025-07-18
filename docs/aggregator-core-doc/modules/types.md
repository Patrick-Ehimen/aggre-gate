# Types Module

## Overview

The types module defines core data types and interfaces used throughout the aggregator-core library.

## Core Types

- **PriceLevel, Bid, Ask**: These define order book levels.
  - `PriceLevel` comprises `price`, `quantity`, `exchange`, and `timestamp`.
  - Bids and Asks have custom ordering semantics; higher bid prices are better, lower ask prices are better.
- **Summary**: Aggregates data across exchanges for a trading pair.

### TradingPair Parsing Rules
The `TradingPair` is parsed from a string like "BTC/USD".
```rust
let pair = TradingPair::from_str("BTC/USD").unwrap();
```

### Constructing Updates Example

```rust
let update = PriceLevelUpdate {
    id: Uuid::new_v4(),
    symbol: "BTCUSD".to_string(),
    exchange: Exchange::Binance,
    bids: vec![Bid::default()],
    asks: vec![Ask::default()],
    timestamp: Utc::now(),
};
```

## Detailed Field/Function Tables

### Exchange Enum

| Variant | String Representation | Description |
|---------|----------------------|-------------|
| `Binance` | "binance" | Binance exchange |
| `Bitstamp` | "bitstamp" | Bitstamp exchange |
| `Bybit` | "bybit" | Bybit exchange |
| `Kraken` | "kraken" | Kraken exchange |
| `Coinbase` | "coinbase" | Coinbase exchange |
| `CryptoDotCom` | "crypto_dot_com" | Crypto.com exchange |
| `OKX` | "okx" | OKX exchange |

### Domain Types

#### PriceLevel

| Field | Type | Description |
|-------|------|-------------|
| `price` | `f64` | Price at this level |
| `quantity` | `f64` | Available quantity |
| `exchange` | `Exchange` | Source exchange |
| `timestamp` | `DateTime<Utc>` | Time of last update |

#### Bid

| Field | Type | Description |
|-------|------|-------------|
| `price` | `f64` | Bid price |
| `quantity` | `f64` | Bid quantity |
| `exchange` | `Exchange` | Source exchange |
| `timestamp` | `DateTime<Utc>` | Time of bid |

**Ordering**: Higher prices are better (descending order)

#### Ask

| Field | Type | Description |
|-------|------|-------------|
| `price` | `f64` | Ask price |
| `quantity` | `f64` | Ask quantity |
| `exchange` | `Exchange` | Source exchange |
| `timestamp` | `DateTime<Utc>` | Time of ask |

**Ordering**: Lower prices are better (ascending order)

#### Summary

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Trading symbol (e.g., "BTCUSDT") |
| `spread` | `f64` | Best ask - best bid |
| `bids` | `Vec<PriceLevel>` | Bid levels (sorted by price desc) |
| `asks` | `Vec<PriceLevel>` | Ask levels (sorted by price asc) |
| `timestamp` | `DateTime<Utc>` | Time of summary generation |

#### TradingPair

| Field | Type | Description |
|-------|------|-------------|
| `base` | `String` | Base currency (e.g., "BTC") |
| `quote` | `String` | Quote currency (e.g., "USDT") |

**Parsing Rules**: 
- Format: "BASE/QUOTE" (e.g., "BTC/USDT")
- Case insensitive input, normalized to uppercase
- Must contain exactly one "/" separator

#### PriceLevelUpdate

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Unique update identifier |
| `symbol` | `String` | Trading symbol |
| `exchange` | `Exchange` | Source exchange |
| `bids` | `Vec<Bid>` | Updated bid levels |
| `asks` | `Vec<Ask>` | Updated ask levels |
| `timestamp` | `DateTime<Utc>` | Time of update |

### Additional Types

#### ArbitrageOpportunity

| Field | Type | Description |
|-------|------|-------------|
| `buy_exchange` | `Exchange` | Exchange to buy from |
| `sell_exchange` | `Exchange` | Exchange to sell to |
| `symbol` | `String` | Trading symbol |
| `buy_price` | `f64` | Buy price |
| `sell_price` | `f64` | Sell price |
| `profit_percentage` | `f64` | Profit percentage |
| `volume` | `f64` | Maximum volume |
| `timestamp` | `DateTime<Utc>` | Time of opportunity |

#### HealthStatus

| Field | Type | Description |
|-------|------|-------------|
| `exchange` | `Exchange` | Exchange being monitored |
| `is_healthy` | `bool` | Health status |
| `last_update` | `DateTime<Utc>` | Last update time |
| `error_message` | `Option<String>` | Error message if unhealthy |

#### Metrics

| Field | Type | Description |
|-------|------|-------------|
| `exchange` | `Exchange` | Exchange being monitored |
| `symbol` | `String` | Trading symbol |
| `updates_per_second` | `f64` | Update rate |
| `latency_ms` | `f64` | Average latency |
| `error_count` | `u64` | Error count |
| `last_update` | `DateTime<Utc>` | Last update time |

### Trait Implementations

| Type | Traits | Notes |
|------|--------|-------|
| `Exchange` | `Debug, Clone, PartialEq, Eq, Hash, Display, FromStr` | String conversion support |
| `TradingPair` | `Debug, Clone, PartialEq, Eq, Hash, Display, FromStr` | String conversion support |
| `Bid` | `Debug, Clone, PartialEq, Eq, PartialOrd, Ord` | Custom ordering by price |
| `Ask` | `Debug, Clone, PartialEq, Eq, PartialOrd, Ord` | Custom ordering by price |
| `PriceLevel` | `Debug, Clone, PartialEq` | Basic comparison |
| `Summary` | `Debug, Clone` | Basic traits |

## API Reference

For detailed descriptions of each function and data structure, please refer to the Rust documentation. Cross-link to:
- [Aggregator Documentation](aggregator.md)
- [Error Documentation](error.md)
- [Config Documentation](config.md)

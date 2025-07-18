use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// The `#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]`
/// attribute on the `Exchange` enum in Rust is implementing several traits and functionalities for the
/// enum automatically. Here's what each trait does:
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Exchange {
    Binance,
    Bitstamp,
    Bybit,
    Kraken,
    Coinbase,
    CryptoDotCom,
    OKX,
}

/// The `impl Exchange { ... }` block with the `all()` function is defining a method associated with the
/// `Exchange` enum in Rust.
impl Exchange {
    pub fn all() -> Vec<Exchange> {
        vec![
            Exchange::Binance,
            Exchange::Bitstamp,
            Exchange::Bybit,
            Exchange::Kraken,
            Exchange::Coinbase,
            Exchange::CryptoDotCom,
            Exchange::OKX,
        ]
    }
}

/// The `impl fmt::Display for Exchange { ... }` block in Rust is implementing the `fmt::Display` trait
/// for the `Exchange` enum. This trait allows instances of the `Exchange` enum to be formatted as
/// strings when using formatting macros like `println!` or `format!`.
impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Exchange::Binance => "binance",
            Exchange::Bitstamp => "bitstamp",
            Exchange::Bybit => "bybit",
            Exchange::Kraken => "kraken",
            Exchange::Coinbase => "coinbase",
            Exchange::CryptoDotCom => "crypto_dot_com",
            Exchange::OKX => "okx",
        };
        write!(f, "{}", name)
    }
}

/// The `impl FromStr for Exchange` block in Rust is implementing the `FromStr` trait for the `Exchange`
/// enum. This trait allows a string to be parsed into an `Exchange` enum variant.
impl FromStr for Exchange {
    type Err = crate::AggregatorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "binance" => Ok(Exchange::Binance),
            "bitstamp" => Ok(Exchange::Bitstamp),
            "bybit" => Ok(Exchange::Bybit),
            "kraken" => Ok(Exchange::Kraken),
            "coinbase" => Ok(Exchange::Coinbase),
            "crypto_dot_com" => Ok(Exchange::CryptoDotCom),
            "okx" => Ok(Exchange::OKX),
            _ => Err(crate::AggregatorError::Parsing {
                message: format!("Unknown exchange: {}", s),
                data_type: "Exchange".to_string(),
            }),
        }
    }
}

/// The `PriceLevel` struct represents a price level with associated quantity, exchange, and timestamp
/// in Rust.
///
/// Properties:
///
/// * `price`: The `price` property in the `PriceLevel` struct represents the price of a financial
/// instrument or asset. It is of type `f64`, which is a 64-bit floating-point number in Rust.
/// * `quantity`: The `quantity` property in the `PriceLevel` struct represents the amount of a
/// particular asset available at a specific price level on an exchange. It indicates the volume of the
/// asset that can be bought or sold at that price.
/// * `exchange`: The `exchange` property in the `PriceLevel` struct represents the exchange where the
/// price level data is sourced from.
/// * `timestamp`: The `timestamp` property in the `PriceLevel` struct represents the date and time when
/// the price level data was recorded. It is of type `DateTime<Utc>`, which is a datetime type provided
/// by the `chrono` crate that represents a datetime in the UTC timezone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
    pub exchange: Exchange,
    pub timestamp: DateTime<Utc>,
}

/// The `Bid` struct represents a bid with price, quantity, exchange, and timestamp information in Rust.
///
/// Properties:
///
/// * `price`: The `price` property in the `Bid` struct represents the price at which a bid is placed in
/// a trading scenario. It is of type `f64`, which is a 64-bit floating-point number in Rust.
/// * `quantity`: The `quantity` property in the `Bid` struct represents the amount of the asset that
/// the bid is placed for. It indicates how much of the asset the bidder is willing to buy at the
/// specified price.
/// * `exchange`: The `exchange` property in the `Bid` struct represents the exchange where the bid was
/// placed. It seems like `Exchange` is a custom type that is used to specify different exchanges.
/// * `timestamp`: The `timestamp` property in the `Bid` struct represents the date and time at which
/// the bid was placed. It is of type `DateTime<Utc>`, which is a datetime type provided by the `chrono`
/// crate that represents a specific point in time with timezone information (in this case,
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    pub price: f64,
    pub quantity: f64,
    pub exchange: Exchange,
    pub timestamp: DateTime<Utc>,
}

/// The `impl PartialOrd for Bid { ... }` block is implementing the `PartialOrd` trait for the `Bid`
/// struct in Rust. This trait allows instances of the `Bid` struct to be compared in a partial ordering
/// context, meaning that not all pairs of elements need to have a defined ordering relationship.
impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Higher price is better for bids
        other.price.partial_cmp(&self.price)
    }
}

/// The `impl Ord for Bid { ... }` block in Rust is implementing the `Ord` trait for the `Bid` struct.
/// The `Ord` trait is used for types that have a total ordering, meaning that all instances of the type
/// can be compared and ordered relative to each other.
impl Ord for Bid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// The `impl Eq for Bid {}` block is implementing the `Eq` trait for the `Bid` struct in Rust.
impl Eq for Bid {}

/// The `impl Default for Bid { ... }` block in Rust is implementing the `Default` trait for the `Bid`
/// struct. This trait allows you to define a default value for instances of the `Bid` struct when no
/// initial values are provided.
impl Default for Bid {
    fn default() -> Self {
        Bid {
            price: 0.0,
            quantity: 0.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        }
    }
}

/// Represents an ask order in the order book, containing price, quantity, exchange, and timestamp.
///
/// # Fields
/// - `price`: The price at which the ask is placed.
/// - `quantity`: The quantity available at the specified price.
/// - `exchange`: The exchange where the ask originates.
/// - `timestamp`: The time when the ask was recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ask {
    pub price: f64,
    pub quantity: f64,
    pub exchange: Exchange,
    pub timestamp: DateTime<Utc>,
}

/// Implements partial ordering for the `Ask` type based on price.
///
/// Lower prices are considered better for asks, so this implementation
/// compares the `price` fields of two `Ask` instances using their
/// `partial_cmp` method.
///
/// # Arguments
///
/// * `other` - Another `Ask` instance to compare with.
///
/// # Returns
///
/// An `Option<std::cmp::Ordering>` indicating the ordering between
/// `self` and `other` based on their prices.
impl PartialOrd for Ask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Lower price is better for asks
        self.price.partial_cmp(&other.price)
    }
}

/// Implements the `Ord` trait for the `Ask` type, allowing for total ordering comparisons.
///
/// This implementation delegates to the `partial_cmp` method and returns `Ordering::Equal`
/// if the comparison is not possible (i.e., if either value is NaN or otherwise not comparable).
/// This is useful for sorting or ordering collections of `Ask` values.
///
/// # Panics
///
/// This implementation does not panic, but it may return `Ordering::Equal` for values that
/// are not strictly equal if `partial_cmp` returns `None`.
impl Ord for Ask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl Eq for Ask {}

/// Provides a default implementation for the `Ask` struct.
///
/// The default `Ask` has:
/// - `price` set to the maximum possible `f64` value,
/// - `quantity` set to `0.0`,
/// - `exchange` set to `Exchange::Binance`,
/// - `timestamp` set to the current UTC time.
///
/// This is useful for initializing an `Ask` with placeholder or sentinel values.
impl Default for Ask {
    fn default() -> Self {
        Ask {
            price: f64::MAX,
            quantity: 0.0,
            exchange: Exchange::Binance,
            timestamp: Utc::now(),
        }
    }
}

/// Represents an update to the price levels for a specific trading symbol on a given exchange.
///
/// This struct contains the latest bid and ask levels, along with metadata such as the update's unique identifier,
/// the symbol being updated, the exchange, and the timestamp of the update.
///
/// # Fields
/// - `id`: Unique identifier for this price level update.
/// - `symbol`: The trading symbol (e.g., "BTCUSD") for which the price levels are updated.
/// - `exchange`: The exchange where the price levels are sourced from.
/// - `bids`: A vector of bid levels, representing buy orders.
/// - `asks`: A vector of ask levels, representing sell orders.
/// - `timestamp`: The time at which this update was generated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevelUpdate {
    pub id: Uuid,
    pub symbol: String,
    pub exchange: Exchange,
    pub bids: Vec<Bid>,
    pub asks: Vec<Ask>,
    pub timestamp: DateTime<Utc>,
}

/// Represents a summary of market data for a specific trading symbol.
///
/// # Fields
/// - `symbol`: The trading symbol (e.g., "BTCUSD") associated with this summary.
/// - `spread`: The difference between the best ask and best bid prices.
/// - `bids`: A list of bid price levels, typically sorted by price descending.
/// - `asks`: A list of ask price levels, typically sorted by price ascending.
/// - `timestamp`: The UTC timestamp indicating when this summary was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub symbol: String,
    pub spread: f64,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Represents a trading pair consisting of a base and a quote asset.
///
/// # Fields
/// - `base`: The symbol of the base asset (e.g., "BTC").
/// - `quote`: The symbol of the quote asset (e.g., "USD").
///
/// # Example
/// let pair = TradingPair {
///     base: "BTC".to_string(),
///     quote: "USD".to_string(),
/// };
pub struct TradingPair {
    pub base: String,
    pub quote: String,
}

/// Creates a new `TradingPair` instance with the given base and quote currencies.
///
/// # Arguments
///
/// * `base` - A string slice that holds the base currency symbol.
/// * `quote` - A string slice that holds the quote currency symbol.
///
/// # Returns
///
/// A `TradingPair` with both `base` and `quote` converted to uppercase.
/// let pair = TradingPair::new("btc", "usd");
/// assert_eq!(pair.base, "BTC");
/// assert_eq!(pair.quote, "USD");
impl TradingPair {
    pub fn new(base: &str, quote: &str) -> Self {
        Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }
}

/// Implements the `fmt::Display` trait for the `TradingPair` struct,
/// allowing it to be formatted as a string in the form "BASE/QUOTE".
/// This enables easy and human-readable printing of trading pairs,
/// such as "BTC/USD" or "ETH/EUR".
impl fmt::Display for TradingPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

/// Implements the `FromStr` trait for the `TradingPair` type, allowing it to be created from a string representation.
///
/// # Format
/// Expects the input string to be in the format `"BASE/QUOTE"`, where `BASE` and `QUOTE` are the trading pair symbols.
///
/// # Errors
/// Returns an `AggregatorError::Parsing` if the input string does not contain exactly one '/' separator or does not conform to the expected format.
///
/// # Examples
/// use std::str::FromStr;
/// let pair = TradingPair::from_str("BTC/USDT");
/// assert!(pair.is_ok());
impl FromStr for TradingPair {
    type Err = crate::AggregatorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(crate::AggregatorError::Parsing {
                message: format!("Invalid trading pair format: {}", s),
                data_type: "TradingPair".to_string(),
            });
        }
        Ok(TradingPair::new(parts[0], parts[1]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketType {
    Spot,
    Futures,
    Options,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookDepth {
    pub levels: usize,
    pub market_type: MarketType,
}

impl Default for OrderBookDepth {
    fn default() -> Self {
        Self {
            levels: 20,
            market_type: MarketType::Spot,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub buy_exchange: Exchange,
    pub sell_exchange: Exchange,
    pub symbol: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub exchange: Exchange,
    pub is_healthy: bool,
    pub last_update: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub exchange: Exchange,
    pub symbol: String,
    pub updates_per_second: f64,
    pub latency_ms: f64,
    pub error_count: u64,
    pub last_update: DateTime<Utc>,
}

use crate::types::{Exchange, MarketType, TradingPair};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// The `Config` struct in Rust contains configurations for exchanges, trading pairs, order book,
/// server, logging, and metrics.
///
/// Properties:
///
/// * `exchanges`: The `exchanges` property in the `Config` struct is a HashMap that maps `Exchange`
/// enum values to `ExchangeConfig` struct values. This allows you to store configurations for different
/// exchanges in a key-value pair format.
/// * `trading_pairs`: The `trading_pairs` property in the `Config` struct is a vector of `TradingPair`
/// instances. It likely represents a collection of trading pairs that are supported or configured
/// within the application. Each `TradingPair` instance may contain information about a specific pair of
/// assets that can be traded on
/// * `orderbook`: The `orderbook` property in the `Config` struct represents the configuration for the
/// order book. It likely contains settings and parameters related to how the order book is managed and
/// displayed within the application. This could include things like order book depth, update frequency,
/// storage options, and any other relevant configurations
/// * `server`: The `server` property in the `Config` struct represents the configuration for the server
/// settings. It likely includes details such as the server's host, port, security settings, and any
/// other configurations related to running the server for your application.
/// * `logging`: The `logging` property in the `Config` struct represents the configuration settings for
/// logging in the application. It likely includes details such as log levels, log file paths, log
/// rotation settings, and any other configurations related to logging messages and events within the
/// application.
/// * `metrics`: The `metrics` property in the `Config` struct represents the configuration for metrics
/// collection and monitoring in the application. This configuration likely includes settings related to
/// collecting and reporting metrics such as performance metrics, system health metrics, and other
/// relevant data for monitoring the application's behavior and performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub exchanges: HashMap<Exchange, ExchangeConfig>,
    pub trading_pairs: Vec<TradingPair>,
    pub orderbook: OrderBookConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
}

/// The `ExchangeConfig` struct represents configuration settings for an exchange, including API key,
/// API secret, and other parameters.
///
/// Properties:
///
/// * `enabled`: This `ExchangeConfig` struct represents the configuration settings for an exchange.
/// Here's a brief description of the properties:
/// * `api_key`: The `api_key` property in the `ExchangeConfig` struct is of type `Option<String>`,
/// which means it can either contain a `String` value or be `None`. This is commonly used in Rust to
/// represent optional values.
/// * `api_secret`: The `api_secret` property in the `ExchangeConfig` struct is of type
/// `Option<String>`. This means that it can either contain a `String` value or be `None`. It is
/// commonly used for storing sensitive information such as API secrets or keys, providing flexibility
/// in handling optional values.
/// * `passphrase`: The `passphrase` field in the `ExchangeConfig` struct is of type `Option<String>`.
/// This means that it can either contain a `String` value or be `None`. It is commonly used in API
/// authentication for certain exchanges to provide an additional layer of security along with the API
/// key
/// * `sandbox`: The `sandbox` property in the `ExchangeConfig` struct is a boolean value that indicates
/// whether the exchange is running in a sandbox environment. Sandboxes are often used for testing
/// purposes to simulate real trading conditions without using actual funds.
/// * `rate_limit`: The `rate_limit` property in the `ExchangeConfig` struct likely represents the
/// configuration for rate limiting settings related to API requests made to the exchange. It is of type
/// `RateLimitConfig`, which would contain specific parameters such as the maximum number of requests
/// allowed within a certain time frame, any retry
/// * `websocket`: The `websocket` property in the `ExchangeConfig` struct likely represents the
/// configuration settings for WebSocket communication with the exchange. It is of type
/// `WebSocketConfig`, which may contain details such as the WebSocket endpoint URL, connection
/// settings, authentication details, and any other configurations related to WebSocket communication
/// with the exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub passphrase: Option<String>,
    pub sandbox: bool,
    pub rate_limit: RateLimitConfig,
    pub websocket: WebSocketConfig,
}

/// The `RateLimitConfig` struct in Rust represents configuration settings for rate limiting with fields
/// for requests per second and burst size.
///
/// Properties:
///
/// * `requests_per_second`: The `requests_per_second` property in the `RateLimitConfig` struct
/// represents the maximum number of requests that can be processed per second according to the rate
/// limiting configuration.
/// * `burst_size`: The `burst_size` property in the `RateLimitConfig` struct represents the maximum
/// number of requests that can be allowed to pass through the rate limiter in a short period of time,
/// even if the average rate is lower. It allows for temporary bursts of traffic to be processed without
/// being subject to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

/// The `WebSocketConfig` struct represents configuration settings for a WebSocket connection in Rust.
///
/// Properties:
///
/// * `reconnect_interval`: The `reconnect_interval` property in the `WebSocketConfig` struct represents
/// the time interval (in milliseconds) at which the WebSocket client should attempt to reconnect to the
/// server in case the connection is lost.
/// * `ping_interval`: The `ping_interval` property in the `WebSocketConfig` struct represents the
/// interval at which ping messages are sent to the WebSocket server to keep the connection alive. It is
/// specified in milliseconds.
/// * `max_reconnect_attempts`: The `max_reconnect_attempts` property in the `WebSocketConfig` struct
/// specifies the maximum number of reconnect attempts that will be made by the WebSocket client before
/// giving up. If the connection is lost, the client will try to reconnect up to this specified number
/// of attempts before stopping further reconnect attempts.
/// * `buffer_size`: The `buffer_size` property in the `WebSocketConfig` struct specifies the size of
/// the buffer used for reading and writing data in the WebSocket connection. It determines the maximum
/// amount of data that can be stored in memory before it needs to be processed or cleared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub reconnect_interval: u64,
    pub ping_interval: u64,
    pub max_reconnect_attempts: u32,
    pub buffer_size: usize,
}

/// The `OrderBookConfig` struct in Rust represents configuration settings for an order book.
///
/// Properties:
///
/// * `max_depth`: The `max_depth` property in the `OrderBookConfig` struct specifies the maximum depth
/// or number of price levels to be maintained in the order book. This determines how many buy and sell
/// orders will be stored and displayed in the order book at any given time.
/// * `market_type`: The `market_type` property in the `OrderBookConfig` struct represents the type of
/// market for which the order book configuration is defined. It is of type `MarketType`, which I assume
/// is an enum or a custom type specific to your application. This property helps in specifying the
/// market context for
/// * `update_interval`: The `update_interval` property in the `OrderBookConfig` struct represents the
/// time interval, in milliseconds for example, at which the order book should be updated with new data
/// or changes. This interval determines how frequently the order book will be refreshed with the latest
/// market information.
/// * `cleanup_interval`: The `cleanup_interval` property in the `OrderBookConfig` struct represents the
/// interval (in milliseconds, seconds, etc.) at which the order book should be cleaned up or
/// maintained. This interval determines how often the order book data should be checked for outdated or
/// irrelevant information and cleaned up to ensure the
/// * `implementation`: The `implementation` property in the `OrderBookConfig` struct represents the
/// type of implementation used for the order book. It could be an enum or a specific type that defines
/// how the order book operations are handled internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookConfig {
    pub max_depth: usize,
    pub market_type: MarketType,
    pub update_interval: u64,
    pub cleanup_interval: u64,
    pub implementation: OrderBookImplementation,
}

/// The above Rust code defines an enum `OrderBookImplementation` with four variants: `BTreeSet`,
/// `AvlTree`, `RbTree`, and `HashMap`. This enum can be used to represent different implementations for
/// an order book in a trading system. The enum derives `Debug`, `Clone`, `Serialize`, and `Deserialize`
/// traits, allowing for debugging, cloning, and serialization/deserialization of instances of this
/// enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderBookImplementation {
    BTreeSet,
    AvlTree,
    RbTree,
    HashMap,
}

/// The `ServerConfig` struct contains configurations for gRPC, REST, and WebSocket servers.
///
/// Properties:
///
/// * `grpc`: The `grpc` property in the `ServerConfig` struct represents the configuration for the gRPC
/// server. It likely contains settings such as the host address, port number, SSL configuration, and
/// any other options specific to the gRPC server setup.
/// * `rest`: The `rest` property in the `ServerConfig` struct represents the configuration for a REST
/// server. It likely contains settings such as the port number, host address, SSL configuration,
/// middleware options, and any other parameters needed to configure the REST server.
/// * `websocket`: The `websocket` property in the `ServerConfig` struct represents the configuration
/// settings for a WebSocket server. It likely includes details such as the host, port, protocols, and
/// any additional settings required to set up and configure the WebSocket server for communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub grpc: GrpcConfig,
    pub rest: RestConfig,
    pub websocket: WebSocketServerConfig,
}

/// The `GrpcConfig` struct represents configuration settings for a gRPC connection in Rust.
///
/// Properties:
///
/// * `enabled`: The `enabled` property in the `GrpcConfig` struct is a boolean value that indicates
/// whether the gRPC communication is enabled or not. If `enabled` is `true`, it means that gRPC
/// communication is enabled; if `enabled` is `false`, it means that gRPC communication
/// * `host`: The `host` property in the `GrpcConfig` struct represents the hostname or IP address of
/// the gRPC server that the client will connect to. It is of type `String`.
/// * `port`: The `port` property in the `GrpcConfig` struct represents the port number used for the
/// gRPC communication. It is of type `u16`, which means it can hold values from 0 to 65,535.
/// * `tls`: The `tls` property in the `GrpcConfig` struct is an optional field of type `TlsConfig`.
/// This field allows you to configure Transport Layer Security (TLS) settings for the gRPC connection.
/// If the `tls` field is `Some`, it means that TLS is enabled and
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
}

/// The `RestConfig` struct represents configuration settings for a REST API in Rust.
///
/// Properties:
///
/// * `enabled`: The `enabled` property in the `RestConfig` struct indicates whether the REST service is
/// enabled or not. It is a boolean value (`true` or `false`) that determines if the REST service should
/// be active or inactive.
/// * `host`: The `host` property in the `RestConfig` struct represents the host address where the REST
/// API will be served from. It is of type `String`, which means it should store the hostname or IP
/// address of the server where the REST API will be accessible.
/// * `port`: The `port` property in the `RestConfig` struct represents the port number on which the
/// REST service will be running. It is of type `u16`, which means it can hold values from 0 to 65,535.
/// This port number is used to uniquely identify different network services running
/// * `cors`: The `cors` property in the `RestConfig` struct represents the CORS (Cross-Origin Resource
/// Sharing) configuration for the REST API. It likely contains settings related to allowing or
/// restricting cross-origin requests from web browsers, such as allowed origins, methods, headers, and
/// credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub cors: CorsConfig,
}

/// The `WebSocketServerConfig` struct represents configuration settings for a WebSocket server in Rust.
///
/// Properties:
///
/// * `enabled`: The `enabled` property in the `WebSocketServerConfig` struct is a boolean value that
/// indicates whether the WebSocket server is enabled or not. If `enabled` is `true`, the server is
/// active and ready to accept connections. If `enabled` is `false`, the server is not running and
/// * `host`: The `host` property in the `WebSocketServerConfig` struct represents the host address to
/// which the WebSocket server will bind. This is the network address where the server will listen for
/// incoming WebSocket connections. It is typically an IP address or a domain name.
/// * `port`: The `port` property in the `WebSocketServerConfig` struct represents the port number on
/// which the WebSocket server will listen for incoming connections. It is of type `u16`, which means it
/// can hold values from 0 to 65535.
/// * `max_connections`: The `max_connections` property in the `WebSocketServerConfig` struct represents
/// the maximum number of connections that the WebSocket server can handle simultaneously. This value
/// determines the capacity of the server to accept incoming connections from clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

/// The `TlsConfig` struct in Rust represents configuration settings for TLS with fields for certificate
/// and key file paths.
///
/// Properties:
///
/// * `cert_path`: The `cert_path` property in the `TlsConfig` struct represents the file path to the
/// TLS/SSL certificate file. This file contains the public key of the server or client, which is used
/// in the SSL/TLS handshake process to establish a secure connection.
/// * `key_path`: The `key_path` property in the `TlsConfig` struct represents the file path where the
/// private key for the TLS configuration is stored. This private key is used for encrypting and
/// decrypting data during secure communication over TLS (Transport Layer Security).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

/// The `CorsConfig` struct in Rust represents configuration settings for Cross-Origin Resource Sharing
/// (CORS).
///
/// Properties:
///
/// * `allowed_origins`: The `allowed_origins` property in the `CorsConfig` struct specifies a list of
/// origins that are allowed to make cross-origin requests to the server. These origins can be domains,
/// subdomains, or even specific URLs that are permitted to access the server's resources.
/// * `allowed_methods`: The `allowed_methods` property in the `CorsConfig` struct represents a list of
/// HTTP methods that are allowed for cross-origin requests. These methods typically include common HTTP
/// methods such as GET, POST, PUT, DELETE, etc. The `allowed_methods` field in the `CorsConfig` struct
/// is
/// * `allowed_headers`: The `allowed_headers` property in the `CorsConfig` struct represents a list of
/// headers that are allowed in the CORS (Cross-Origin Resource Sharing) requests. These headers are
/// permitted to be included in the requests from the allowed origins specified in the `allowed_origins`
/// property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

/// The `LoggingConfig` struct represents configuration settings for logging in Rust.
///
/// Properties:
///
/// * `level`: The `level` property in the `LoggingConfig` struct represents the logging level, such as
/// "info", "debug", "error", etc., indicating the severity of the log message.
/// * `format`: The `format` property in the `LoggingConfig` struct represents the format in which log
/// messages will be output. This could be a specific pattern or template that defines how log messages
/// should be structured, including placeholders for variables like timestamp, log level, message, etc.
/// * `output`: The `output` property in the `LoggingConfig` struct represents the destination where the
/// log messages will be written to. It could be a file, console, database, etc. The value of this
/// property will specify where the logs will be outputted.
/// * `file_path`: The `file_path` property in the `LoggingConfig` struct is an optional field of type
/// `Option<String>`. This means that it can either contain a `Some` value with a `String` value inside,
/// or it can be `None` if no file path is specified. It is
/// * `max_file_size`: The `max_file_size` property in the `LoggingConfig` struct represents the maximum
/// size in bytes that a log file can reach before it is rotated or a new log file is created. This
/// property is of type `u64`, which means it can hold unsigned 64-bit integers.
/// * `max_files`: The `max_files` property in the `LoggingConfig` struct represents the maximum number
/// of log files that can be created before old log files are rotated or deleted. This property
/// specifies the limit for the number of log files that can be retained for logging purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub file_path: Option<String>,
    pub max_file_size: u64,
    pub max_files: u32,
}

/// The `MetricsConfig` struct in Rust represents configuration settings for metrics, including
/// Prometheus configuration.
///
/// Properties:
///
/// * `enabled`: The `enabled` property in the `MetricsConfig` struct is a boolean field that indicates
/// whether metrics collection is enabled or not. If `enabled` is set to `true`, it means that metrics
/// collection is active, while if it is set to `false`, metrics collection is disabled.
/// * `prometheus`: The `MetricsConfig` struct has two properties:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub prometheus: PrometheusConfig,
}

/// The `PrometheusConfig` struct represents configuration settings for Prometheus monitoring with
/// fields for enabling, host, port, and path.
///
/// Properties:
///
/// * `enabled`: The `enabled` property in the `PrometheusConfig` struct indicates whether the
/// Prometheus monitoring is enabled or not. If `enabled` is `true`, it means that Prometheus monitoring
/// is active; if `enabled` is `false`, it means that Prometheus monitoring is disabled.
/// * `host`: The `host` property in the `PrometheusConfig` struct represents the host address where the
/// Prometheus server is running. This could be an IP address or a domain name.
/// * `port`: The `port` property in the `PrometheusConfig` struct represents the port number used for
/// connecting to the Prometheus server. It is of type `u16`, which means it can hold values from 0 to
/// 65,535.
/// * `path`: The `path` property in the `PrometheusConfig` struct represents the path where the
/// Prometheus metrics will be available. This could be a specific endpoint on your server where
/// Prometheus metrics are exposed, such as `/metrics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub path: String,
}

/// The above Rust code is defining an enum `ConfigError` that represents different types of errors that
/// can occur related to configuration. It has one variant `FileNotFound` which includes a string
/// message indicating the file that was not found. The `#[derive(Error, Debug)]` attribute is used to
/// automatically implement the `Error` trait from the `thiserror` crate, which allows for easy error
/// handling and formatting. The `#[error("File not found: {0}")]` attribute specifies the error message
/// format for the `FileNotFound` variant.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// The above Rust code is implementing the `Default` trait for a struct named `Config`. The `Default`
/// trait provides a way to create an instance of a type with default values.
impl Default for Config {
    fn default() -> Self {
        let mut exchanges = HashMap::new();

        // Add default configurations for supported exchanges
        for exchange in Exchange::all() {
            exchanges.insert(exchange, ExchangeConfig::default());
        }

        Self {
            exchanges,
            trading_pairs: vec![
                TradingPair::new("BTC", "USDT"),
                TradingPair::new("ETH", "USDT"),
                TradingPair::new("BNB", "USDT"),
            ],
            orderbook: OrderBookConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

/// The above code is implementing the `Default` trait for a struct named `ExchangeConfig`. This
/// implementation provides a default configuration for the `ExchangeConfig` struct. The default
/// configuration sets the `enabled` field to `true`, `api_key`, `api_secret`, and `passphrase` fields
/// to `None`, `sandbox` field to `false`, and initializes `rate_limit` and `websocket` fields with
/// their default configurations using `RateLimitConfig::default()` and `WebSocketConfig::default()`
/// respectively.
impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key: None,
            api_secret: None,
            passphrase: None,
            sandbox: false,
            rate_limit: RateLimitConfig::default(),
            websocket: WebSocketConfig::default(),
        }
    }
}

/// The above code is implementing the `Default` trait for the `RateLimitConfig` struct in Rust. It
/// provides a default implementation for the `default()` method, which initializes a `RateLimitConfig`
/// instance with default values for `requests_per_second` and `burst_size` (10 and 20 respectively).
/// This allows instances of `RateLimitConfig` to be created with default values using
/// `RateLimitConfig::default()`.
impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 20,
        }
    }
}

/// The above code is implementing the `Default` trait for the `WebSocketConfig` struct in Rust. By
/// implementing the `Default` trait, it provides a default implementation for creating instances of
/// `WebSocketConfig` when no specific values are provided. In this implementation, the default values
/// for `reconnect_interval`, `ping_interval`, `max_reconnect_attempts`, and `buffer_size` are set to
/// 5000, 30000, 10, and 1000 respectively.
impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            reconnect_interval: 5000,
            ping_interval: 30000,
            max_reconnect_attempts: 10,
            buffer_size: 1000,
        }
    }
}

/// The above code is implementing the `Default` trait for the `OrderBookConfig` struct in Rust. It
/// provides a default implementation for the `default()` method, which returns an instance of
/// `OrderBookConfig` with default values for its fields. The default values set in this implementation
/// are `max_depth: 20`, `market_type: MarketType::Spot`, `update_interval: 100`, `cleanup_interval:
/// 60000`, and `implementation: OrderBookImplementation::BTreeSet`. This allows instances of
/// `OrderBookConfig` to be created with these default values if no specific
impl Default for OrderBookConfig {
    fn default() -> Self {
        Self {
            max_depth: 20,
            market_type: MarketType::Spot,
            update_interval: 100,
            cleanup_interval: 60000,
            implementation: OrderBookImplementation::BTreeSet,
        }
    }
}

/// The above code is implementing the `Default` trait for the `ServerConfig` struct in Rust. This
/// allows instances of `ServerConfig` to be created with default values using the `default()` method.
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            grpc: GrpcConfig::default(),
            rest: RestConfig::default(),
            websocket: WebSocketServerConfig::default(),
        }
    }
}

/// The above code is implementing the `Default` trait for a struct named `GrpcConfig`. This allows
/// instances of `GrpcConfig` to be created with default values using the `default()` method. In this
/// implementation, the default values set for `GrpcConfig` are `enabled` as `true`, `host` as
/// `"0.0.0.0"`, `port` as `50051`, and `tls` as `None`.
impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 50051,
            tls: None,
        }
    }
}

/// The above code is implementing the `Default` trait for a struct named `RestConfig`. This allows
/// instances of `RestConfig` to be created with default values using the `default()` method. In this
/// implementation, the default values set for `RestConfig` include `enabled` as `true`, `host` as
/// `"0.0.0.0"`, `port` as `8080`, and `cors` as the default value of `CorsConfig`.
impl Default for RestConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 8080,
            cors: CorsConfig::default(),
        }
    }
}

/// The above code is implementing the `Default` trait for the `WebSocketServerConfig` struct in Rust.
/// This allows the struct to have a default implementation when no values are provided during
/// initialization. The `default()` function sets default values for the `WebSocketServerConfig` struct,
/// such as `enabled` being `true`, `host` being `"0.0.0.0"`, `port` being `8081`, and `max_connections`
/// being `1000`.
impl Default for WebSocketServerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 8081,
            max_connections: 1000,
        }
    }
}

/// The above code is implementing the `Default` trait for a struct named `CorsConfig`. By implementing
/// the `Default` trait, the code provides a default implementation for the `CorsConfig` struct. The
/// `default()` function specifies the default values for the fields of the `CorsConfig` struct, setting
/// `allowed_origins` to contain a single element with value `"*"`, `allowed_methods` to contain two
/// elements with values `"GET"` and `"POST"`, and `allowed_headers` to contain a single element with
/// value `"*"`.
impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["*".to_string()],
        }
    }
}

/// The above code is implementing the `Default` trait for a struct named `LoggingConfig`. This allows
/// instances of `LoggingConfig` to be created with default values without explicitly specifying all
/// fields. The `default` function sets default values for the fields `level`, `format`, `output`,
/// `file_path`, `max_file_size`, and `max_files` of the `LoggingConfig` struct.
impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
            file_path: None,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
        }
    }
}
/// The above code is implementing the `Default` trait for a struct named `MetricsConfig`. This
/// implementation provides a default configuration for `MetricsConfig` instances. The `default()`
/// function initializes a `MetricsConfig` instance with the `enabled` field set to `true` and the
/// `prometheus` field initialized with the default configuration provided by
/// `PrometheusConfig::default()`.

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prometheus: PrometheusConfig::default(),
        }
    }
}

/// The above code is implementing the `Default` trait for the `PrometheusConfig` struct in Rust. This
/// allows the struct to have a default implementation when no values are provided. The default
/// implementation sets the `enabled` field to `true`, `host` field to "0.0.0.0", `port` field to 9090,
/// and `path` field to "/metrics".
impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 9090,
            path: "/metrics".to_string(),
        }
    }
}

impl Config {
    /// The function `from_file` reads a JSON file, parses its content into a `Config` struct using serde,
    /// and returns a result.
    ///
    /// Arguments:
    ///
    /// * `path`: The `path` parameter in the `from_file` function is a reference to a string that
    /// represents the file path from which the configuration data will be read.
    ///
    /// Returns:
    ///
    /// The `from_file` function returns a `Result` containing either a `Config` instance if the file is
    /// successfully read and parsed, or an error of type `crate::AggregatorError` if there are any issues
    /// during the process.
    pub fn from_file(path: &str) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::AggregatorError::parsing(
                "Config",
                format!("Failed to write config file: {}", e).as_str(),
            )
        })?;

        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// The function `to_file` serializes a struct to JSON and writes it to a file in Rust.
    ///
    /// Arguments:
    ///
    /// * `path`: The `path` parameter in the `to_file` function represents the file path where the JSON
    /// content will be written. It is a reference to a string (`&str`) that specifies the location where
    /// the content will be saved.
    ///
    /// Returns:
    ///
    /// The `to_file` function returns a `Result` with the success type `()` (unit) and an error type
    /// defined in the `crate` module.
    pub fn to_file(&self, path: &str) -> crate::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content).map_err(|e| {
            crate::AggregatorError::parsing(
                "Config",
                format!("Failed to write config file: {}", e).as_str(),
            )
        })?;
        Ok(())
    }

    /// The `enabled_exchanges` function returns a vector of enabled exchanges based on a given
    /// configuration.
    ///
    /// Returns:
    ///
    /// A vector of enabled exchanges is being returned.
    pub fn enabled_exchanges(&self) -> Vec<Exchange> {
        self.exchanges
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(exchange, _)| exchange.clone())
            .collect()
    }
}

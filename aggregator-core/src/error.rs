use crate::config;
use std::io;
use thiserror::Error;

/// The main error type for the aggregator application.
#[derive(Error, Debug)]
pub enum AggregatorError {
    /// Represents an error originating from the configuration module.
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// Represents an error during serialization or deserialization.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Represents an error when sending a message through a channel.
    #[error("Channel send error: {message}")]
    ChannelSend { message: String },

    /// Represents an error when receiving a message from a channel.
    #[error("Channel receive error: {message}")]
    ChannelReceive { message: String },

    /// Represents a generic error from a specific exchange.
    #[error("Exchange '{exchange}' error: {message}")]
    ExchangeError { exchange: String, message: String },

    /// Represents an error related to order book operations.
    #[error("OrderBook error: {operation} failed - {message}")]
    OrderBookError { operation: String, message: String },

    /// Represents a network-related error.
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Represents an error from an HTTP request.
    #[error("HTTP request failed: {status_code}- {message}")]
    HttpRequestError { status_code: u16, message: String },

    /// Represents an error from a WebSocket connection.
    #[error("WebSocket error: {message}")]
    WebSocketError { message: String },

    /// Represents a data parsing error.
    #[error("Parsing error for {data_type}: {message}")]
    Parsing { data_type: String, message: String },

    /// Represents a timeout error.
    #[error("Timeout error: {operation} timed out after {duration_ms}ms")]
    Timeout { operation: String, duration_ms: u64 },

    /// Represents an I/O error.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Represents a URL parsing error.
    #[error("URL parsing error: {0}")]
    UrlParsing(#[from] url::ParseError),

    /// Represents a UUID parsing error.
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    /// Represents a data validation error.
    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    /// Represents a database-related error.
    #[error("Database error: {operation} failed - {message}")]
    Database { operation: String, message: String },

    /// Represents an authentication error.
    #[error("Authentication error: {message}")]
    Authentication { message: String },

    /// Represents a rate limit exceeded error.
    #[error("Rate limit exceeded for {resource}: {message}")]
    RateLimit { resource: String, message: String },

    /// Represents an internal server error.
    #[error("Internal server error: {message}")]
    Internal { message: String },

    /// Represents a resource not found error.
    #[error("Not found: {resource} with id '{id}'")]
    NotFound { resource: String, id: String },

    /// Represents a resource that already exists.
    #[error("Already exists: {resource} with id '{id}'")]
    AlreadyExists { resource: String, id: String },

    /// Represents an error during application shutdown.
    #[error("Shutdown error: {message}")]
    Shutdown { message: String },
}

/// A specialized `Result` type for the aggregator application.
pub type Result<T> = std::result::Result<T, AggregatorError>;

/// Represents an error related to HTTP requests.
#[derive(Error, Debug)]
pub enum HttpError {
    /// Represents a failed HTTP request.
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// Represents an invalid response from an HTTP request.
    #[error("Invalid response: {message}")]
    InvalidResponse { message: String },

    /// Represents an unexpected status code from an HTTP response.
    #[error("Status code {code}: {message}")]
    StatusCode { code: u16, message: String },
}

/// Represents an error related to WebSocket connections.
#[derive(Error, Debug)]
pub enum WebSocketError {
    /// Represents a failed WebSocket connection.
    #[error("Connection failed: {message}")]
    Connection { message: String },

    /// Represents a WebSocket protocol error.
    #[error("Protocol error: {message}")]
    Protocol { message: String },

    /// Represents a failed message send over a WebSocket connection.
    #[error("Message send failed: {message}")]
    Send { message: String },

    /// Represents a failed message receive over a WebSocket connection.
    #[error("Message receive failed: {message}")]
    Receive { message: String },

    /// Represents a failed WebSocket handshake.
    #[error("Handshake failed: {message}")]
    Handshake { message: String },
}

/// Represents an error specific to an exchange.
#[derive(Error, Debug)]
pub enum ExchangeError {
    /// Represents a failed connection to an exchange.
    #[error("Connection failed to {exchange}: {message}")]
    Connection { exchange: String, message: String },

    /// Represents a failed authentication with an exchange.
    #[error("Authentication failed for {exchange}: {message}")]
    Authentication { exchange: String, message: String },

    /// Represents a rate limit exceeded error from an exchange.
    #[error("Rate limit exceeded for {exchange}: {message}")]
    RateLimit { exchange: String, message: String },

    /// Represents an invalid symbol for an exchange.
    #[error("Invalid symbol '{symbol}' for {exchange}: {message}")]
    InvalidSymbol {
        exchange: String,
        symbol: String,
        message: String,
    },

    /// Represents an API error from an exchange.
    #[error("API error from {exchange}: {code} - {message}")]
    ApiError {
        exchange: String,
        code: String,
        message: String,
    },
}

/// This `impl From<HttpError> for AggregatorError` block is implementing a conversion from the
/// `HttpError` enum to the `AggregatorError` enum. This allows for seamless conversion between these
/// two error types.
impl From<HttpError> for AggregatorError {
    fn from(err: HttpError) -> Self {
        match err {
            HttpError::Request(e) => AggregatorError::NetworkError {
                message: e.to_string(),
            },
            HttpError::InvalidResponse { message } => AggregatorError::Parsing {
                data_type: "HTTP Response".to_string(),
                message,
            },
            HttpError::StatusCode { code, message } => AggregatorError::HttpRequestError {
                status_code: code,
                message,
            },
        }
    }
}

/// This `impl From<WebSocketError> for AggregatorError` block is implementing a conversion from the
/// `WebSocketError` enum to the `AggregatorError` enum. This allows for seamless conversion between
/// errors of these two types.
impl From<WebSocketError> for AggregatorError {
    fn from(err: WebSocketError) -> Self {
        AggregatorError::WebSocketError {
            message: err.to_string(),
        }
    }
}

/// This `impl From<ExchangeError> for AggregatorError` block is implementing a conversion from the
/// `ExchangeError` enum to the `AggregatorError` enum. It defines how instances of `ExchangeError` can
/// be converted into instances of `AggregatorError`.
impl From<ExchangeError> for AggregatorError {
    fn from(err: ExchangeError) -> Self {
        match err {
            ExchangeError::Connection { exchange, message } => {
                AggregatorError::ExchangeError { exchange, message }
            }
            ExchangeError::Authentication { exchange, message } => {
                AggregatorError::Authentication {
                    message: format!("{}: {}", exchange, message),
                }
            }
            ExchangeError::RateLimit { exchange, message } => AggregatorError::RateLimit {
                resource: exchange,
                message,
            },
            ExchangeError::InvalidSymbol {
                exchange,
                symbol,
                message,
            } => AggregatorError::Validation {
                field: format!("{} symbol", exchange),
                message: format!("{}: {}", symbol, message),
            },
            ExchangeError::ApiError {
                exchange,
                code,
                message,
            } => AggregatorError::ExchangeError {
                exchange,
                message: format!("{}: {}", code, message),
            },
        }
    }
}

/// This implementation block is defining how a specific type of error,
/// `tokio::sync::broadcast::error::SendError<crate::Summary>`, can be converted into an
/// `AggregatorError`.
impl From<tokio::sync::broadcast::error::SendError<crate::Summary>> for AggregatorError {
    fn from(e: tokio::sync::broadcast::error::SendError<crate::Summary>) -> Self {
        AggregatorError::ChannelSend {
            message: e.to_string(),
        }
    }
}

/// This implementation block is defining how errors of type `tokio::sync::mpsc::error::SendError<T>`
/// can be converted into an `AggregatorError`. When a `tokio::sync::mpsc::error::SendError<T>` occurs,
/// this implementation provides a way to convert it into an `AggregatorError` by creating a
/// `ChannelSend` variant with a message describing the error using the `to_string()` method of the
/// original error `e`. This allows for seamless conversion and handling of errors from the `tokio`
/// multi-producer, single-consumer channel in the context of the `AggregatorError` enum.
impl<T> From<tokio::sync::mpsc::error::SendError<T>> for AggregatorError {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        AggregatorError::ChannelSend {
            message: e.to_string(),
        }
    }
}

/// This implementation block is defining how errors of type `tokio::sync::broadcast::error::RecvError`
/// can be converted into an `AggregatorError`. When a `tokio::sync::broadcast::error::RecvError`
/// occurs, this implementation provides a way to convert it into an `AggregatorError` by creating a
/// `ChannelReceive` variant with a message describing the error using the `to_string()` method of the
/// original error `e`. This allows for seamless conversion and handling of errors from the `tokio`
/// broadcast channel in the context of the `AggregatorError` enum.
impl From<tokio::sync::broadcast::error::RecvError> for AggregatorError {
    fn from(e: tokio::sync::broadcast::error::RecvError) -> Self {
        AggregatorError::ChannelReceive {
            message: e.to_string(),
        }
    }
}

/// This implementation block is defining how errors of type `tokio::time::error::Elapsed` can be
/// converted into an `AggregatorError`. When a `tokio::time::error::Elapsed` error occurs, this
/// implementation provides a way to convert it into an `AggregatorError` by creating a `Timeout`
/// variant with a default operation name of "Unknown" and a duration of 0 milliseconds. This allows for
/// handling timeout errors from the Tokio time library within the context of the `AggregatorError`
/// enum.
impl From<tokio::time::error::Elapsed> for AggregatorError {
    fn from(e: tokio::time::error::Elapsed) -> Self {
        AggregatorError::Timeout {
            operation: "Unknown".to_string(),
            duration_ms: 0,
        }
    }
}

/// The above code is implementing a conversion from a `reqwest::Error` to an `AggregatorError`. It
/// defines a `From` trait implementation for converting a `reqwest::Error` into an `AggregatorError`.
/// Inside the implementation, it creates a new `AggregatorError` with a variant `Network` and sets the
/// error message to the string representation of the original `reqwest::Error`.
impl From<reqwest::Error> for AggregatorError {
    fn from(e: reqwest::Error) -> Self {
        AggregatorError::NetworkError {
            message: e.to_string(),
        }
    }
}

/// The above Rust code snippet is implementing a conversion from a `tungstenite::Error` to an
/// `AggregatorError`. It defines an implementation of the `From` trait for this conversion. When a
/// `tungstenite::Error` is encountered, it will be converted into an `AggregatorError` with a variant
/// `WebSocket`, containing the error message from the original `tungstenite::Error` converted to a
/// string.
impl From<tungstenite::Error> for AggregatorError {
    fn from(e: tungstenite::Error) -> Self {
        AggregatorError::WebSocketError {
            message: e.to_string(),
        }
    }
}

impl AggregatorError {
    /// The `parsing` function in Rust creates an `AggregatorError` with a parsing error message.
    ///
    /// Arguments:
    ///
    /// * `data_type`: The `data_type` parameter in the `parsing` function is used to specify the type of
    /// data being parsed. It is expected to be a reference to a string (`AsRef<str>`).
    /// * `message`: The `message` parameter in the `parsing` function is the error message or description
    /// associated with the parsing error that occurred. It is converted to a `String` using
    /// `message.as_ref().to_string()` before being stored in the `AggregatorError::Parsing` variant.
    ///
    /// Returns:
    ///
    /// An AggregatorError enum variant called Parsing is being returned, with the data_type and message
    /// fields set to the string representations of the input data_type and message parameters,
    /// respectively.
    pub fn parsing<T: AsRef<str>>(data_type: T, message: T) -> Self {
        AggregatorError::Parsing {
            data_type: data_type.as_ref().to_string(),
            message: message.as_ref().to_string(),
        }
    }

    /// The function `exchange` creates an `AggregatorError` with an `ExchangeError` variant, using the
    /// provided exchange and message strings.
    ///
    /// Arguments:
    ///
    /// * `exchange`: The `exchange` parameter is of type `T`, which must implement the `AsRef<str>` trait.
    /// This means that `exchange` can be any type that can be converted into a string reference.
    /// * `message`: The `message` parameter in the `exchange` function is of type `T`, which must implement
    /// the `AsRef<str>` trait. This means that `message` can be any type that can be converted into a
    /// string reference.
    ///
    /// Returns:
    ///
    /// An `AggregatorError` enum variant `ExchangeError` is being returned with the exchange and message
    /// converted to strings.
    pub fn exchange<T: AsRef<str>>(exchange: T, message: T) -> Self {
        AggregatorError::ExchangeError {
            exchange: exchange.as_ref().to_string(),
            message: message.as_ref().to_string(),
        }
    }

    /// The function `network` in Rust creates a `NetworkError` variant of the `AggregatorError` enum with a
    /// specified message.
    ///
    /// Arguments:
    ///
    /// * `message`: The `message` parameter in the `network` function is of type `T`, which must implement
    /// the `AsRef<str>` trait. This means that `message` can be any type that can be converted into a
    /// string reference.
    ///
    /// Returns:
    ///
    /// An instance of the `AggregatorError` enum with the variant `NetworkError`, containing the message
    /// converted to a `String`.
    pub fn network<T: AsRef<str>>(message: T) -> Self {
        AggregatorError::NetworkError {
            message: message.as_ref().to_string(),
        }
    }

    /// The `timeout` function in Rust creates an `AggregatorError` with a timeout error message containing
    /// the operation and duration.
    ///
    /// Arguments:
    ///
    /// * `operation`: The `operation` parameter is a generic type `T` that implements the `AsRef<str>`
    /// trait. It is used to describe the operation that timed out.
    /// * `duration_ms`: The `duration_ms` parameter represents the duration in milliseconds for which the
    /// operation is allowed to run before a timeout error is triggered.
    ///
    /// Returns:
    ///
    /// An AggregatorError enum variant Timeout is being returned, with the operation name and duration in
    /// milliseconds provided as parameters.
    pub fn timeout<T: AsRef<str>>(operation: T, duration_ms: u64) -> Self {
        AggregatorError::Timeout {
            operation: operation.as_ref().to_string(),
            duration_ms,
        }
    }

    /// The function `validation` in Rust creates a `AggregatorError` with a validation error message for a
    /// specified field.
    ///
    /// Arguments:
    ///
    /// * `field`: The `field` parameter is used to specify the field that failed validation. It is expected
    /// to implement the `AsRef<str>` trait, allowing it to be converted to a string for error reporting
    /// purposes.
    /// * `message`: The `message` parameter in the `validation` function is used to provide a description
    /// or explanation of the validation error that occurred for a specific field. It is converted to a
    /// string using `message.as_ref().to_string()` before being stored in the `AggregatorError::Validation`
    /// variant.
    ///
    /// Returns:
    ///
    /// An AggregatorError enum variant called Validation is being returned, with the field and message
    /// values converted to strings using the as_ref() method.
    pub fn validation<T: AsRef<str>>(field: T, message: T) -> Self {
        AggregatorError::Validation {
            field: field.as_ref().to_string(),
            message: message.as_ref().to_string(),
        }
    }

    /// The function `not_found` creates a `AggregatorError::NotFound` instance with the provided `resource`
    /// and `id` converted to strings.
    ///
    /// Arguments:
    ///
    /// * `resource`: The `resource` parameter is a reference to a type that can be converted into a string
    /// slice (`&str`).
    /// * `id`: The `id` parameter is a reference to the resource identifier that was not found.
    ///
    /// Returns:
    ///
    /// An `AggregatorError` enum variant `NotFound` is being returned with the `resource` and `id` values
    /// converted to strings.
    pub fn not_found<T: AsRef<str>>(resource: T, id: T) -> Self {
        AggregatorError::NotFound {
            resource: resource.as_ref().to_string(),
            id: id.as_ref().to_string(),
        }
    }

    /// The function `is_recoverable` in Rust checks if an error is recoverable based on specific error
    /// types.
    ///
    /// Returns:
    ///
    /// The `is_recoverable` method returns a boolean value indicating whether the error is recoverable or
    /// not. It returns `true` if the error is of type `NetworkError`, `WebSocketError`, `Timeout`,
    /// `RateLimit`, `ChannelSend`, or `ChannelReceive`. Otherwise, it returns `false`.
    pub fn is_recoverable(&self) -> bool {
        match self {
            AggregatorError::NetworkError { .. } => true,
            AggregatorError::WebSocketError { .. } => true,
            AggregatorError::Timeout { .. } => true,
            AggregatorError::RateLimit { .. } => true,
            AggregatorError::ChannelSend { .. } => true,
            AggregatorError::ChannelReceive { .. } => true,
            _ => false,
        }
    }

    /// The function `category` in Rust returns a string representing the category of an `AggregatorError`
    /// enum variant.
    ///
    /// Returns:
    ///
    /// The `category` method returns a string representing the category of the `AggregatorError` enum
    /// variant that is being matched. The returned string corresponds to different error categories such as
    /// configuration, serialization, channel, exchange, orderbook, network, http, websocket, parsing,
    /// timeout, io, url, uuid, validation, database, authentication, rate_limit, internal, not_found,
    /// already_exists, and
    pub fn category(&self) -> &'static str {
        match self {
            AggregatorError::Config(..) => "configuration",
            AggregatorError::Serialization(..) => "serialization",
            AggregatorError::ChannelSend { .. } => "channel",
            AggregatorError::ChannelReceive { .. } => "channel",
            AggregatorError::ExchangeError { .. } => "exchange",
            AggregatorError::OrderBookError { .. } => "orderbook",
            AggregatorError::NetworkError { .. } => "network",
            AggregatorError::HttpRequestError { .. } => "http",
            AggregatorError::WebSocketError { .. } => "websocket",
            AggregatorError::Parsing { .. } => "parsing",
            AggregatorError::Timeout { .. } => "timeout",
            AggregatorError::Io(..) => "io",
            AggregatorError::UrlParsing(..) => "url",
            AggregatorError::Uuid(..) => "uuid",
            AggregatorError::Validation { .. } => "validation",
            AggregatorError::Database { .. } => "database",
            AggregatorError::Authentication { .. } => "authentication",
            AggregatorError::RateLimit { .. } => "rate_limit",
            AggregatorError::Internal { .. } => "internal",
            AggregatorError::NotFound { .. } => "not_found",
            AggregatorError::AlreadyExists { .. } => "already_exists",
            AggregatorError::Shutdown { .. } => "shutdown",
        }
    }
}

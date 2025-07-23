// // aggregator-core/tests/aggregator-core/error_tests.rs
// // Unit tests for error.rs

// // use aggregator_core::config::ConfigError;
// // use aggregator_core::config::*;
// use aggregator_core::error::*;

// use std::io;

// // #[test]
// // fn test_config_error_display() {
// //     let err = AggregatorError::Config(ConfigError::MissingField(
// //         "api_key".to_string(),
// //     ));
// //     let msg = format!("{}", err);
// //     assert!(msg.contains("Configuration error"));
// // }

// #[test]
// fn test_serialization_error_display() {
//     let err = AggregatorError::Serialization(serde_json::Error::custom("bad json"));
//     let msg = format!("{}", err);
//     assert!(msg.contains("Serialization error"));
// }

// #[test]
// fn test_channel_send_error() {
//     let err = AggregatorError::ChannelSend {
//         message: "fail".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Channel send error"));
// }

// #[test]
// fn test_exchange_error() {
//     let err = AggregatorError::ExchangeError {
//         exchange: "Binance".to_string(),
//         message: "oops".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Exchange 'Binance' error"));
// }

// #[test]
// fn test_orderbook_error() {
//     let err = AggregatorError::OrderBookError {
//         operation: "insert".to_string(),
//         message: "fail".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("OrderBook error"));
// }

// #[test]
// fn test_network_error() {
//     let err = AggregatorError::NetworkError {
//         message: "timeout".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Network error"));
// }

// #[test]
// fn test_http_request_error() {
//     let err = AggregatorError::HttpRequestError {
//         status_code: 404,
//         message: "not found".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("HTTP request failed"));
// }

// #[test]
// fn test_parsing_error() {
//     let err = AggregatorError::Parsing {
//         data_type: "Exchange".to_string(),
//         message: "bad".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Parsing error for Exchange"));
// }

// #[test]
// fn test_timeout_error() {
//     let err = AggregatorError::Timeout {
//         operation: "fetch",
//         duration_ms: 1000,
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Timeout error"));
// }

// #[test]
// fn test_io_error() {
//     let err = AggregatorError::Io(io::Error::new(io::ErrorKind::Other, "io fail"));
//     let msg = format!("{}", err);
//     assert!(msg.contains("IO error"));
// }

// #[test]
// fn test_validation_error() {
//     let err = AggregatorError::Validation {
//         field: "symbol".to_string(),
//         message: "empty".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Validation error"));
// }

// #[test]
// fn test_database_error() {
//     let err = AggregatorError::Database {
//         operation: "insert".to_string(),
//         message: "fail".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Database error"));
// }

// #[test]
// fn test_authentication_error() {
//     let err = AggregatorError::Authentication {
//         message: "bad login".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Authentication error"));
// }

// #[test]
// fn test_rate_limit_error() {
//     let err = AggregatorError::RateLimit {
//         resource: "api",
//         message: "too many".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Rate limit exceeded"));
// }

// #[test]
// fn test_internal_error() {
//     let err = AggregatorError::Internal {
//         message: "fail".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Internal server error"));
// }

// #[test]
// fn test_not_found_error() {
//     let err = AggregatorError::NotFound {
//         resource: "order",
//         id: "123".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Not found"));
// }

// #[test]
// fn test_already_exists_error() {
//     let err = AggregatorError::AlreadyExists {
//         resource: "order",
//         id: "123".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Already exists"));
// }

// #[test]
// fn test_shutdown_error() {
//     let err = AggregatorError::Shutdown {
//         message: "fail".to_string(),
//     };
//     let msg = format!("{}", err);
//     assert!(msg.contains("Shutdown error"));
// }

// // mod aggregator_tests;

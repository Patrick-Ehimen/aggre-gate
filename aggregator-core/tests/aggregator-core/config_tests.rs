// aggregator-core/tests/aggregator-core/config_tests.rs
// Unit tests for config.rs

use aggregator_core::config::*;
use aggregator_core::types::*;

#[test]
fn test_exchange_config_defaults() {
    let rate_limit = RateLimitConfig {
        requests_per_second: 10,
        burst_size: 5,
    };
    let ws_config = WebSocketConfig {
        reconnect_interval: 1000,
        ping_interval: 5000,
        max_reconnect_attempts: 3,
        buffer_size: 1024,
    };
    let ex_cfg = ExchangeConfig {
        enabled: true,
        api_key: Some("key".to_string()),
        api_secret: Some("secret".to_string()),
        passphrase: None,
        sandbox: false,
        rate_limit: rate_limit.clone(),
        websocket: ws_config.clone(),
    };
    assert!(ex_cfg.enabled);
    assert_eq!(ex_cfg.api_key.as_deref(), Some("key"));
    assert_eq!(ex_cfg.api_secret.as_deref(), Some("secret"));
    assert!(!ex_cfg.sandbox);
    assert_eq!(ex_cfg.rate_limit.requests_per_second, 10);
    assert_eq!(ex_cfg.websocket.reconnect_interval, 1000);
}

#[test]
fn test_orderbook_config() {
    let ob_cfg = OrderBookConfig {
        max_depth: 10,
        market_type: MarketType::Spot,
        update_interval: 100,
        cleanup_interval: 1000,
        implementation: OrderBookImplementation::BTreeSet,
    };
    assert_eq!(ob_cfg.max_depth, 10);
    assert_eq!(ob_cfg.market_type, MarketType::Spot);
    assert_eq!(ob_cfg.update_interval, 100);
    assert_eq!(ob_cfg.cleanup_interval, 1000);
}

#[test]
fn test_server_config() {
    let grpc = GrpcConfig {
        enabled: true,
        host: "localhost".to_string(),
        port: 50051,
        tls: None,
    };
    let rest = RestConfig {
        enabled: true,
        host: "localhost".to_string(),
        port: 8080,
        cors: CorsConfig {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
        },
    };
    let ws = WebSocketServerConfig {
        enabled: true,
        host: "localhost".to_string(),
        port: 9000,
        max_connections: 100,
    };
    let server_cfg = ServerConfig {
        grpc,
        rest,
        websocket: ws,
    };
    assert!(server_cfg.grpc.enabled);
    assert!(server_cfg.rest.enabled);
    assert!(server_cfg.websocket.enabled);
}

#[test]
fn test_config_struct() {
    let mut exchanges = std::collections::HashMap::new();
    exchanges.insert(
        Exchange::Binance,
        ExchangeConfig {
            enabled: true,
            api_key: None,
            api_secret: None,
            passphrase: None,
            sandbox: false,
            rate_limit: RateLimitConfig {
                requests_per_second: 1,
                burst_size: 1,
            },
            websocket: WebSocketConfig {
                reconnect_interval: 1000,
                ping_interval: 1000,
                max_reconnect_attempts: 1,
                buffer_size: 256,
            },
        },
    );
    let config = Config {
        exchanges,
        trading_pairs: vec![TradingPair::new("BTC", "USD")],
        orderbook: OrderBookConfig {
            max_depth: 5,
            market_type: MarketType::Spot,
            update_interval: 100,
            cleanup_interval: 1000,
            implementation: OrderBookImplementation::BTreeSet,
        },
        server: ServerConfig {
            grpc: GrpcConfig {
                enabled: true,
                host: "localhost".to_string(),
                port: 1,
                tls: None,
            },
            rest: RestConfig {
                enabled: true,
                host: "localhost".to_string(),
                port: 2,
                cors: CorsConfig {
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                    allowed_headers: vec!["Content-Type".to_string()],
                },
            },
            websocket: WebSocketServerConfig {
                enabled: true,
                host: "localhost".to_string(),
                port: 3,
                max_connections: 10,
            },
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
            file_path: None,
            max_file_size: 1024 * 1024,
            max_files: 5,
        },
        metrics: MetricsConfig {
            enabled: true,
            prometheus: PrometheusConfig {
                enabled: true,
                host: "localhost".to_string(),
                port: 9000,
                path: "/metrics".to_string(),
            },
        },
    };
    assert_eq!(config.trading_pairs[0].base, "BTC");
    assert_eq!(config.orderbook.max_depth, 5);
    assert!(config.server.grpc.enabled);
}

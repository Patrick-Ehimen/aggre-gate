use exchange_connectors::{Binance, Bybit, Kraken};

#[cfg(test)]
mod binance_tests {
    use super::*;

    #[test]
    fn test_binance_creation() {
        let binance = Binance::new();
        // Test that creation doesn't panic
        assert_eq!(std::mem::size_of_val(&binance), std::mem::size_of::<Binance>());
    }

    #[test]
    fn test_binance_default() {
        let binance1 = Binance::new();
        let binance2 = Binance::default();
        
        // Both should be equivalent (empty structs)
        assert_eq!(
            std::mem::size_of_val(&binance1),
            std::mem::size_of_val(&binance2)
        );
    }
}

#[cfg(test)]
mod bybit_tests {
    use super::*;

    #[test]
    fn test_bybit_creation() {
        let bybit = Bybit::new();
        assert_eq!(std::mem::size_of_val(&bybit), std::mem::size_of::<Bybit>());
    }

    #[test]
    fn test_bybit_with_config() {
        let config = exchange_connectors::bybit::BybitConfig::default();
        let bybit = Bybit::with_config(config);
        assert_eq!(std::mem::size_of_val(&bybit), std::mem::size_of::<Bybit>());
    }

    #[test]
    fn test_bybit_config_default() {
        let config = exchange_connectors::bybit::BybitConfig::default();
        assert_eq!(config.websocket_url, "wss://stream.bybit.com/v5/public/linear");
        assert_eq!(config.rest_url, "https://api.bybit.com/v5/market/orderbook");
        assert_eq!(config.reconnect_interval, 5000);
        assert_eq!(config.ping_interval, 20000);
    }

    #[test]
    fn test_bybit_config_access() {
        let bybit = Bybit::new();
        // Test that we can access the config field
        assert!(!bybit.config.websocket_url.is_empty());
        assert!(!bybit.config.rest_url.is_empty());
    }
}

#[cfg(test)]
mod kraken_tests {
    use super::*;

    #[test]
    fn test_kraken_creation() {
        let kraken = Kraken::new();
        assert_eq!(std::mem::size_of_val(&kraken), std::mem::size_of::<Kraken>());
    }

    #[test]
    fn test_kraken_with_config() {
        let config = exchange_connectors::kraken::KrakenConfig::default();
        let kraken = Kraken::with_config(config);
        assert_eq!(std::mem::size_of_val(&kraken), std::mem::size_of::<Kraken>());
    }

    #[test]
    fn test_kraken_config_default() {
        let config = exchange_connectors::kraken::KrakenConfig::default();
        assert_eq!(config.websocket_url, "wss://ws.kraken.com");
        assert_eq!(config.reconnect_interval, 5000);
        assert_eq!(config.ping_interval, 20000);
    }

    #[test]
    fn test_kraken_config_access() {
        let kraken = Kraken::new();
        // Test that we can access the config field
        assert!(!kraken.config.websocket_url.is_empty());
        assert_eq!(kraken.config.reconnect_interval, 5000);
    }
}
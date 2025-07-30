use aggregator_core::PriceLevelUpdate;
use exchange_connectors::{Bitstamp, Coinbase, OrderBookService};
use tokio::sync::mpsc;

#[cfg(test)]
mod bitstamp_tests {
    use super::*;

    #[test]
    fn test_bitstamp_creation() {
        let bitstamp = Bitstamp::new();
        assert_eq!(
            std::mem::size_of_val(&bitstamp),
            std::mem::size_of::<Bitstamp>()
        );
    }

    #[test]
    fn test_bitstamp_default() {
        let bitstamp1 = Bitstamp::new();
        let bitstamp2 = Bitstamp::default();

        assert_eq!(
            std::mem::size_of_val(&bitstamp1),
            std::mem::size_of_val(&bitstamp2)
        );
    }

    #[tokio::test]
    async fn test_bitstamp_placeholder_service() {
        let bitstamp = Bitstamp::new();
        let (tx, _rx) = mpsc::channel::<PriceLevelUpdate>(100);

        let result = bitstamp
            .spawn_order_book_service(["BTC", "USD"], 100, 1000, tx)
            .await;

        // Should return Ok with empty vector (placeholder implementation)
        assert!(result.is_ok());
        let handles = result.unwrap();
        assert!(handles.is_empty());
    }
}

#[cfg(test)]
mod coinbase_tests {
    use super::*;

    #[test]
    fn test_coinbase_creation() {
        let coinbase = Coinbase::new();
        assert_eq!(
            std::mem::size_of_val(&coinbase),
            std::mem::size_of::<Coinbase>()
        );
    }

    #[test]
    fn test_coinbase_default() {
        let coinbase1 = Coinbase::new();
        let coinbase2 = Coinbase::default();

        assert_eq!(
            std::mem::size_of_val(&coinbase1),
            std::mem::size_of_val(&coinbase2)
        );
    }

    #[tokio::test]
    async fn test_coinbase_placeholder_service() {
        let coinbase = Coinbase::new();
        let (tx, _rx) = mpsc::channel::<PriceLevelUpdate>(100);

        let result = coinbase
            .spawn_order_book_service(["BTC", "USD"], 100, 1000, tx)
            .await;

        // Should return Ok with empty vector (placeholder implementation)
        assert!(result.is_ok());
        let handles = result.unwrap();
        assert!(handles.is_empty());
    }
}

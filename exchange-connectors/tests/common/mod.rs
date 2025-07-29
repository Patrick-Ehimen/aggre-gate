use aggregator_core::{Ask, Bid, Exchange, PriceLevelUpdate};
use chrono::Utc;
use uuid::Uuid;

/// Common test utilities for exchange connector tests

pub fn create_test_price_level_update(exchange: Exchange, symbol: &str) -> PriceLevelUpdate {
    PriceLevelUpdate {
        id: Uuid::new_v4(),
        symbol: symbol.to_string(),
        exchange,
        bids: vec![
            Bid {
                price: 50000.0,
                quantity: 1.0,
                exchange,
                timestamp: Utc::now(),
            },
            Bid {
                price: 49999.0,
                quantity: 2.0,
                exchange,
                timestamp: Utc::now(),
            },
        ],
        asks: vec![
            Ask {
                price: 50001.0,
                quantity: 1.5,
                exchange,
                timestamp: Utc::now(),
            },
            Ask {
                price: 50002.0,
                quantity: 0.5,
                exchange,
                timestamp: Utc::now(),
            },
        ],
        timestamp: Utc::now(),
    }
}

pub fn assert_price_level_update_valid(update: &PriceLevelUpdate) {
    assert!(!update.symbol.is_empty());
    assert!(!update.bids.is_empty() || !update.asks.is_empty());

    // Validate bids are in descending order (highest price first)
    for window in update.bids.windows(2) {
        assert!(
            window[0].price >= window[1].price,
            "Bids should be in descending price order"
        );
    }

    // Validate asks are in ascending order (lowest price first)
    for window in update.asks.windows(2) {
        assert!(
            window[0].price <= window[1].price,
            "Asks should be in ascending price order"
        );
    }

    // Validate all prices and quantities are positive
    for bid in &update.bids {
        assert!(bid.price > 0.0, "Bid price must be positive");
        assert!(bid.quantity >= 0.0, "Bid quantity must be non-negative");
    }

    for ask in &update.asks {
        assert!(ask.price > 0.0, "Ask price must be positive");
        assert!(ask.quantity >= 0.0, "Ask quantity must be non-negative");
    }
}

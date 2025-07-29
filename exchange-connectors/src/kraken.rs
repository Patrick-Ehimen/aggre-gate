//! Kraken Exchange Connector
//! Handles WebSocket connections and order book streaming for Kraken

use async_trait::async_trait;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

use crate::OrderBookService;
use aggregator_core::{AggregatorError, Ask, Bid, Exchange, PriceLevelUpdate, Result};

const KRAKEN_WS_URL: &str = "wss://ws.kraken.com";

pub struct Kraken {
    pub config: KrakenConfig,
}

#[derive(Debug, Clone)]
pub struct KrakenConfig {
    pub websocket_url: String,
    pub reconnect_interval: u64,
    pub ping_interval: u64,
}

impl Default for KrakenConfig {
    fn default() -> Self {
        Self {
            websocket_url: KRAKEN_WS_URL.to_string(),
            reconnect_interval: 5000,
            ping_interval: 20000,
        }
    }
}

#[derive(Debug, Serialize)]
struct KrakenSubscription {
    event: String,
    pair: Vec<String>,
    subscription: SubscriptionDetails,
}

#[derive(Debug, Serialize)]
struct SubscriptionDetails {
    name: String,
    depth: usize,
}

#[derive(Debug, Deserialize)]
struct KrakenMessage {
    #[serde(flatten)]
    data: Option<KrakenData>,
    event: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum KrakenData {
    Book(Vec<Value>),
}

#[derive(Debug, Deserialize)]
struct KrakenOrderBookSnapshot {
    #[serde(rename = "as")]
    asks: Vec<[String; 3]>,
    #[serde(rename = "bs")]
    bids: Vec<[String; 3]>,
}

#[derive(Debug, Deserialize)]
struct KrakenOrderBookUpdate {
    #[serde(rename = "a")]
    asks: Option<Vec<[String; 3]>>,
    #[serde(rename = "b")]
    bids: Option<Vec<[String; 3]>>,
}

impl Kraken {
    pub fn new() -> Self {
        Self {
            config: KrakenConfig::default(),
        }
    }

    pub fn with_config(config: KrakenConfig) -> Self {
        Self { config }
    }

    fn format_symbol(&self, pair: [&str; 2]) -> String {
        format!("{}/{}", pair[0].to_uppercase(), pair[1].to_uppercase())
    }

    fn parse_price_level(&self, level: &[String; 3]) -> Result<(f64, f64)> {
        let price = level[0]
            .parse::<f64>()
            .map_err(|e| AggregatorError::Parsing(format!("Invalid price: {}", e)))?;
        let quantity = level[1]
            .parse::<f64>()
            .map_err(|e| AggregatorError::Parsing(format!("Invalid quantity: {}", e)))?;
        Ok((price, quantity))
    }

    fn create_price_level_update(
        &self,
        symbol: &str,
        snapshot: &KrakenOrderBookSnapshot,
    ) -> Result<PriceLevelUpdate> {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for bid_data in &snapshot.bids {
            let (price, quantity) = self.parse_price_level(bid_data)?;
            bids.push(Bid {
                price,
                quantity,
                exchange: Exchange::Kraken,
                timestamp: Utc::now(),
            });
        }

        for ask_data in &snapshot.asks {
            let (price, quantity) = self.parse_price_level(ask_data)?;
            asks.push(Ask {
                price,
                quantity,
                exchange: Exchange::Kraken,
                timestamp: Utc::now(),
            });
        }

        Ok(PriceLevelUpdate {
            id: Uuid::new_v4(),
            symbol: symbol.to_string(),
            exchange: Exchange::Kraken,
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    fn create_price_level_delta(
        &self,
        symbol: &str,
        update: &KrakenOrderBookUpdate,
    ) -> Result<PriceLevelUpdate> {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        if let Some(bid_updates) = &update.bids {
            for bid_data in bid_updates {
                let (price, quantity) = self.parse_price_level(bid_data)?;
                bids.push(Bid {
                    price,
                    quantity,
                    exchange: Exchange::Kraken,
                    timestamp: Utc::now(),
                });
            }
        }

        if let Some(ask_updates) = &update.asks {
            for ask_data in ask_updates {
                let (price, quantity) = self.parse_price_level(ask_data)?;
                asks.push(Ask {
                    price,
                    quantity,
                    exchange: Exchange::Kraken,
                    timestamp: Utc::now(),
                });
            }
        }

        Ok(PriceLevelUpdate {
            id: Uuid::new_v4(),
            symbol: symbol.to_string(),
            exchange: Exchange::Kraken,
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    async fn spawn_websocket_stream(
        &self,
        symbol: String,
        depth: usize,
        exchange_stream_buffer: usize,
    ) -> Result<(Receiver<Message>, JoinHandle<Result<()>>)> {
        let (ws_tx, ws_rx) = tokio::sync::mpsc::channel::<Message>(exchange_stream_buffer);
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            loop {
                match Self::connect_websocket(&config, &symbol, depth, ws_tx.clone()).await {
                    Ok(_) => {
                        warn!("WebSocket connection closed, reconnecting...");
                    }
                    Err(e) => {
                        error!("WebSocket connection error: {}", e);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(
                    config.reconnect_interval,
                ))
                .await;
            }
        });

        Ok((ws_rx, handle))
    }

    async fn connect_websocket(
        config: &KrakenConfig,
        symbol: &str,
        depth: usize,
        ws_tx: Sender<Message>,
    ) -> Result<()> {
        let url = Url::parse(&config.websocket_url)
            .map_err(|e| AggregatorError::Parsing(format!("Invalid URL: {}", e)))?;

        let (mut ws_stream, _) = tokio_tungstenite::connect_async(url).await.map_err(|e| {
            AggregatorError::NetworkError(format!("WebSocket connection failed: {}", e))
        })?;

        info!("Connected to Kraken WebSocket");

        // Subscribe to orderbook updates
        let subscription = KrakenSubscription {
            event: "subscribe".to_string(),
            pair: vec![symbol.to_string()],
            subscription: SubscriptionDetails {
                name: "book".to_string(),
                depth,
            },
        };

        let subscription_msg =
            serde_json::to_string(&subscription).map_err(|e| AggregatorError::Serialization(e))?;

        ws_stream
            .send(Message::Text(subscription_msg))
            .await
            .map_err(|e| {
                AggregatorError::NetworkError(format!("Failed to send subscription: {}", e))
            })?;

        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(message) => {
                    if let Err(e) = ws_tx.send(message).await {
                        error!("Failed to send message: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_websocket_messages(
        &self,
        symbol: String,
        mut ws_rx: Receiver<Message>,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<()> {
        while let Some(message) = ws_rx.recv().await {
            if let Message::Text(text) = message {
                match serde_json::from_str::<Value>(&text) {
                    Ok(value) => {
                        if let Some(arr) = value.as_array() {
                            if arr.len() > 1 {
                                let data = &arr[1];
                                if let Ok(snapshot) =
                                    serde_json::from_value::<KrakenOrderBookSnapshot>(data.clone())
                                {
                                    match self.create_price_level_update(&symbol, &snapshot) {
                                        Ok(update) => {
                                            if let Err(e) = price_level_tx.send(update).await {
                                                error!("Failed to send snapshot update: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to create snapshot update: {}", e);
                                        }
                                    }
                                } else if let Ok(update) =
                                    serde_json::from_value::<KrakenOrderBookUpdate>(data.clone())
                                {
                                    match self.create_price_level_delta(&symbol, &update) {
                                        Ok(delta) => {
                                            if let Err(e) = price_level_tx.send(delta).await {
                                                error!("Failed to send delta update: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to create delta update: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl OrderBookService for Kraken {
    async fn spawn_order_book_service(
        &self,
        pair: [&str; 2],
        order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        let symbol = self.format_symbol(pair);
        info!("Starting Kraken order book service for {}", symbol);

        let (ws_rx, ws_handle) = self
            .spawn_websocket_stream(symbol.clone(), order_book_depth, exchange_stream_buffer)
            .await?;

        let self_clone = Self::new();
        let message_handle = tokio::spawn(async move {
            self_clone
                .handle_websocket_messages(symbol, ws_rx, price_level_tx)
                .await
        });

        Ok(vec![ws_handle, message_handle])
    }
}

impl Default for Kraken {
    fn default() -> Self {
        Self::new()
    }
}

//! Bybit Exchange Connector
//! Handles WebSocket connections and order book streaming for Bybit

use async_trait::async_trait;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

use crate::OrderBookService;
use aggregator_core::{AggregatorError, Ask, Bid, Exchange, PriceLevelUpdate, Result};

const BYBIT_WS_URL: &str = "wss://stream.bybit.com/v5/public/linear";
const BYBIT_REST_URL: &str = "https://api.bybit.com/v5/market/orderbook";

pub struct Bybit {
    pub config: BybitConfig,
}

#[derive(Debug, Clone)]
pub struct BybitConfig {
    pub websocket_url: String,
    pub rest_url: String,
    pub reconnect_interval: u64,
    pub ping_interval: u64,
}

impl Default for BybitConfig {
    fn default() -> Self {
        Self {
            websocket_url: BYBIT_WS_URL.to_string(),
            rest_url: BYBIT_REST_URL.to_string(),
            reconnect_interval: 5000,
            ping_interval: 20000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BybitSubscription {
    op: String,
    args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BybitPong {
    op: String,
}

#[derive(Debug, Deserialize)]
struct BybitDepthMessage {
    topic: String,
    #[serde(rename = "type")]
    data_type: String,
    ts: u64,
    data: BybitDepthData,
}

#[derive(Debug, Deserialize)]
struct BybitDepthData {
    s: String,
    b: Vec<[String; 2]>,
    a: Vec<[String; 2]>,
    u: u64,
    seq: u64,
}

#[derive(Debug, Deserialize)]
struct BybitSnapshotResponse {
    #[serde(rename = "retCode")]
    ret_code: i32,
    #[serde(rename = "retMsg")]
    ret_msg: String,
    result: BybitSnapshotResult,
}

#[derive(Debug, Deserialize)]
struct BybitSnapshotResult {
    s: String,
    b: Vec<[String; 2]>,
    a: Vec<[String; 2]>,
    ts: u64,
    u: u64,
}

impl Bybit {
    pub fn new() -> Self {
        Self {
            config: BybitConfig::default(),
        }
    }

    pub fn with_config(config: BybitConfig) -> Self {
        Self { config }
    }

    fn format_symbol(&self, pair: [&str; 2]) -> String {
        format!("{}{}", pair[0].to_uppercase(), pair[1].to_uppercase())
    }

    async fn get_orderbook_snapshot(
        &self,
        symbol: &str,
        depth: usize,
    ) -> Result<BybitSnapshotResult> {
        let url = format!("{}?symbol={}&limit={}", self.config.rest_url, symbol, depth);

        let response = reqwest::get(&url)
            .await
            .map_err(|e| AggregatorError::Network(format!("Failed to get snapshot: {}", e)))?;

        if !response.status().is_success() {
            return Err(AggregatorError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let snapshot: BybitSnapshotResponse = response
            .json()
            .await
            .map_err(|e| AggregatorError::Parsing(format!("Failed to parse snapshot: {}", e)))?;

        if snapshot.ret_code != 0 {
            return Err(AggregatorError::Exchange(format!(
                "Bybit API error: {}",
                snapshot.ret_msg
            )));
        }

        Ok(snapshot.result)
    }

    fn parse_price_level(&self, level: &[String; 2]) -> Result<(f64, f64)> {
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
        data: &BybitDepthData,
    ) -> Result<PriceLevelUpdate> {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for bid_data in &data.b {
            let (price, quantity) = self.parse_price_level(bid_data)?;
            if quantity > 0.0 {
                bids.push(Bid {
                    price,
                    quantity,
                    exchange: Exchange::Bybit,
                    timestamp: Utc::now(),
                });
            }
        }

        for ask_data in &data.a {
            let (price, quantity) = self.parse_price_level(ask_data)?;
            if quantity > 0.0 {
                asks.push(Ask {
                    price,
                    quantity,
                    exchange: Exchange::Bybit,
                    timestamp: Utc::now(),
                });
            }
        }

        Ok(PriceLevelUpdate {
            id: Uuid::new_v4(),
            symbol: symbol.to_string(),
            exchange: Exchange::Bybit,
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    async fn spawn_websocket_stream(
        &self,
        symbol: String,
        exchange_stream_buffer: usize,
    ) -> Result<(Receiver<Message>, JoinHandle<Result<()>>)> {
        let (ws_tx, ws_rx) = tokio::sync::mpsc::channel::<Message>(exchange_stream_buffer);
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            loop {
                match Self::connect_websocket(&config, &symbol, ws_tx.clone()).await {
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
        config: &BybitConfig,
        symbol: &str,
        ws_tx: Sender<Message>,
    ) -> Result<()> {
        let url = Url::parse(&config.websocket_url)
            .map_err(|e| AggregatorError::Parsing(format!("Invalid URL: {}", e)))?;

        let (mut ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|e| AggregatorError::Network(format!("WebSocket connection failed: {}", e)))?;

        info!("Connected to Bybit WebSocket");

        // Subscribe to orderbook updates
        let subscription = BybitSubscription {
            op: "subscribe".to_string(),
            args: vec![format!("orderbook.50.{}", symbol)],
        };

        let subscription_msg =
            serde_json::to_string(&subscription).map_err(|e| AggregatorError::Serialization(e))?;

        ws_stream
            .send(Message::Text(subscription_msg))
            .await
            .map_err(|e| AggregatorError::Network(format!("Failed to send subscription: {}", e)))?;

        let mut last_ping = std::time::Instant::now();

        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = ws_tx.send(Message::Text(text)).await {
                        error!("Failed to send message: {}", e);
                        break;
                    }
                }
                Ok(Message::Ping(_)) => {
                    if let Err(e) = ws_stream.send(Message::Pong(vec![])).await {
                        error!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }

            // Send periodic pings
            if last_ping.elapsed().as_millis() > config.ping_interval {
                let pong = BybitPong {
                    op: "pong".to_string(),
                };
                let pong_msg =
                    serde_json::to_string(&pong).map_err(|e| AggregatorError::Serialization(e))?;

                if let Err(e) = ws_stream.send(Message::Text(pong_msg)).await {
                    error!("Failed to send pong: {}", e);
                    break;
                }
                last_ping = std::time::Instant::now();
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
        let mut is_initialized = false;

        while let Some(message) = ws_rx.recv().await {
            match message {
                Message::Text(text) => {
                    if text.contains("orderbook.50") {
                        match serde_json::from_str::<BybitDepthMessage>(&text) {
                            Ok(depth_msg) => {
                                if depth_msg.data_type == "snapshot" {
                                    is_initialized = true;
                                    info!("Received orderbook snapshot for {}", symbol);
                                }

                                if is_initialized {
                                    match self.create_price_level_update(&symbol, &depth_msg.data) {
                                        Ok(update) => {
                                            if let Err(e) = price_level_tx.send(update).await {
                                                error!("Failed to send price level update: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to create price level update: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse depth message: {}", e);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl OrderBookService for Bybit {
    async fn spawn_order_book_service(
        &self,
        pair: [&str; 2],
        _order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        let symbol = self.format_symbol(pair);
        info!("Starting Bybit order book service for {}", symbol);

        let (ws_rx, ws_handle) = self
            .spawn_websocket_stream(symbol.clone(), exchange_stream_buffer)
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

impl Default for Bybit {
    fn default() -> Self {
        Self::new()
    }
}

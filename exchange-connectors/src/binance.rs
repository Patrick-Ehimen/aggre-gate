//! Binance Exchange Module
//! Handles connectivity and interaction with Binance's API

use async_trait::async_trait;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

use crate::OrderBookService;
use aggregator_core::{AggregatorError, Ask, Bid, Exchange, PriceLevelUpdate, Result};

const WS_BASE_ENDPOINT: &str = "wss://stream.binance.com:9443/ws/";
const ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT: &str = "https://api.binance.com/api/v3/depth?symbol=";
const DEPTH_UPDATE_EVENT: &str = "depthUpdate";
const GET_ORDER_BOOK_SNAPSHOT: Vec<u8> = vec![];

pub struct Binance;

#[async_trait]
impl OrderBookService for Binance {
    async fn spawn_order_book_service(
        &self,
        pair: [&str; 2],
        order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        let pair_str = pair.join("");
        let stream_pair = pair_str.to_lowercase();
        let snapshot_pair = pair_str.to_uppercase();

        info!("Spawning Binance order book stream for {}", stream_pair);

        // Spawn WebSocket stream handler
        let (ws_stream_rx, stream_handle) =
            Self::spawn_order_book_stream(stream_pair, exchange_stream_buffer);

        info!("Spawning Binance order book stream processor");

        // Spawn stream processor
        let processor_handle = Self::spawn_stream_processor(
            snapshot_pair,
            order_book_depth,
            ws_stream_rx,
            price_level_tx,
        );

        Ok(vec![stream_handle, processor_handle])
    }
}

impl Binance {
    pub fn new() -> Self {
        Binance
    }

    /// Spawn WebSocket stream for order book updates
    fn spawn_order_book_stream(
        pair: String,
        exchange_stream_buffer: usize,
    ) -> (tokio::sync::mpsc::Receiver<Message>, JoinHandle<Result<()>>) {
        let (ws_stream_tx, ws_stream_rx) =
            tokio::sync::mpsc::channel::<Message>(exchange_stream_buffer);

        let stream_handle = tokio::spawn(async move {
            let ws_stream_tx = ws_stream_tx.clone();
            loop {
                let order_book_endpoint = format!("{}{}{}", WS_BASE_ENDPOINT, pair, "@depth");

                match connect_async(&order_book_endpoint).await {
                    Ok((mut ws_stream, _)) => {
                        info!("WebSocket connection established for {}", pair);

                        // Signal to get initial snapshot
                        if let Err(e) = ws_stream_tx
                            .send(Message::Binary(GET_ORDER_BOOK_SNAPSHOT))
                            .await
                        {
                            error!("Failed to send snapshot signal: {}", e);
                            continue;
                        }

                        // Process messages from WebSocket
                        while let Some(msg) = ws_stream.next().await {
                            match msg {
                                Ok(Message::Text(_)) => {
                                    if let Err(e) = ws_stream_tx.send(msg.unwrap()).await {
                                        error!("Failed to send message: {}", e);
                                        break;
                                    }
                                }
                                Ok(Message::Ping(_)) => {
                                    info!("Received ping from Binance");
                                    if let Err(e) = ws_stream.send(Message::Pong(vec![])).await {
                                        error!("Failed to send pong: {}", e);
                                    }
                                }
                                Ok(Message::Close(_)) => {
                                    warn!("WebSocket connection closed, reconnecting...");
                                    break;
                                }
                                Err(e) => {
                                    error!("WebSocket error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect to Binance WebSocket: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        (ws_stream_rx, stream_handle)
    }

    /// Spawn stream processor for handling order book updates
    fn spawn_stream_processor(
        pair: String,
        order_book_depth: usize,
        mut ws_stream_rx: tokio::sync::mpsc::Receiver<Message>,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let mut last_update_id = 0u64;

            while let Some(message) = ws_stream_rx.recv().await {
                match message {
                    Message::Text(text) => {
                        if let Err(e) =
                            Self::process_depth_update(&text, &mut last_update_id, &price_level_tx)
                                .await
                        {
                            error!("Failed to process depth update: {}", e);
                        }
                    }
                    Message::Binary(data) => {
                        if data.is_empty() {
                            // Get order book snapshot
                            if let Err(e) = Self::process_snapshot(
                                &pair,
                                order_book_depth,
                                &mut last_update_id,
                                &price_level_tx,
                            )
                            .await
                            {
                                error!("Failed to process snapshot: {}", e);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        })
    }

    /// Process depth update message
    async fn process_depth_update(
        message: &str,
        last_update_id: &mut u64,
        price_level_tx: &Sender<PriceLevelUpdate>,
    ) -> Result<()> {
        // Parse the event to check if it's a depth update
        let event: OrderBookEvent = serde_json::from_str(message)
            .map_err(|e| AggregatorError::Parsing(format!("Failed to parse event: {}", e)))?;

        if event.event == DEPTH_UPDATE_EVENT {
            let update: OrderBookUpdate = serde_json::from_str(message).map_err(|e| {
                AggregatorError::Parsing(format!("Failed to parse depth update: {}", e))
            })?;

            // Validate update sequence
            if update.final_updated_id <= *last_update_id {
                warn!("Received out of order update, ignoring");
                return Ok(());
            }

            if update.first_update_id <= *last_update_id + 1
                && update.final_updated_id >= *last_update_id + 1
            {
                // Process bids and asks
                let mut bids = Vec::new();
                for bid_data in update.bids {
                    let price: f64 = bid_data[0].parse().map_err(|e| {
                        AggregatorError::Parsing(format!("Invalid bid price: {}", e))
                    })?;
                    let quantity: f64 = bid_data[1].parse().map_err(|e| {
                        AggregatorError::Parsing(format!("Invalid bid quantity: {}", e))
                    })?;

                    bids.push(Bid {
                        price,
                        quantity,
                        exchange: Exchange::Binance,
                        timestamp: Utc::now(),
                    });
                }

                let mut asks = Vec::new();
                for ask_data in update.asks {
                    let price: f64 = ask_data[0].parse().map_err(|e| {
                        AggregatorError::Parsing(format!("Invalid ask price: {}", e))
                    })?;
                    let quantity: f64 = ask_data[1].parse().map_err(|e| {
                        AggregatorError::Parsing(format!("Invalid ask quantity: {}", e))
                    })?;

                    asks.push(Ask {
                        price,
                        quantity,
                        exchange: Exchange::Binance,
                        timestamp: Utc::now(),
                    });
                }

                let price_level_update = PriceLevelUpdate {
                    id: uuid::Uuid::new_v4(),
                    symbol: update.symbol.clone(),
                    exchange: Exchange::Binance,
                    bids,
                    asks,
                    timestamp: Utc::now(),
                };

                price_level_tx.send(price_level_update).await.map_err(|e| {
                    AggregatorError::ChannelSend(format!(
                        "Failed to send price level update: {}",
                        e
                    ))
                })?;

                *last_update_id = update.final_updated_id;
            } else {
                return Err(AggregatorError::Exchange(
                    "Invalid update sequence".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Process initial order book snapshot
    async fn process_snapshot(
        pair: &str,
        order_book_depth: usize,
        last_update_id: &mut u64,
        price_level_tx: &Sender<PriceLevelUpdate>,
    ) -> Result<()> {
        info!("Getting order book snapshot for {}", pair);

        let snapshot = Self::get_order_book_snapshot(pair, order_book_depth).await?;

        let mut bids = Vec::new();
        for bid_data in snapshot.bids {
            bids.push(Bid {
                price: bid_data[0],
                quantity: bid_data[1],
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            });
        }

        let mut asks = Vec::new();
        for ask_data in snapshot.asks {
            asks.push(Ask {
                price: ask_data[0],
                quantity: ask_data[1],
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            });
        }

        let price_level_update = PriceLevelUpdate {
            id: uuid::Uuid::new_v4(),
            symbol: pair.to_string(),
            exchange: Exchange::Binance,
            bids,
            asks,
            timestamp: Utc::now(),
        };

        price_level_tx
            .send(price_level_update)
            .await
            .map_err(|e| AggregatorError::ChannelSend(format!("Failed to send snapshot: {}", e)))?;

        *last_update_id = snapshot.last_update_id;
        Ok(())
    }

    /// Get order book snapshot from REST API
    async fn get_order_book_snapshot(
        pair: &str,
        order_book_depth: usize,
    ) -> Result<OrderBookSnapshot> {
        let url = format!(
            "{}{}&limit={}",
            ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT, pair, order_book_depth
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| AggregatorError::Network(format!("Failed to get snapshot: {}", e)))?;

        if response.status().is_success() {
            let snapshot: OrderBookSnapshot = response.json().await.map_err(|e| {
                AggregatorError::Parsing(format!("Failed to parse snapshot: {}", e))
            })?;
            Ok(snapshot)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AggregatorError::Network(format!(
                "HTTP error: {}",
                error_text
            )))
        }
    }
}

#[derive(Debug, Deserialize)]
struct OrderBookSnapshot {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,
    bids: Vec<[f64; 2]>,
    asks: Vec<[f64; 2]>,
}

#[derive(Debug, Deserialize)]
struct OrderBookUpdate {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "U")]
    first_update_id: u64,
    #[serde(rename = "u")]
    final_updated_id: u64,
    #[serde(rename = "b")]
    bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
struct OrderBookEvent {
    #[serde(rename = "e")]
    event: String,
}

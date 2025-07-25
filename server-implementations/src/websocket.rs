//! WebSocket server implementation for crypto orderbook aggregator

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};

use crate::Server as ServerTrait;
use aggregator_core::{Aggregator, AggregatorError, Result, Summary};

/// WebSocket server implementation
pub struct WebSocketServer {
    host: String,
    port: u16,
    max_connections: usize,
}

impl WebSocketServer {
    /// Create new WebSocket server
    pub fn new(host: String, port: u16, max_connections: usize) -> Self {
        Self {
            host,
            port,
            max_connections,
        }
    }
}

#[async_trait]
impl ServerTrait for WebSocketServer {
    async fn start(&self, aggregator: Arc<Aggregator>) -> Result<JoinHandle<Result<()>>> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| AggregatorError::network(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("Starting WebSocket server on {}", addr);

        let connection_count = Arc::new(AtomicUsize::new(0));
        let max_connections = self.max_connections;

        let handle = tokio::spawn(async move {
            let mut summary_receiver = aggregator.subscribe_summaries();
            let client_senders = Arc::new(RwLock::new(
                HashMap::<usize, broadcast::Sender<String>>::new(),
            ));
            let client_id_counter = Arc::new(AtomicUsize::new(0));

            // Spawn background task to broadcast summaries to all connected clients
            let broadcast_task = {
                let client_senders = client_senders.clone();
                tokio::spawn(async move {
                    while let Ok(summary) = summary_receiver.recv().await {
                        let message = json!({
                            "type": "summary",
                            "data": {
                                "symbol": summary.symbol,
                                "spread": summary.spread,
                                "bids": summary.bids,
                                "asks": summary.asks,
                                "timestamp": summary.timestamp,
                            }
                        })
                        .to_string();

                        let senders = client_senders.read().await;
                        for (client_id, sender) in senders.iter() {
                            if let Err(e) = sender.send(message.clone()) {
                                warn!("Failed to send message to client {}: {}", client_id, e);
                            }
                        }
                    }
                })
            };

            // Accept incoming connections
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let current_connections = connection_count.load(Ordering::Relaxed);
                        if current_connections >= max_connections {
                            warn!(
                                "Maximum connections reached, rejecting connection from {}",
                                addr
                            );
                            continue;
                        }

                        connection_count.fetch_add(1, Ordering::Relaxed);
                        let client_id = client_id_counter.fetch_add(1, Ordering::Relaxed);

                        info!(
                            "New WebSocket connection from {} (client_id: {})",
                            addr, client_id
                        );

                        let connection_count_clone = connection_count.clone();
                        let client_senders_clone = client_senders.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(
                                stream,
                                client_id,
                                client_senders_clone,
                                connection_count_clone,
                            )
                            .await
                            {
                                error!("Error handling connection from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(handle)
    }

    async fn stop(&self) -> Result<()> {
        // WebSocket server shutdown is handled by dropping the listener
        Ok(())
    }

    fn name(&self) -> &'static str {
        "WebSocket"
    }

    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

async fn handle_connection(
    stream: TcpStream,
    client_id: usize,
    client_senders: Arc<RwLock<HashMap<usize, broadcast::Sender<String>>>>,
    connection_count: Arc<AtomicUsize>,
) -> Result<()> {
    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| AggregatorError::network(format!("WebSocket handshake failed: {}", e)))?;

    let (mut tx, mut rx) = ws_stream.split();
    let (bcast_tx, bcast_rx) = broadcast::channel::<String>(100);

    client_senders.write().await.insert(client_id, bcast_tx);

    let mut bcast_rx = bcast_rx;

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = bcast_rx.recv().await {
            if tx.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = rx.next().await {
            // Handle incoming messages from client (e.g., subscription requests)
            // For now, we just ignore them
        }
    });

    tokio::select! {
        _ = &mut send_task => (),
        _ = &mut recv_task => (),
    }

    info!("WebSocket connection closed (client_id: {})", client_id);
    client_senders.write().await.remove(&client_id);
    connection_count.fetch_sub(1, Ordering::Relaxed);

    Ok(())
}

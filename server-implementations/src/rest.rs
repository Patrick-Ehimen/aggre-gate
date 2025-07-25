//! REST server implementation for crypto orderbook aggregator

use async_trait::async_trait;
use axum::response::Json;
use axum::{extract::Path, routing::get, Extension, Router};
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::Server as ServerTrait;
use aggregator_core::{Aggregator, AggregatorError, Result, Summary, TradingPair};

/// REST server implementation
pub struct RestServer {
    host: String,
    port: u16,
}

impl RestServer {
    /// Create new REST server
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait]
impl ServerTrait for RestServer {
    async fn start(&self, aggregator: Arc<Aggregator>) -> Result<JoinHandle<Result<()>>> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| AggregatorError::network(format!("Failed to bind to {}: {}", addr, e)))?;

        let app = create_app(aggregator);

        info!("Starting REST server on {}", addr);

        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .map_err(|e| AggregatorError::network(format!("REST server error: {}", e)))
        });
        Ok(handle)
    }

    async fn stop(&self) -> Result<()> {
        // REST server shutdown is handled by dropping the server
        Ok(())
    }

    fn name(&self) -> &'static str {
        "REST"
    }

    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn create_app(aggregator: Arc<Aggregator>) -> Router {
    Router::new()
        .route("/summary/:base/:quote", get(get_summary_handler))
        .layer(Extension(aggregator))
}

/// Handler for getting a summary
async fn get_summary_handler(
    Path((base, quote)): Path<(String, String)>,
    axum::extract::Extension(aggregator): axum::extract::Extension<Arc<Aggregator>>,
) -> Json<serde_json::Value> {
    let pair = TradingPair::new(&base, &quote);

    match aggregator.get_summary(&pair).await {
        Some(summary) => Json(json!({
            "symbol": summary.symbol,
            "spread": summary.spread,
            "bids": summary.bids,
            "asks": summary.asks,
            "timestamp": summary.timestamp,
        })),
        None => Json(json!({ "error": "Summary not found" })),
    }
}

//! gRPC server implementation for crypto orderbook aggregator

use async_trait::async_trait;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::tokio_stream::Stream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info};

use crate::Server as ServerTrait;
use aggregator_core::{
    Aggregator, AggregatorError, ArbitrageOpportunity, Exchange, HealthStatus, Metrics, Result,
    Summary, TradingPair,
};

// Define the protobuf service
pub mod orderbook_service {
    tonic::include_proto!("orderbook_service");
}

use orderbook_service::{
    orderbook_service_server::{OrderbookService, OrderbookServiceServer},
    ArbitrageMessage, GetAllSummariesRequest, GetAllSummariesResponse, GetHealthStatusRequest,
    GetHealthStatusResponse, GetMetricsRequest, GetMetricsResponse, GetSummaryRequest,
    GetSummaryResponse, HealthStatusMessage, MetricsMessage, PriceLevel, StreamArbitrageRequest,
    StreamSummariesRequest, SummaryMessage,
};

/// gRPC server implementation
pub struct GrpcServer {
    host: String,
    port: u16,
}

impl GrpcServer {
    /// Create new gRPC server
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait]
impl ServerTrait for GrpcServer {
    async fn start(&self, aggregator: Arc<Aggregator>) -> Result<JoinHandle<Result<()>>> {
        let addr = format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e| AggregatorError::Network(format!("Invalid address: {}", e)))?;

        let service = OrderbookServiceImpl::new(aggregator);

        info!("Starting gRPC server on {}", addr);

        let handle = tokio::spawn(async move {
            Server::builder()
                .add_service(OrderbookServiceServer::new(service))
                .serve(addr)
                .await
                .map_err(|e| AggregatorError::Network(format!("gRPC server error: {}", e)))
        });

        Ok(handle)
    }

    async fn stop(&self) -> Result<()> {
        // gRPC server shutdown is handled by dropping the server
        Ok(())
    }

    fn name(&self) -> &'static str {
        "gRPC"
    }

    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// gRPC service implementation
pub struct OrderbookServiceImpl {
    aggregator: Arc<Aggregator>,
}

impl OrderbookServiceImpl {
    pub fn new(aggregator: Arc<Aggregator>) -> Self {
        Self { aggregator }
    }
}

#[async_trait]
impl OrderbookService for OrderbookServiceImpl {
    /// Get summary for a specific trading pair
    async fn get_summary(
        &self,
        request: Request<GetSummaryRequest>,
    ) -> std::result::Result<Response<GetSummaryResponse>, Status> {
        let req = request.into_inner();
        let pair = TradingPair::new(&req.base, &req.quote);

        match self.aggregator.get_summary(&pair).await {
            Some(summary) => {
                let response = GetSummaryResponse {
                    summary: Some(convert_summary_to_grpc(summary)),
                };
                Ok(Response::new(response))
            }
            None => Err(Status::not_found("Summary not found")),
        }
    }

    /// Get all summaries
    async fn get_all_summaries(
        &self,
        _request: Request<GetAllSummariesRequest>,
    ) -> std::result::Result<Response<GetAllSummariesResponse>, Status> {
        let summaries = self.aggregator.get_all_summaries().await;
        let grpc_summaries: Vec<SummaryMessage> =
            summaries.into_iter().map(convert_summary_to_grpc).collect();

        let response = GetAllSummariesResponse {
            summaries: grpc_summaries,
        };

        Ok(Response::new(response))
    }

    type StreamSummariesStream =
        Pin<Box<dyn Stream<Item = std::result::Result<SummaryMessage, Status>> + Send>>;

    /// Stream summaries for all trading pairs
    async fn stream_summaries(
        &self,
        _request: Request<StreamSummariesRequest>,
    ) -> std::result::Result<Response<Self::StreamSummariesStream>, Status> {
        let mut rx = self.aggregator.subscribe_summaries();
        let stream = async_stream::stream! {
            while let Ok(summary) = rx.recv().await {
                yield Ok(convert_summary_to_grpc(summary));
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    type StreamArbitrageStream =
        Pin<Box<dyn Stream<Item = std::result::Result<ArbitrageMessage, Status>> + Send>>;

    /// Stream arbitrage opportunities
    async fn stream_arbitrage(
        &self,
        _request: Request<StreamArbitrageRequest>,
    ) -> std::result::Result<Response<Self::StreamArbitrageStream>, Status> {
        let mut rx = self.aggregator.subscribe_arbitrage();
        let stream = async_stream::stream! {
            while let Ok(opportunity) = rx.recv().await {
                yield Ok(convert_arbitrage_to_grpc(opportunity));
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    /// Get health status
    async fn get_health_status(
        &self,
        _request: Request<GetHealthStatusRequest>,
    ) -> std::result::Result<Response<GetHealthStatusResponse>, Status> {
        let health_status = self.aggregator.get_health_status().await;
        let response = GetHealthStatusResponse {
            health_status: Some(convert_health_status_to_grpc(health_status)),
        };
        Ok(Response::new(response))
    }

    /// Get metrics
    async fn get_metrics(
        &self,
        _request: Request<GetMetricsRequest>,
    ) -> std::result::Result<Response<GetMetricsResponse>, Status> {
        let metrics = self.aggregator.get_metrics().await;
        let response = GetMetricsResponse {
            metrics: Some(convert_metrics_to_grpc(metrics)),
        };
        Ok(Response::new(response))
    }
}

// --- Conversion functions ---

fn convert_summary_to_grpc(summary: Summary) -> SummaryMessage {
    SummaryMessage {
        symbol: summary.symbol,
        spread: summary.spread,
        bids: summary
            .bids
            .into_iter()
            .map(|l| PriceLevel {
                price: l.price,
                amount: l.amount,
            })
            .collect(),
        asks: summary
            .asks
            .into_iter()
            .map(|l| PriceLevel {
                price: l.price,
                amount: l.amount,
            })
            .collect(),
        timestamp: summary.timestamp,
    }
}

fn convert_arbitrage_to_grpc(opportunity: ArbitrageOpportunity) -> ArbitrageMessage {
    ArbitrageMessage {
        symbol: opportunity.symbol,
        profit_percentage: opportunity.profit_percentage,
        buy_exchange: opportunity.buy_exchange.to_string(),
        sell_exchange: opportunity.sell_exchange.to_string(),
        buy_price: opportunity.buy_price,
        sell_price: opportunity.sell_price,
        timestamp: opportunity.timestamp,
    }
}

fn convert_health_status_to_grpc(health_status: HealthStatus) -> HealthStatusMessage {
    HealthStatusMessage {
        is_healthy: health_status.is_healthy,
        message: health_status.message,
        last_update: health_status.last_update,
    }
}

fn convert_metrics_to_grpc(metrics: Metrics) -> MetricsMessage {
    MetricsMessage {
        updates_per_second: metrics.updates_per_second,
        connected_exchanges: metrics
            .connected_exchanges
            .into_iter()
            .map(|e| e.to_string())
            .collect(),
        active_trading_pairs: metrics.active_trading_pairs,
        memory_usage_bytes: metrics.memory_usage_bytes,
        cpu_usage_percentage: metrics.cpu_usage_percentage,
    }
}

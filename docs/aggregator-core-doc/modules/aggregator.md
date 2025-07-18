# Aggregator Module

## Overview

The aggregator module provides the core functionality for data aggregation and processing.

## Key Components

The `Aggregator` struct is central to the module and manages various tasks related to the life-cycle and data aggregation.

### Main Functions

- **new**: Initializes the aggregator with the specified configuration.
- **start**: Begins the aggregation process and spawns connector tasks.
- **stop**: Sends a shutdown signal to stop the aggregator gracefully.
- **subscribe_summaries**, **subscribe_arbitrage**, **subscribe_shutdown**: Provides channels for external modules to subscribe to various events.

### Data Structures

- **Aggregator**:
  - **config**: Maintains configuration settings.
  - **summaries**, **metrics**, **health_status**: Keeps track of data from exchanges.
  - **summary_sender**, **arbitrage_sender**, **shutdown_sender**: Channels for broadcasting updates and control signals.

### Example Spawning Connectors

Here's an example of how to spawn exchange connectors:

```rust
tokio::spawn(async move {
    let aggregator = Aggregator::new(config);
    aggregator.start().await;
});
```

## Subscription Channels

The aggregator provides several subscription channels for external components to receive updates:
- **Summaries**: Subscribe to get `Summary` updates.
- **Arbitrage Opportunities**: Subscribe to get notified of potential arbitrage opportunities.
- **Shutdown**: Monitor when the aggregator is shutting down.

## Shutdown Semantics

The aggregator uses a broadcast channel to propagate shutdown signals to all active tasks, ensuring a clean exit for all processes.

## Detailed Field/Function Tables

### Aggregator Struct Fields

| Field | Type | Description |
|-------|------|-------------|
| `config` | `Arc<Config>` | Shared configuration reference |
| `summaries` | `Arc<RwLock<HashMap<TradingPair, Summary>>>` | Current market summaries |
| `health_status` | `Arc<RwLock<HashMap<Exchange, HealthStatus>>>` | Exchange health tracking |
| `metrics` | `Arc<RwLock<HashMap<Exchange, Metrics>>>` | Performance metrics |
| `summary_sender` | `broadcast::Sender<Summary>` | Summary broadcast channel |
| `arbitrage_sender` | `broadcast::Sender<ArbitrageOpportunity>` | Arbitrage opportunity channel |
| `shutdown_sender` | `broadcast::Sender<()>` | Shutdown signal channel |

### Aggregator Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `new` | `config: Config` | `Self` | Creates new aggregator instance |
| `start` | `&self` | `Result<Vec<JoinHandle<Result<()>>>>` | Starts all async tasks |
| `stop` | `&self` | `Result<()>` | Initiates graceful shutdown |
| `subscribe_summaries` | `&self` | `broadcast::Receiver<Summary>` | Subscribe to summary updates |
| `subscribe_arbitrage` | `&self` | `broadcast::Receiver<ArbitrageOpportunity>` | Subscribe to arbitrage opportunities |
| `subscribe_shutdown` | `&self` | `broadcast::Receiver<()>` | Subscribe to shutdown signals |
| `get_summary` | `&self, pair: &TradingPair` | `Option<Summary>` | Get current summary for trading pair |
| `get_all_summaries` | `&self` | `HashMap<TradingPair, Summary>` | Get all current summaries |
| `get_health_status` | `&self, exchange: &Exchange` | `Option<HealthStatus>` | Get health status for exchange |
| `get_metrics` | `&self, exchange: &Exchange` | `Option<Metrics>` | Get metrics for exchange |

### Async Task Life-cycle

1. **Exchange Connectors**: Connect to exchange APIs and stream price updates
2. **Price Level Processors**: Process incoming price updates and create summaries
3. **Aggregation Processor**: Combine summaries from multiple exchanges
4. **Arbitrage Detector**: Analyze price differences across exchanges
5. **Health Monitor**: Track exchange connection health

## API Reference

Refer to the Rust documentation for further details on each function and struct. Cross-link to:
- [Config Documentation](config.md)
- [Types Documentation](types.md)
- [Error Documentation](error.md)

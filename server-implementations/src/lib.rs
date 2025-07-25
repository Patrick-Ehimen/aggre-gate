//! Server implementations for crypto orderbook aggregator
//!
//! This module provides different server implementations:
//! - gRPC server for high-performance streaming
//! - REST API server for HTTP-based access
//! - WebSocket server for real-time web clients

#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "rest")]
pub mod rest;
#[cfg(feature = "websocket")]
pub mod websocket;

use aggregator_core::{Aggregator, AggregatorError, Config, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Common trait for all server implementations
#[async_trait]
pub trait Server: Send + Sync {
    /// Start the server
    async fn start(&self, aggregator: Arc<Aggregator>) -> Result<JoinHandle<Result<()>>>;

    /// Stop the server
    async fn stop(&self) -> Result<()>;

    /// Get server name
    fn name(&self) -> &'static str;

    /// Get server address
    fn address(&self) -> String;
}

/// Server manager to coordinate multiple server types
pub struct ServerManager {
    servers: Vec<Box<dyn Server>>,
}

impl ServerManager {
    /// Create a new server manager
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    /// Add a server implementation
    pub fn add_server(&mut self, server: Box<dyn Server>) {
        self.servers.push(server);
    }

    /// Start all servers
    pub async fn start_all(
        &self,
        aggregator: Arc<Aggregator>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        let mut handles = Vec::new();

        for server in &self.servers {
            let handle = server.start(aggregator.clone()).await?;
            handles.push(handle);
        }

        Ok(handles)
    }

    /// Stop all servers
    pub async fn stop_all(&self) -> Result<()> {
        for server in &self.servers {
            server.stop().await?;
        }
        Ok(())
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create servers from config
pub fn create_servers_from_config(config: &Config) -> ServerManager {
    let mut manager = ServerManager::new();

    // Add gRPC server if enabled and feature is available
    #[cfg(feature = "grpc")]
    if config.server.grpc.enabled {
        let grpc_server =
            grpc::GrpcServer::new(config.server.grpc.host.clone(), config.server.grpc.port);
        manager.add_server(Box::new(grpc_server));
    }

    // Add REST server if enabled and feature is available
    #[cfg(feature = "rest")]
    if config.server.rest.enabled {
        let rest_server =
            rest::RestServer::new(config.server.rest.host.clone(), config.server.rest.port);
        manager.add_server(Box::new(rest_server));
    }

    // Add WebSocket server if enabled and feature is available
    #[cfg(feature = "websocket")]
    if config.server.websocket.enabled {
        let ws_server = websocket::WebSocketServer::new(
            config.server.websocket.host.clone(),
            config.server.websocket.port,
            config.server.websocket.max_connections,
        );
        manager.add_server(Box::new(ws_server));
    }

    manager
}

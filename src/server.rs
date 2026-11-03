//! HTTP Server and Metrics Collection
//!
//! This module implements the Prometheus exporter HTTP server and the metric collection loop.
//!
//! # Architecture
//!
//! - **HTTP Server**: Axum-based server exposing `/metrics`, `/health`, and `/` endpoints
//! - **Collection Loop**: Background task that periodically queries TrueNAS API and updates metrics
//! - **State Management**: Shared state (config, metrics, client) using Arc for thread-safety
//!
//! # Endpoints
//!
//! - `GET /` - HTML landing page with links to metrics and health
//! - `GET /metrics` - Prometheus metrics in text format
//! - `GET /health` - Health check (returns 200 if TrueNAS is reachable, 503 otherwise)
//!
//! # Metrics Collection
//!
//! The collection loop runs every N seconds (configured via `scrape_interval_seconds`) and:
//! 1. Queries all enabled TrueNAS API endpoints
//! 2. Updates Prometheus metrics with the latest values
//! 3. Sets `truenas_up` to 1 if any query succeeds, 0 if all fail
//!
//! # Error Handling
//!
//! Individual API failures are logged as warnings but don't stop the collection loop.
//! This ensures partial metrics are still exposed even if some APIs are unavailable.

use crate::collectors::{self, CollectionContext, CollectionStatus};
use crate::config::Config;
use crate::metrics::MetricsCollector;
use crate::truenas::TrueNasClient;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    config: Config,
    metrics: MetricsCollector,
    client: Arc<TrueNasClient>,
}

pub async fn start(config: Config) -> anyhow::Result<()> {
    let metrics = MetricsCollector::new()?;
    let client = Arc::new(TrueNasClient::new(config.truenas.clone()));

    let state = AppState {
        config: config.clone(),
        metrics: metrics.clone(),
        client: client.clone(),
    };

    // Start background metrics collection
    let collection_state = state.clone();
    tokio::spawn(async move {
        collect_metrics_loop(collection_state).await;
    });

    // Build the router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    // Start the server
    let addr = format!("{}:{}", config.server.addr, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let bound_addr = listener.local_addr()?;
    info!("Metrics server listening on {}", bound_addr);
    info!("Metrics available at http://{}/metrics", bound_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn collect_metrics_loop(state: AppState) {
    let mut ticker = interval(Duration::from_secs(
        state.config.metrics.scrape_interval_seconds,
    ));

    loop {
        ticker.tick().await;

        if let Err(e) = collect_metrics(&state).await {
            error!("Failed to collect metrics: {}", e);
            state.metrics.up.set(0.0);
        } else {
            state.metrics.up.set(1.0);
        }
    }
}

async fn collect_metrics(state: &AppState) -> anyhow::Result<()> {
    info!("Collecting metrics from TrueNAS");

    let ctx = CollectionContext {
        client: &state.client,
        metrics: &state.metrics,
        config: &state.config.metrics,
    };

    let mut any_success = false;

    // Helper macro to track success
    macro_rules! collect {
        ($collector:expr) => {
            match $collector.await? {
                CollectionStatus::Success => any_success = true,
                CollectionStatus::Failed => { /* Already logged */ }
            }
        };
    }

    // Collect pool metrics
    if state.config.metrics.collect_pool_metrics {
        collect!(collectors::collect_pool_metrics(&ctx));
        collect!(collectors::collect_dataset_metrics(&ctx));
    }

    // Collect share metrics
    collect!(collectors::collect_share_metrics(&ctx));

    // Collect data protection metrics
    collect!(collectors::collect_cloud_sync_metrics(&ctx));
    collect!(collectors::collect_snapshot_metrics(&ctx));

    // Collect alerts
    collect!(collectors::collect_alert_metrics(&ctx));

    // Collect system metrics
    if state.config.metrics.collect_system_metrics {
        collect!(collectors::collect_system_info_metrics(&ctx));
        collect!(collectors::collect_system_reporting_metrics(&ctx));
    }

    // Collect disk metrics
    collect!(collectors::collect_disk_metrics(&ctx));
    collect!(collectors::collect_smart_metrics(&ctx));

    // Collect application metrics
    collect!(collectors::collect_app_metrics(&ctx));

    // Collect network interface metrics
    collect!(collectors::collect_network_interface_metrics(&ctx));

    // Collect service status
    collect!(collectors::collect_service_metrics(&ctx));

    // Collect boot pool, NFS clients (count + per-client detail), iSCSI sessions
    collect!(collectors::collect_boot_pool_metrics(&ctx));
    collect!(collectors::collect_nfs_metrics(&ctx));
    collect!(collectors::collect_iscsi_client_count(&ctx));

    // If all queries failed, return error so truenas_up is set to 0
    if !any_success {
        anyhow::bail!("Failed to collect any metrics from TrueNAS - check authentication");
    }

    Ok(())
}

async fn root_handler() -> impl IntoResponse {
    r#"<html>
<head><title>TrueNAS Exporter</title></head>
<body>
<h1>TrueNAS Prometheus Exporter</h1>
<p><a href="/metrics">Metrics</a></p>
<p><a href="/health">Health</a></p>
</body>
</html>"#
}

async fn metrics_handler(State(state): State<AppState>) -> Response {
    match state.metrics.render() {
        Ok(metrics) => metrics.into_response(),
        Err(e) => {
            error!("Failed to render metrics: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error rendering metrics: {}", e),
            )
                .into_response()
        }
    }
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let up_value = state.metrics.up.get();

    if up_value > 0.0 {
        (axum::http::StatusCode::OK, "OK")
    } else {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "TrueNAS API unreachable",
        )
    }
}

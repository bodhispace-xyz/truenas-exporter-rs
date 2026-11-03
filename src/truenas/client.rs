//! TrueNAS WebSocket API Client
//!
//! This module provides a client for interacting with the TrueNAS Scale WebSocket API.
//! The API uses JSON-RPC 2.0 over WebSocket for all communication.
//!
//! # Architecture
//!
//! - **Connection**: WebSocket (ws:// or wss://) to `/websocket` endpoint
//! - **Authentication**: API key passed as array in initial auth message
//! - **Protocol**: JSON-RPC 2.0 with DDP (Distributed Data Protocol) handshake
//!
//! # Example
//!
//! ```no_run
//! use truenas_exporter::config::TrueNasConfig;
//! use truenas_exporter::truenas::TrueNasClient;
//! use secrecy::SecretString;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = TrueNasConfig {
//!     host: "truenas.local:443".to_string(),
//!     api_key: SecretString::from("your-api-key"),
//!     use_tls: true,
//!     verify_ssl: true,
//!     ca_cert_path: None,
//! };
//!
//! let client = TrueNasClient::new(config);
//! let pools = client.query_pools().await?;
//! # Ok(())
//! # }
//! ```

use crate::config::TrueNasConfig;
use crate::error::Result;
use crate::truenas::connection::ConnectionManager;
use crate::truenas::types::*;
use std::sync::Arc;

/// Client for TrueNAS Scale WebSocket API
///
/// Handles WebSocket connection lifecycle, authentication, and JSON-RPC method calls.
/// Uses a persistent `ConnectionManager` to reuse a single WebSocket connection across
/// multiple API calls, which is required for TrueNAS SCALE 25.04+ authentication.
///
/// # Thread Safety
///
/// This client is `Send` and `Sync`, allowing it to be shared across async tasks.
/// Request IDs are managed atomically to prevent collisions.
pub struct TrueNasClient {
    connection_manager: ConnectionManager,
}

impl TrueNasClient {
    pub fn new(config: TrueNasConfig) -> Self {
        let config = Arc::new(config);
        let connection_manager = ConnectionManager::new(config.clone());
        Self { connection_manager }
    }

    /// Query pool information
    pub async fn query_pools(&self) -> Result<Vec<Pool>> {
        self.execute_query("pool.query", Some(serde_json::Value::Null))
            .await
    }

    /// Query system information
    /// Query system information
    pub async fn query_system_info(&self) -> Result<SystemInfo> {
        self.execute_query("system.info", None).await
    }

    /// Execute a JSON-RPC method call over WebSocket
    ///
    /// This is the core method used by all API query methods. It handles:
    /// 1. WebSocket connection establishment (with optional TLS)
    /// 2. DDP handshake
    /// 3. Authentication via API key
    /// 4. Method invocation with parameters
    /// 5. Response parsing and deserialization
    ///
    /// # Arguments
    ///
    /// * `method` - The TrueNAS API method name (e.g., "pool.query", "system.info")
    /// * `params` - Optional JSON parameters for the method call
    ///
    /// # Returns
    ///
    /// Returns the deserialized response of type `T`, or an error if:
    /// - Connection fails
    /// - Authentication fails
    /// - Method call returns an error
    /// - Response cannot be deserialized
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use truenas_exporter::truenas::TrueNasClient;
    /// # use truenas_exporter::config::TrueNasConfig;
    /// # use secrecy::SecretString;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let config = TrueNasConfig {
    /// #     host: "truenas.local:443".to_string(),
    /// #     api_key: SecretString::from("key"),
    /// #     use_tls: true,
    /// #     verify_ssl: true,
    /// #     ca_cert_path: None,
    /// # };
    /// let client = TrueNasClient::new(config);
    /// let pools = client.query_pools().await?;
    /// # Ok(())
    /// # }
    /// ```
    /// Execute a JSON-RPC method call over WebSocket
    ///
    /// This method delegates to the `ConnectionManager` to handle the persistent
    /// connection, authentication, and request execution.
    async fn execute_query<T>(&self, method: &str, params: Option<serde_json::Value>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.connection_manager.execute_query(method, params).await
    }

    /// Query disk information
    pub async fn query_disks(&self) -> Result<Vec<DiskInfo>> {
        self.execute_query("disk.query", Some(serde_json::json!([])))
            .await
    }

    /// Query SMART test results
    pub async fn query_smart_tests(&self) -> Result<Vec<SmartTestDisk>> {
        self.execute_query("smart.test.results", Some(serde_json::json!([])))
            .await
    }

    pub async fn query_datasets(&self) -> Result<Vec<Dataset>> {
        let params = serde_json::json!([
            [],
            {"select": ["name", "used", "available", "compressratio", "encrypted"]}
        ]);
        self.execute_query("pool.dataset.query", Some(params)).await
    }

    pub async fn query_smb_shares(&self) -> Result<Vec<SmbShare>> {
        self.execute_query("sharing.smb.query", Some(serde_json::json!([])))
            .await
    }

    pub async fn query_nfs_shares(&self) -> Result<Vec<NfsShare>> {
        self.execute_query("sharing.nfs.query", Some(serde_json::json!([])))
            .await
    }

    pub async fn query_cloud_sync_tasks(&self) -> Result<Vec<CloudSyncTask>> {
        self.execute_query("cloudsync.query", Some(serde_json::json!([])))
            .await
    }

    pub async fn query_snapshot_tasks(&self) -> Result<Vec<SnapshotTask>> {
        self.execute_query("pool.snapshottask.query", Some(serde_json::json!([])))
            .await
    }

    pub async fn query_alerts(&self) -> Result<Vec<TruenasAlert>> {
        self.execute_query("alert.list", Some(serde_json::json!([])))
            .await
    }

    /// Query application information
    pub async fn query_apps(&self) -> Result<Vec<AppInfo>> {
        self.execute_query("app.query", Some(serde_json::json!([])))
            .await
    }

    /// Query network interfaces
    pub async fn query_network_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        self.execute_query("interface.query", Some(serde_json::json!([])))
            .await
    }

    /// Query services
    pub async fn query_services(&self) -> Result<Vec<ServiceInfo>> {
        self.execute_query("service.query", Some(serde_json::json!([])))
            .await
    }

    /// Query active NFSv4 clients with per-client detail
    pub async fn query_nfs4_clients(&self) -> Result<Vec<NfsClient>> {
        self.execute_query("nfs.get_nfs4_clients", Some(serde_json::json!([])))
            .await
    }

    /// Query active NFSv3 clients with per-client detail
    pub async fn query_nfs3_clients(&self) -> Result<Vec<NfsClient>> {
        self.execute_query("nfs.get_nfs3_clients", Some(serde_json::json!([])))
            .await
    }

    /// Query boot pool state
    pub async fn query_boot_pool(&self) -> Result<BootPoolState> {
        self.execute_query("boot.get_state", None).await
    }

    /// Query active NFS client count
    pub async fn query_nfs_client_count(&self) -> Result<u64> {
        self.execute_query("nfs.client_count", None).await
    }

    /// Query active iSCSI client count
    pub async fn query_iscsi_client_count(&self) -> Result<u64> {
        self.execute_query("iscsi.global.client_count", None).await
    }

    /// Query available reporting graphs
    pub async fn query_reporting_graphs(&self) -> Result<Vec<ReportingGraph>> {
        self.execute_query("reporting.graphs", Some(serde_json::json!([])))
            .await
    }

    /// Query reporting data
    pub async fn query_reporting_data(
        &self,
        queries: Vec<ReportingQuery>,
        start: Option<i64>,
    ) -> Result<Vec<ReportingData>> {
        // params: [queries, {options}]
        let options = if let Some(s) = start {
            serde_json::json!({"start": s})
        } else {
            // Default to start=now-300s (5 mins) if not specified
            use std::time::{SystemTime, UNIX_EPOCH};
            let start = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
                - 300;
            serde_json::json!({"start": start})
        };

        let params = serde_json::json!([queries, options]);
        self.execute_query("reporting.get_data", Some(params)).await
    }
}

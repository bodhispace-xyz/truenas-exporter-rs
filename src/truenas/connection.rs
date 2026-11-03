//! WebSocket Connection Management
//!
//! This module handles persistent WebSocket connections to TrueNAS.
//! It maintains a single long-lived connection that is reused across multiple API calls,
//! which is required for proper authentication in TrueNAS 25.04+.

use crate::config::TrueNasConfig;
use crate::error::{ExporterError, Result};
use crate::truenas::types::{DdpConnect, JsonRpcRequest, JsonRpcResponse};
use futures_util::{SinkExt, StreamExt};
use secrecy::ExposeSecret;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, info, warn};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Manages a persistent WebSocket connection to TrueNAS
pub struct ConnectionManager {
    config: Arc<TrueNasConfig>,
    connection: Arc<Mutex<Option<ActiveConnection>>>,
    request_id: Arc<std::sync::atomic::AtomicU64>,
}

/// An active WebSocket connection
struct ActiveConnection {
    stream: WsStream,
    authenticated: bool,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(config: Arc<TrueNasConfig>) -> Self {
        Self {
            config,
            connection: Arc::new(Mutex::new(None)),
            request_id: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Get next request ID
    fn next_id(&self) -> String {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string()
    }

    /// Build WebSocket URL
    fn websocket_url(&self) -> String {
        let protocol = if self.config.use_tls { "wss" } else { "ws" };
        format!("{}://{}/websocket", protocol, self.config.host)
    }

    /// Ensure we have an active, authenticated connection
    async fn ensure_connected(&self) -> Result<()> {
        let mut conn_guard = self.connection.lock().await;

        // Check if we have a connection
        if conn_guard.is_none() {
            info!("Establishing WebSocket connection to TrueNAS...");
            let stream = self.connect_websocket().await?;
            *conn_guard = Some(ActiveConnection {
                stream,
                authenticated: false,
            });
        }

        // Check if we need to authenticate
        if let Some(conn) = conn_guard.as_mut() {
            if !conn.authenticated {
                info!("Authenticating with TrueNAS...");
                if let Err(e) = self.authenticate_connection(conn).await {
                    warn!("Authentication failed, dropping connection: {}", e);
                    *conn_guard = None;
                    return Err(e);
                }
                conn.authenticated = true;
                info!("Successfully authenticated to TrueNAS");
            }
        }

        Ok(())
    }

    /// Connect to WebSocket
    async fn connect_websocket(&self) -> Result<WsStream> {
        let url = self.websocket_url();
        debug!("Connecting to {}", url);

        let connector = self
            .config
            .tls_connector()
            .map_err(|error| ExporterError::Config(format!("{error:#}")))?;
        if !self.config.use_tls || !self.config.verify_ssl {
            warn!("TrueNAS peer verification is disabled; API-key credentials are not protected against peer impersonation");
        }
        let (ws_stream, _) = if let Some(connector) = connector {
            let connector = tokio_tungstenite::Connector::NativeTls(connector);
            tokio_tungstenite::connect_async_tls_with_config(&url, None, false, Some(connector))
                .await
                .map_err(|e| ExporterError::Config(format!("TLS connection failed: {}", e)))?
        } else {
            connect_async(&url)
                .await
                .map_err(ExporterError::WebSocket)?
        };

        Ok(ws_stream)
    }

    /// Authenticate an active connection
    async fn authenticate_connection(&self, conn: &mut ActiveConnection) -> Result<()> {
        // Send DDP connect
        let connect_msg = serde_json::to_string(&DdpConnect::default())?;
        conn.stream
            .send(Message::Text(connect_msg.into()))
            .await
            .map_err(ExporterError::WebSocket)?;

        // Read connect response
        if let Some(msg) = conn.stream.next().await {
            let msg = msg.map_err(ExporterError::WebSocket)?;
            debug!("Received raw DDP response: {:?}", msg);
            if let Message::Text(text) = msg {
                debug!("DDP connect response: {}", text);
            }
        }

        // Wait a bit to ensure server is ready (mitigate potential race condition)
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        // Send auth request
        let auth_request = JsonRpcRequest {
            id: self.next_id(),
            msg: "method".to_string(),
            method: "auth.login_with_api_key".to_string(),
            params: Some(serde_json::json!([self
                .config
                .api_key
                .expose_secret()
                .trim()])),
        };

        let auth_json = serde_json::to_string(&auth_request)?;
        debug!("Sending auth request");
        conn.stream
            .send(Message::Text(auth_json.into()))
            .await
            .map_err(ExporterError::WebSocket)?;

        // Read auth response
        if let Some(msg) = conn.stream.next().await {
            let msg = msg.map_err(ExporterError::WebSocket)?;
            if let Message::Text(text) = msg {
                debug!("Auth response: {}", text);
                let response: JsonRpcResponse = serde_json::from_str(&text)?;

                // Check for errors
                if let Some(error) = response.error {
                    let error_msg = error.reason.unwrap_or_else(|| "Unknown error".to_string());
                    return Err(ExporterError::Auth(format!(
                        "Authentication failed: {}",
                        error_msg
                    )));
                }

                // Check result
                if let Some(result) = response.result {
                    if result == serde_json::Value::Bool(true) {
                        return Ok(());
                    } else if result == serde_json::Value::Bool(false) {
                        return Err(ExporterError::Auth(
                            "Authentication failed: API key rejected by TrueNAS".to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a query on the persistent connection
    pub async fn execute_query<T>(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Ensure we're connected and authenticated
        self.ensure_connected().await?;

        let mut conn_guard = self.connection.lock().await;
        // Take ownership of the connection (temporarily remove from mutex)
        let mut conn = conn_guard
            .take()
            .ok_or_else(|| ExporterError::Config("No active connection".to_string()))?;

        // Send request
        let request = JsonRpcRequest {
            id: self.next_id(),
            msg: "method".to_string(),
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)?;
        debug!("Sending request: {}", method);

        if let Err(e) = conn.stream.send(Message::Text(request_json.into())).await {
            // Connection failed, do not put it back (it remains None)
            return Err(ExporterError::WebSocket(e));
        }

        // Read response
        let response_msg = conn.stream.next().await;

        // Put connection back immediately if we got a response (IO is okay)
        // If response is None, it means stream closed, so we don't put it back
        let msg = match response_msg {
            Some(Ok(msg)) => {
                *conn_guard = Some(conn);
                msg
            }
            Some(Err(e)) => return Err(ExporterError::WebSocket(e)),
            None => {
                return Err(ExporterError::TrueNasApi(
                    "Connection closed by server".to_string(),
                ))
            }
        };

        if let Message::Text(text) = msg {
            debug!("{} response received", method);
            let response: JsonRpcResponse = serde_json::from_str(&text)?;

            // Check for errors
            if let Some(error) = response.error {
                let error_msg = error.reason.unwrap_or_else(|| "Unknown error".to_string());

                // If not authenticated, clear connection to force re-auth
                if error_msg.contains("ENOTAUTHENTICATED") {
                    warn!("Session expired, will re-authenticate on next request");
                    *conn_guard = None;
                }

                return Err(ExporterError::TrueNasApi(error_msg));
            }

            // Parse result
            if let Some(result) = response.result {
                return serde_json::from_value(result).map_err(ExporterError::Json);
            }
        }

        Err(ExporterError::TrueNasApi(
            "No valid JSON-RPC response received".to_string(),
        ))
    }

    /// Close the connection
    pub async fn close(&self) {
        let mut conn_guard = self.connection.lock().await;
        if let Some(mut conn) = conn_guard.take() {
            let _ = conn.stream.close(None).await;
            info!("WebSocket connection closed");
        }
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        // Connection will be closed when the stream is dropped
        debug!("ConnectionManager dropped");
    }
}

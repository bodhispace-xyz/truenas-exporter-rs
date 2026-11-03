use anyhow::{Context, Result};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub truenas: TrueNasConfig,
    pub server: ServerConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TrueNasConfig {
    pub host: String,
    pub api_key: SecretString,
    #[serde(default = "default_use_tls")]
    pub use_tls: bool,
    #[serde(default = "default_verify_ssl")]
    pub verify_ssl: bool,
    /// Optional additional trusted PEM CA or self-signed server certificate.
    #[serde(default)]
    pub ca_cert_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_addr")]
    pub addr: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    #[serde(default = "default_scrape_interval")]
    pub scrape_interval_seconds: u64,
    #[serde(default = "default_true")]
    pub collect_pool_metrics: bool,
    #[serde(default = "default_true")]
    pub collect_system_metrics: bool,
}

fn default_addr() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    9100
}

fn default_use_tls() -> bool {
    true
}

fn default_verify_ssl() -> bool {
    true
}

fn default_scrape_interval() -> u64 {
    60
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        // Load environment variables from .env if present
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::File::with_name(path).required(false))
            .add_source(config::Environment::with_prefix("TRUENAS_EXPORTER").separator("__"))
            .build()
            .context("Failed to build configuration")?;

        let settings: Self = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;
        // Fail startup before binding if a selected trust configuration is unusable.
        settings.truenas.tls_connector()?;
        Ok(settings)
    }
}

impl TrueNasConfig {
    pub(crate) fn tls_connector(&self) -> Result<Option<native_tls::TlsConnector>> {
        if !self.use_tls {
            anyhow::ensure!(
                !self.verify_ssl,
                "Plain WebSocket transport requires explicit verify_ssl=false"
            );
            anyhow::ensure!(
                self.ca_cert_path.is_none(),
                "A CA certificate requires verified TLS"
            );
            return Ok(None);
        }
        let mut builder = native_tls::TlsConnector::builder();
        builder
            .danger_accept_invalid_certs(!self.verify_ssl)
            .danger_accept_invalid_hostnames(!self.verify_ssl);
        if let Some(path) = &self.ca_cert_path {
            anyhow::ensure!(self.verify_ssl, "A CA certificate requires verify_ssl=true");
            let pem = std::fs::read(path)
                .with_context(|| format!("Cannot read TrueNAS CA certificate: {path}"))?;
            let marker = b"-----BEGIN CERTIFICATE-----";
            anyhow::ensure!(
                pem.windows(marker.len()).filter(|w| *w == marker).count() == 1,
                "TrueNAS CA file must contain exactly one PEM certificate"
            );
            let certificate = native_tls::Certificate::from_pem(&pem)
                .context("Invalid TrueNAS PEM CA certificate")?;
            builder.add_root_certificate(certificate);
        }
        Ok(Some(
            builder
                .build()
                .context("Cannot build TrueNAS TLS connector")?,
        ))
    }
}

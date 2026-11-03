use anyhow::Result;
use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use truenas_exporter::{config::Config, server};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/Default.toml")]
    config: String,

    /// TrueNAS host (overrides config)
    #[arg(long, env = "TRUENAS_HOST")]
    truenas_host: Option<String>,

    /// TrueNAS API key (overrides config)
    #[arg(long, env = "TRUENAS_API_KEY")]
    truenas_api_key: Option<String>,

    /// Port to listen on for metrics
    #[arg(short, long, env = "EXPORTER_PORT")]
    port: Option<u16>,

    /// Address to bind to
    #[arg(short, long, env = "EXPORTER_ADDR")]
    addr: Option<String>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
    // Load dotenv before clap reads its environment aliases. Config::load also
    // supports dotenv for library callers; neither load overwrites process variables.
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Starting TrueNAS Prometheus Exporter v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Parse CLI arguments
    let args = Args::parse();

    // Load configuration
    let mut config = Config::load(&args.config)?;

    // Override with CLI arguments if provided
    if let Some(host) = args.truenas_host {
        config.truenas.host = host;
    }
    if let Some(api_key) = args.truenas_api_key {
        config.truenas.api_key = secrecy::SecretString::new(api_key.into());
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }
    if let Some(addr) = args.addr {
        config.server.addr = addr;
    }

    info!("Configuration loaded successfully");
    info!("TrueNAS host: {}", config.truenas.host);
    info!(
        "Metrics endpoint: http://{}:{}/metrics",
        config.server.addr, config.server.port
    );

    // Start the metrics server
    if let Err(e) = server::start(config).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

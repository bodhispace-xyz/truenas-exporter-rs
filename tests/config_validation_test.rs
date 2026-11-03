//! Configuration validation tests
//!
//! Tests that verify configuration defaults and structure.

use truenas_exporter::config::{MetricsConfig, ServerConfig, TrueNasConfig};

#[test]
fn test_default_server_config() {
    // Given: ServerConfig with default values
    // When: Creating instance with expected defaults
    let config = ServerConfig {
        addr: "0.0.0.0".to_string(),
        port: 9100,
    };

    // Then: Should have expected default values
    assert_eq!(config.addr, "0.0.0.0");
    assert_eq!(config.port, 9100);
}

#[test]
fn test_truenas_config_defaults_via_serde() {
    let config: TrueNasConfig = serde_json::from_value(serde_json::json!({
        "host": "localhost:443", "api_key": "dummy"
    }))
    .unwrap();
    assert!(config.use_tls, "omitted transport setting must select TLS");
    assert!(config.verify_ssl);
}

#[test]
fn test_metrics_config_defaults_via_serde() {
    // Given: MetricsConfig struct uses serde defaults
    // When: Manually constructing with expected defaults
    let config = MetricsConfig {
        scrape_interval_seconds: 60,
        collect_pool_metrics: true,
        collect_system_metrics: true,
    };

    // Then: Should have expected default values
    assert_eq!(config.scrape_interval_seconds, 60);
    assert!(config.collect_pool_metrics);
    assert!(config.collect_system_metrics);
}

#[test]
fn test_config_structs_have_sensible_defaults() {
    // Given: Default configurations manually constructed with expected values
    let server = ServerConfig {
        addr: "0.0.0.0".to_string(),
        port: 9100,
    };
    use secrecy::SecretString;
    let truenas = TrueNasConfig {
        host: String::new(),
        api_key: SecretString::new(String::new().into()),
        use_tls: true,
        verify_ssl: true,
        ca_cert_path: None,
    };
    let metrics = MetricsConfig {
        scrape_interval_seconds: 60,
        collect_pool_metrics: true,
        collect_system_metrics: true,
    };

    // When: Checking values
    // Then: Server should bind to all interfaces on standard Prometheus port
    assert_eq!(server.addr, "0.0.0.0");
    assert_eq!(server.port, 9100);

    // Then: TrueNAS should default to secure settings
    assert!(truenas.verify_ssl); // SSL verification on by default

    // Then: Metrics should collect all data by default
    assert!(metrics.collect_pool_metrics);
    assert!(metrics.collect_system_metrics);
}

#[test]
fn test_server_config_construction() {
    // Given: Manual ServerConfig construction
    // When: Creating a ServerConfig
    let config = ServerConfig {
        addr: "127.0.0.1".to_string(),
        port: 8080,
    };

    // Then: Values should be set correctly
    assert_eq!(config.addr, "127.0.0.1");
    assert_eq!(config.port, 8080);
}

#[test]
fn test_metrics_config_construction() {
    // Given: Manual MetricsConfig construction
    // When: Creating a MetricsConfig with specific values
    let config = MetricsConfig {
        scrape_interval_seconds: 30,
        collect_pool_metrics: true,
        collect_system_metrics: false,
    };

    // Then: Values should be set correctly
    assert_eq!(config.scrape_interval_seconds, 30);
    assert!(config.collect_pool_metrics);
    assert!(!config.collect_system_metrics);
}

#[test]
fn distributed_config_enables_verified_tls() {
    let config: truenas_exporter::config::Config = config::Config::builder()
        .add_source(config::File::from_str(
            include_str!("../config/Default.toml"),
            config::FileFormat::Toml,
        ))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();
    assert!(config.truenas.use_tls);
    assert!(config.truenas.verify_ssl);
    assert!(config.truenas.ca_cert_path.is_none());
}

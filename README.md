# TrueNAS Scale Prometheus Exporter

A robust Prometheus metrics exporter for TrueNAS Scale 25.x and newer, written in Rust. Connects to the TrueNAS WebSocket API to collect comprehensive metrics for ZFS pools, datasets, sharing services, data protection tasks, boot pool health, NFS clients, and system health.

## Features

- **WebSocket API**: TrueNAS Scale 25.x JSON-RPC 2.0 with persistent connection and auto-reconnect.
- **Storage**: ZFS pools, datasets, VDevs, physical disks (SMART/temp/IO).
- **Boot Pool**: Health, used ratio, scrub errors, last scrub timestamp.
- **ZFS ARC**: Current ARC cache size over time.
- **Data Protection**: Cloud sync (status/progress) and snapshot task monitoring.
- **NFS Clients**: Total count + per-client detail (address, name, status, seconds since renew) for NFSv3 and NFSv4.
- **Services & Apps**: SMB/NFS/SSH service status, app status and update availability.
- **System Health**: Alerts, CPU/memory/network stats, uptime.
- **Performance**: Async design using Tokio, zero-copy metrics with Arc.

## Quick Start

### Prerequisites

- TrueNAS Scale 25.x or later
- API key from TrueNAS (generate under "My API Keys" in the UI)
- Docker and Docker Compose (recommended)

### Using Docker Compose

1. **Clone the repository**:

   ```bash
   git clone https://github.com/yourusername/truenas-exporter-rs.git
   cd truenas-exporter-rs
   ```

2. **Create `.env` file**:

   ```bash
   cp .env.example .env
   ```

3. **Edit `.env` with your TrueNAS details**:

   ```env
   # TrueNAS Connection
   TRUENAS_EXPORTER__TRUENAS__HOST=YOUR_TRUENAS_IP:443
   TRUENAS_EXPORTER__TRUENAS__API_KEY=your-api-key-here
   
   # Security
   TRUENAS_EXPORTER__TRUENAS__USE_TLS=true
   TRUENAS_EXPORTER__TRUENAS__VERIFY_SSL=true
   
   # Logging
   RUST_LOG=info
   ```

4. **Start the exporter**:

   ```bash
   docker-compose up -d
   ```

5. **Verify it's working**:

   ```bash
   curl http://localhost:9100/metrics
   ```

## Configuration

### Configuration File (config/Default.toml)

```toml
[truenas]
host = "YOUR_TRUENAS_IP:443"
api_key = "your-api-key-here"
use_tls = true
verify_ssl = true
# ca_cert_path = "/etc/truenas-exporter/truenas-ca.pem"  # Optional trusted PEM certificate

[server]
addr = "0.0.0.0"
port = 9100

[metrics]
scrape_interval_seconds = 60
collect_pool_metrics = true
collect_system_metrics = true
```

### Listener configuration precedence

The exporter keeps the address and port loaded from TOML or the documented
`TRUENAS_EXPORTER__SERVER__ADDR` / `TRUENAS_EXPORTER__SERVER__PORT` variables.
Only an explicit `--addr` / `--port` or the compatibility variables
`EXPORTER_ADDR` / `EXPORTER_PORT` override them. CLI arguments take precedence over
their compatibility variables. The defaults remain `0.0.0.0:9100` when no loaded
setting or explicit override supplies a value.

The process loads `.env` before reading CLI environment aliases. Existing process
variables take precedence over values in `.env`.

### Private and self-signed certificates

TLS and certificate/hostname verification are enabled by default. For a private CA
or self-signed TrueNAS certificate, obtain the public PEM certificate through a
trusted administration channel and verify its fingerprint. Keep verification enabled:

```toml
[truenas]
host = "truenas.example.com:443"
api_key = "your-api-key-here"
use_tls = true
verify_ssl = true
ca_cert_path = "/etc/truenas-exporter/truenas-ca.pem"
```

Or set `TRUENAS_EXPORTER__TRUENAS__CA_CERT_PATH` to that path. The file must contain
exactly one PEM CA or self-signed server certificate. For a certificate chain,
configure its trusted CA here and ensure TrueNAS serves any required intermediate
certificates. The hostname in `host` must match the certificate's subject alternative
names; trusting a CA does not disable hostname checks.

For Docker Compose, mount the public certificate read-only and configure its
container path:

```yaml
volumes:
  - ./truenas-ca.pem:/etc/truenas-exporter/truenas-ca.pem:ro
environment:
  TRUENAS_EXPORTER__TRUENAS__USE_TLS: "true"
  TRUENAS_EXPORTER__TRUENAS__VERIFY_SSL: "true"
  TRUENAS_EXPORTER__TRUENAS__CA_CERT_PATH: /etc/truenas-exporter/truenas-ca.pem
```

Missing, malformed, empty or multiple-certificate CA files fail startup. Certificate
failures do not fall back to insecure TLS. A CA path combined with disabled TLS or
verification is rejected, so a configured trust certificate cannot be silently ignored.

For temporary testing only, `verify_ssl = false` explicitly disables peer
verification and emits a warning. Plain WebSocket transport additionally requires
`use_tls = false`; both insecure settings must be explicit. Remove `ca_cert_path`
when opting out. These settings expose API-key credentials to peer impersonation;
plain WebSocket also exposes them without encryption.

**Upgrade note:** configurations that omitted `use_tls` now use TLS. Private or
self-signed installations should configure their trusted certificate before upgrading.
Existing explicit `verify_ssl = false` values remain an insecure opt-out; update them
to `true` to enable verification. Direct Rust `TrueNasConfig` literals need the new
`ca_cert_path` field (`None` for normal system certificate trust). Configured listener
addresses and ports are now respected instead of being replaced by CLI defaults.
Metric names, labels and NAS query methods are unchanged.

## Authentication & Connection Details

TrueNAS Scale 25.04+ (Electric Eel) has deprecated the REST API in favor of a WebSocket-only architecture. This exporter implements a robust, persistent connection model to handle this correctly.

### 1. Persistent Connection

Unlike traditional REST-based exporters, this exporter maintains a **single, long-lived WebSocket connection** to TrueNAS. It does **not** reconnect for every scrape.

- **Why?** TrueNAS aggressively rate-limits or rejects clients that attempt to authenticate too frequently (e.g., once per scrape).
- **Behavior:** The exporter connects and authenticates *once* at startup. If the connection drops, it automatically attempts to reconnect with exponential backoff.

### 2. Troubleshooting Authentication

If you see `truenas_up 0` and logs showing "Authentication failed", check the following:

#### `RuntimeError: AUTH: unexpected authenticator run state`

If you see this error in the logs (or server-side), it means the **server-side session state for your API key is corrupted**. This often happens if a client previously hammered the endpoint with rapid connection attempts.
**Fix:**

1. Log into TrueNAS UI.
2. Go to **My API Keys**.
3. **Delete** the problematic API key.
4. **Generate a NEW API key**.
5. Update your `.env` or `config.toml` with the new key.
6. Restart the exporter.

#### `[ENOTAUTHENTICATED]`

This is a standard failure. Check that:

- You pasted the API key correctly.
- The user associated with the key has permission to access the API.
- You are using `wss://` (TLS) if your server requires it (default for API keys).

## Metrics Exposed

### 1. Storage (ZFS & Disks)

- `truenas_pool_health`, `truenas_pool_capacity_bytes`, `truenas_pool_allocated_bytes`, `truenas_pool_free_bytes`
- `truenas_pool_last_scrub_seconds`, `truenas_pool_scrub_errors`
- `truenas_pool_vdev_error_count` — labels: `pool`, `vdev`, `type` (read/write/checksum)
- `truenas_dataset_used_bytes`, `truenas_dataset_available_bytes`, `truenas_dataset_compression_ratio`, `truenas_dataset_encrypted`
- `truenas_disk_temperature_celsius`, `truenas_disk_read_bytes_per_second`, `truenas_disk_write_bytes_per_second`
- `truenas_disk_info`, `truenas_disk_power_on_hours`
- `truenas_smart_test_status`, `truenas_smart_test_lifetime_hours`, `truenas_smart_test_timestamp_seconds`

### 2. Boot Pool

- `truenas_boot_pool_health` — 1=healthy, 0=degraded
- `truenas_boot_pool_used_ratio` — allocated/total (0.0–1.0)
- `truenas_boot_pool_scrub_errors`
- `truenas_boot_pool_last_scrub_seconds` — Unix timestamp of last completed scrub

### 3. ZFS ARC

- `truenas_zfs_arc_size_bytes` — current ARC cache size

### 4. NFS Clients

- `truenas_nfs_client_count` — total active NFS sessions
- `truenas_nfs_client_info` — labels: `address`, `name`, `version` (3/4), `status`
- `truenas_nfs_client_seconds_since_renew` — lease age per client

### 5. Data Protection

- `truenas_cloud_sync_status` — labels: `job`, `direction`, `transfer_mode`
- `truenas_cloud_sync_progress` — percent complete for running jobs
- `truenas_snapshot_task_status` — labels: `dataset`, `naming_schema`

### 6. Services & Apps

- `truenas_share_smb_enabled`, `truenas_share_nfs_enabled` — labels: `name`/`path`
- `truenas_service_status` — labels: `service` (0=stopped, 1=running)
- `truenas_app_status`, `truenas_app_update_available` — labels: `app`

### 7. System Health

- `truenas_up` — 1 if TrueNAS API reachable
- `truenas_alert_count`, `truenas_alert_info` — labels: `level`, `source`
- `truenas_system_cpu_usage_percent` — labels: `mode` (per-core)
- `truenas_system_cpu_temperature_celsius` — labels: `cpu`
- `truenas_system_memory_bytes{state="available"}`, `truenas_system_memory_total_bytes`, `truenas_system_memory_used_bytes`
- `truenas_network_receive_bytes_per_second`, `truenas_network_transmit_bytes_per_second` — labels: `interface`
- `truenas_system_uptime_seconds`

## Limitations / Known Issues

- **Per-App CPU/Memory**: `app.stats` is a WebSocket subscription (streaming), not a one-shot call — not collectible with the current pull-based model.
- **SMART Attributes**: Only Pass/Fail + lifetime hours. Detailed normalized attributes (e.g. wear level) vary by vendor and are not exposed.
- **TrueNAS 26 compatibility**: REST API is removed in TrueNAS 26 (not an issue — exporter uses WebSocket). However, OpenZFS 2.4 may rename reporting legend fields. See [issue #36](https://github.com/mlgruby/truenas-exporter-rs/issues/36) for tracking.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

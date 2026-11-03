//! Exercise the production CLI in child processes with an empty environment and dummy config.
use std::{
    io::{BufRead, BufReader},
    io::{Read, Write},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
use tempfile::TempDir;

struct Exporter {
    child: Child,
    lines: mpsc::Receiver<String>,
    _directory: TempDir,
}
impl Drop for Exporter {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Exporter {
    fn start(addr: &str, port: u16, args: &[&str], env: &[(&str, &str)]) -> Self {
        Self::start_with_dotenv(addr, port, args, env, None)
    }
    fn start_with_dotenv(
        addr: &str,
        port: u16,
        args: &[&str],
        env: &[(&str, &str)],
        dotenv: Option<&str>,
    ) -> Self {
        let directory = tempfile::tempdir().unwrap();
        // Held private loopback fixture: no shared service or deployed NAS is contacted.
        let upstream = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let host = upstream.local_addr().unwrap();
        let config = format!(
            r#"
[truenas]
host = "{host}"
api_key = "dummy"
use_tls = false
verify_ssl = false
[server]
addr = "{addr}"
port = {port}
[metrics]
scrape_interval_seconds = 60
"#
        );
        std::fs::write(directory.path().join("config.toml"), config).unwrap();
        if let Some(contents) = dotenv {
            std::fs::write(directory.path().join(".env"), contents).unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_truenas-exporter"));
        command
            .env_clear()
            .env("RUST_LOG", "info")
            .envs(env.iter().copied())
            .current_dir(directory.path())
            .args(["--config", "config.toml"])
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command.spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, lines) = mpsc::channel();
        std::thread::spawn(move || {
            // Keep the owned dummy listener alive until child stdout closes.
            let _upstream = upstream;
            for line in BufReader::new(stdout)
                .lines()
                .map_while(Result::ok)
                .take(100)
            {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            lines,
            _directory: directory,
        }
    }
    fn wait_for_exit(&mut self) -> std::process::ExitStatus {
        let until = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < until, "child did not terminate");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn wait_for(&mut self, marker: &str) -> String {
        let until = Instant::now() + Duration::from_secs(10);
        loop {
            let line = self
                .lines
                .recv_timeout(until.saturating_duration_since(Instant::now()))
                .expect("child did not report expected startup state");
            if line.contains(marker) {
                return line;
            }
        }
    }
}

#[test]
fn loaded_loopback_address_survives_absent_cli_override() {
    // Explicit ephemeral port keeps the pre-fix reproduction away from fixed host ports.
    let mut exporter = Exporter::start("127.0.0.1", 0, &["--port", "0"], &[]);
    let line = exporter.wait_for("Metrics endpoint:");
    assert!(line.contains("http://127.0.0.1:0/metrics"), "{line}");
}

#[test]
fn loaded_port_survives_absent_cli_override() {
    let mut exporter = Exporter::start("127.0.0.1", 0, &["--addr", "127.0.0.1"], &[]);
    let line = exporter.wait_for("Metrics endpoint:");
    assert!(line.contains("http://127.0.0.1:0/metrics"), "{line}");
}

#[test]
fn prefixed_environment_preserves_loopback_address() {
    let mut exporter = Exporter::start(
        "0.0.0.0",
        0,
        &[],
        &[("TRUENAS_EXPORTER__SERVER__ADDR", "127.0.0.1")],
    );
    assert!(exporter
        .wait_for("Metrics endpoint:")
        .contains("http://127.0.0.1:0/metrics"));
}

#[test]
fn explicit_cli_overrides_loaded_configuration() {
    let mut exporter = Exporter::start(
        "0.0.0.0",
        9101,
        &["--addr", "127.0.0.1", "--port", "0"],
        &[],
    );
    assert!(exporter
        .wait_for("Metrics endpoint:")
        .contains("http://127.0.0.1:0/metrics"));
}

#[test]
fn legacy_environment_aliases_work_from_dotenv() {
    let mut exporter = Exporter::start_with_dotenv(
        "0.0.0.0",
        9101,
        &[],
        &[],
        Some("EXPORTER_ADDR=127.0.0.1\nEXPORTER_PORT=0\n"),
    );
    assert!(exporter
        .wait_for("Metrics endpoint:")
        .contains("http://127.0.0.1:0/metrics"));
}

#[test]
fn configured_loopback_listener_serves_metrics_on_its_actual_socket() {
    let mut exporter = Exporter::start("127.0.0.1", 0, &[], &[]);
    let line = exporter.wait_for("Metrics server listening on ");
    let address: std::net::SocketAddr = line
        .split("Metrics server listening on ")
        .nth(1)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(
        address.ip(),
        "127.0.0.1".parse::<std::net::IpAddr>().unwrap()
    );
    assert_ne!(address.port(), 0);
    let mut stream =
        std::net::TcpStream::connect_timeout(&address, Duration::from_secs(2)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream
        .write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.contains("truenas_up"));
}

#[test]
fn invalid_ca_configuration_fails_startup_before_binding() {
    let mut exporter = Exporter::start(
        "127.0.0.1",
        0,
        &[],
        &[
            ("TRUENAS_EXPORTER__TRUENAS__USE_TLS", "true"),
            ("TRUENAS_EXPORTER__TRUENAS__VERIFY_SSL", "true"),
            ("TRUENAS_EXPORTER__TRUENAS__CA_CERT_PATH", "missing.pem"),
        ],
    );
    assert!(!exporter.wait_for_exit().success());
    let lines: Vec<_> = exporter.lines.try_iter().collect();
    assert!(!lines
        .iter()
        .any(|line| line.contains("Metrics server listening on")));
}

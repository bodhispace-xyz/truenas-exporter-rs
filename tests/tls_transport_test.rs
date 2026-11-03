//! Generated certificates, dummy keys and owned ephemeral loopback services only.
use futures_util::{SinkExt, StreamExt};
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use std::{
    io::Write,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tempfile::NamedTempFile;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    task::{JoinHandle, JoinSet},
};
use tokio_rustls::{
    rustls::{self, pki_types::PrivatePkcs8KeyDer},
    TlsAcceptor,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use truenas_exporter::{
    config::TrueNasConfig,
    error::ExporterError,
    truenas::{types::Pool, TrueNasClient},
};

struct Fixture {
    host: String,
    pem: String,
    auth_received: Arc<AtomicUsize>,
    task: JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve_websocket<S: AsyncRead + AsyncWrite + Unpin>(stream: S, count: Arc<AtomicUsize>) {
    let Ok(mut socket) = accept_async(stream).await else {
        return;
    };
    let Some(Ok(Message::Text(connect))) = socket.next().await else {
        return;
    };
    let connect: serde_json::Value = serde_json::from_str(&connect).unwrap();
    assert_eq!(connect["msg"], "connect");
    socket
        .send(Message::Text(
            r#"{"msg":"connected","session":"dummy"}"#.into(),
        ))
        .await
        .unwrap();
    let Some(Ok(Message::Text(auth))) = socket.next().await else {
        return;
    };
    let auth: serde_json::Value = serde_json::from_str(&auth).unwrap();
    assert_eq!(auth["method"], "auth.login_with_api_key");
    assert_eq!(auth["params"][0], "dummy-key");
    count.fetch_add(1, Ordering::SeqCst);
    socket
        .send(Message::Text(
            serde_json::json!({"id":auth["id"],"msg":"result","result":true})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    while let Some(Ok(Message::Text(query))) = socket.next().await {
        let query: serde_json::Value = serde_json::from_str(&query).unwrap();
        assert_eq!(query["method"], "pool.query");
        socket
            .send(Message::Text(
                serde_json::json!({"id":query["id"],"msg":"result","result":[]})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
    }
}

async fn fixture(tls: bool, self_signed: bool) -> Fixture {
    let (acceptor, pem) = if tls {
        let (chain, key, pem) = if self_signed {
            let certified = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
            (
                vec![certified.cert.der().clone()],
                certified.signing_key.serialize_der(),
                certified.cert.pem(),
            )
        } else {
            let mut params = CertificateParams::default();
            params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
            params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
            params.distinguished_name = DistinguishedName::new();
            params
                .distinguished_name
                .push(DnType::CommonName, "Dummy private CA");
            let issuer =
                CertifiedIssuer::self_signed(params, KeyPair::generate().unwrap()).unwrap();
            let key = KeyPair::generate().unwrap();
            let mut server = CertificateParams::new(vec!["localhost".into()]).unwrap();
            server.distinguished_name = DistinguishedName::new();
            server
                .distinguished_name
                .push(DnType::CommonName, "localhost");
            server.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
            server.key_usages = vec![KeyUsagePurpose::DigitalSignature];
            let server = server.signed_by(&key, &issuer).unwrap();
            (
                vec![server.der().clone(), issuer.der().clone()],
                key.serialize_der(),
                issuer.pem(),
            )
        };
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(chain, PrivatePkcs8KeyDer::from(key).into())
        .unwrap();
        (Some(TlsAcceptor::from(Arc::new(config))), pem)
    } else {
        (None, String::new())
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let host = format!("localhost:{}", listener.local_addr().unwrap().port());
    let auth_received = Arc::new(AtomicUsize::new(0));
    let count = auth_received.clone();
    let task = tokio::spawn(async move {
        let mut connections = JoinSet::new();
        while let Ok((stream, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let count = count.clone();
            connections.spawn(async move {
                let _ = tokio::time::timeout(Duration::from_secs(6), async move {
                    if let Some(acceptor) = acceptor {
                        if let Ok(stream) = acceptor.accept(stream).await {
                            serve_websocket(stream, count).await;
                        }
                    } else {
                        serve_websocket(stream, count).await;
                    }
                })
                .await;
            });
            while connections.try_join_next().is_some() {}
        }
    });
    Fixture {
        host,
        pem,
        auth_received,
        task,
    }
}
fn config(host: &str) -> TrueNasConfig {
    // Exercise actual serde defaults, rather than constructing expected defaults by hand.
    serde_json::from_value(serde_json::json!({"host":host,"api_key":"dummy-key"})).unwrap()
}
fn ca_file(pem: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(pem.as_bytes()).unwrap();
    file
}
async fn request(config: TrueNasConfig) -> Result<Vec<Pool>, ExporterError> {
    tokio::time::timeout(
        Duration::from_secs(8),
        TrueNasClient::new(config).query_pools(),
    )
    .await
    .expect("bounded fixture timed out")
}
fn assert_config_error(result: Result<Vec<Pool>, ExporterError>) {
    assert!(
        matches!(result, Err(ExporterError::Config(_))),
        "{result:?}"
    );
}

#[tokio::test]
async fn defaults_reject_untrusted_peer_before_api_key() {
    let server = fixture(true, false).await;
    assert_config_error(request(config(&server.host)).await);
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn private_ca_allows_authentication_and_connection_reuse() {
    let server = fixture(true, false).await;
    let ca = ca_file(&server.pem);
    let mut config = config(&server.host);
    config.ca_cert_path = Some(ca.path().to_str().unwrap().into());
    let client = TrueNasClient::new(config);
    tokio::time::timeout(Duration::from_secs(8), async {
        assert!(client.query_pools().await.unwrap().is_empty());
        assert!(client.query_pools().await.unwrap().is_empty());
    })
    .await
    .unwrap();
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn trusted_self_signed_server_certificate_is_supported() {
    let server = fixture(true, true).await;
    let ca = ca_file(&server.pem);
    let mut config = config(&server.host);
    config.ca_cert_path = Some(ca.path().to_str().unwrap().into());
    assert!(request(config).await.unwrap().is_empty());
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn private_ca_does_not_disable_hostname_verification() {
    let server = fixture(true, false).await;
    let ca = ca_file(&server.pem);
    let mut config = config(&server.host.replace("localhost", "127.0.0.1"));
    config.ca_cert_path = Some(ca.path().to_str().unwrap().into());
    assert_config_error(request(config).await);
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn unrelated_ca_cannot_authenticate_peer() {
    let server = fixture(true, false).await;
    let unrelated = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let ca = ca_file(&unrelated.cert.pem());
    let mut config = config(&server.host);
    config.ca_cert_path = Some(ca.path().to_str().unwrap().into());
    assert_config_error(request(config).await);
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn insecure_tls_remains_an_explicit_opt_out() {
    let server = fixture(true, false).await;
    let mut config = config(&server.host);
    config.verify_ssl = false;
    assert!(request(config).await.unwrap().is_empty());
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn plaintext_requires_both_explicit_insecure_settings() {
    let server = fixture(false, false).await;
    let mut config = config(&server.host);
    config.use_tls = false;
    assert_config_error(request(config.clone()).await);
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 0);
    config.verify_ssl = false;
    assert!(request(config).await.unwrap().is_empty());
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn unusable_ca_fails_before_authentication_without_fallback() {
    let server = fixture(true, false).await;
    let empty = ca_file("");
    let malformed = ca_file("-----BEGIN CERTIFICATE-----\ninvalid\n-----END CERTIFICATE-----");
    let multiple = ca_file(&format!("{}{}", server.pem, server.pem));
    for path in [
        empty.path().to_owned(),
        malformed.path().to_owned(),
        multiple.path().to_owned(),
        empty.path().with_extension("missing"),
    ] {
        let mut config = config(&server.host);
        config.ca_cert_path = Some(path.to_str().unwrap().into());
        assert_config_error(request(config).await);
    }
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn ca_cannot_be_silently_ignored_by_insecure_settings() {
    let server = fixture(true, false).await;
    let ca = ca_file(&server.pem);
    for use_tls in [true, false] {
        let mut config = config(&server.host);
        config.use_tls = use_tls;
        config.verify_ssl = false;
        config.ca_cert_path = Some(ca.path().to_str().unwrap().into());
        assert_config_error(request(config).await);
    }
    assert_eq!(server.auth_received.load(Ordering::SeqCst), 0);
}

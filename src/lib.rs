use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::{
    client::TlsStream,
    rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName},
    TlsConnector,
};
use url::Url;

pub trait ProxyStream: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static {}

pub async fn connect(proxy_url: &str, host: &str, port: u16) -> Result<Box<dyn ProxyStream>> {
    let proxy_url = Url::parse(proxy_url).with_context(|| "invalid proxy url")?;
    let scheme = proxy_url.scheme();
    let mut stream: Box<dyn ProxyStream> = match scheme {
        "https" => {
            let mut cert_store = RootCertStore::empty();
            let trust_anchors = webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|trust_anchor| {
                OwnedTrustAnchor::from_subject_spki_name_constraints(
                    trust_anchor.subject,
                    trust_anchor.spki,
                    trust_anchor.name_constraints,
                )
            });

            cert_store.add_server_trust_anchors(trust_anchors);
            let config: ClientConfig = ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(cert_store)
                .with_no_client_auth()
                .into();
            let connector = TlsConnector::from(Arc::new(config));
            let server_name = ServerName::try_from(proxy_url.host_str().unwrap_or_default())
                .with_context(|| "invalid host name")?;

            let stream = TcpStream::connect(&format!(
                "{}:{}",
                proxy_url.host_str().unwrap_or_default(),
                proxy_url.port().unwrap_or(443)
            ))
            .await?;
            let stream = connector.connect(server_name, stream).await?;
            Box::new(stream)
        }
        "http" => {
            let stream = TcpStream::connect(&format!(
                "{}:{}",
                proxy_url.host_str().unwrap_or_default(),
                proxy_url.port().unwrap_or(80)
            ))
            .await?;
            Box::new(stream)
        }
        _ => anyhow::bail!("unsupported proxy scheme: {}", scheme),
    };

    async_http_proxy::http_connect_tokio(&mut stream, host, port)
        .await
        .with_context(|| "proxy connect failed")?;

    Ok(stream)
}

impl ProxyStream for TlsStream<TcpStream> {}

impl ProxyStream for TcpStream {}

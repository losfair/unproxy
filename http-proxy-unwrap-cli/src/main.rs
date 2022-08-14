use anyhow::{Context, Result};
use structopt::StructOpt;
use tokio::net::lookup_host;
use tracing_subscriber::{fmt::SubscriberBuilder, EnvFilter};

#[derive(Debug, StructOpt)]
#[structopt(name = "http-proxy-unwrap-cli", about = "HTTP Proxy Unwrap")]
struct Opt {
    /// HTTP/HTTPS URL of the proxy to use. The path is ignored.
    #[structopt(long, short = "p")]
    proxy: String,

    /// Remote address.
    #[structopt(long, short = "r")]
    remote: String,

    /// Local address.
    #[structopt(long, short = "l")]
    local: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    SubscriberBuilder::default()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .json()
        .init();
    let opt = Opt::from_args();
    let remote = lookup_host(&opt.remote)
        .await
        .with_context(|| "failed to resolve remote address")?
        .next()
        .unwrap();
    let local = lookup_host(&opt.local)
        .await
        .with_context(|| "failed to resolve local address")?
        .next()
        .unwrap();

    let listener = tokio::net::TcpListener::bind(&local)
        .await
        .with_context(|| "failed to bind local address")?;
    loop {
        let (mut conn, local_client_addr) = listener
            .accept()
            .await
            .with_context(|| "failed to accept connection")?;
        let proxy = opt.proxy.clone();
        tracing::info!(client = %local_client_addr, "new connection");
        tokio::spawn(async move {
            match http_proxy_unwrap::connect(&proxy, &remote.ip().to_string(), remote.port()).await
            {
                Ok(mut proxy_stream) => {
                    let _ = tokio::io::copy_bidirectional(&mut proxy_stream, &mut conn).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to connect to remote");
                }
            };
        });
    }
}

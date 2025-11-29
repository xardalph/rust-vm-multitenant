mod agent;
mod container;

use anyhow::Result;
use clap::Parser;
use tracing::error;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::agent::Agent;

#[derive(Debug, Parser)]
pub struct Opts {
    /// API URL.
    #[arg(
        short = 'u',
        long = "url",
        env = "API_URL",
        default_value = "https://victoria-monitoring.evan.ovh"
    )]
    url: String,

    /// API key.
    #[arg(short = 'a', long = "apikey", env = "API_KEY")]
    apikey: String,

    /// Regex to exclude containers.
    #[arg(short = 'e', long = "exclude", env = "EXCLUDE_CONTAINER_STATE")]
    exclude: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_default(),
        ))
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;

    let opts = Opts::parse();

    let agent = Agent::start(opts).await?;

    shutdown_signal().await;

    agent.shutdown().await;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

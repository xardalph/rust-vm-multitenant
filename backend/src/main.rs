//! Run with
//!
//! ```not_rust
//! cargo run -p example-sqlite
//! ```
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "you should choose the server or the agent binary by running cargo run --bin <agent|server>."
    );
    Ok(())
}

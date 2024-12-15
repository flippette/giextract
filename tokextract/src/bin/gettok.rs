use eyre::Result;
use tokextract::Server;
use tokio::main;
use tracing_subscriber::{EnvFilter, prelude::*};

#[main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let _ = dotenvy::from_filename(".env");
    let _ = dotenvy::from_filename(".envrc");

    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    let server = Server::from_env().await?;

    println!("{}", tokextract::get_token(&server).await?);

    Ok(())
}

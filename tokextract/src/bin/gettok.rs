use std::io;

use eyre::{Result, bail};
use tokextract::Server;
use tokio::main;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let _ = dotenvy::from_filename(".env");
    let _ = dotenvy::from_filename(".envrc");

    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(io::stderr)
        .compact()
        .init();

    let server = Server::from_env().await?;

    loop {
        match tokextract::get_token(&server).await {
            Ok(tok) => {
                println!("{tok}");
                break;
            }
            Err(err) if err.is_timeout() => error!("timed out, retrying"),
            Err(err) => bail!(err),
        }
    }

    Ok(())
}

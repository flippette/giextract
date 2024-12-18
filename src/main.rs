mod wd;

use std::{env, io};

use eyre::Result;
use tokio::main;
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

    let wd_exec = env::var("WEBDRIVER_EXEC")?;
    let wd_server = wd::Server::spawn(&wd_exec)?;

    Ok(())
}

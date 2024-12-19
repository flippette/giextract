mod token;
mod wd;

use std::{env, io, time::Duration};

use eyre::{bail, Result};
use tokio::{main, time};
use tracing::{error, info};
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
    info!("spawned webdriver server, waiting for it to start up");
    time::sleep(Duration::from_millis(500)).await;

    let email = env::var("GIMAIL")?;
    let passwd = env::var("GIPASS")?;
    let headless = env::var("WEBDRIVER_HEADLESS")?.parse()?;
    let keepalive = env::var("WEBDRIVER_KEEPALIVE")?.parse()?;

    let token = loop {
        match token::get(&wd_server, headless, keepalive, &email, &passwd).await {
            Ok(tok) => break tok,
            Err(err) if err.is_timeout() => error!("token::get timed out, retrying"),
            Err(err) => bail!("token::get failed: {err}"),
        }
    };

    println!("{token}");

    Ok(())
}

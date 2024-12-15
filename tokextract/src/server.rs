//! Wrapper for managing the WebDriver server process.

use std::{
    env,
    process::{Child, Command, Stdio},
    time::Duration,
};

use eyre::{OptionExt, Result, WrapErr, bail};
use tokio::time;
use tracing::{error, info};

/// Wrapper for managing the WebDriver server process.
#[derive(Debug)]
pub struct Server {
    process: Child,
}

impl Server {
    pub async fn from_env() -> Result<Self> {
        let exec = env::var("WEBDRIVER_EXEC").wrap_err("WEBDRIVER_EXEC should be a command")?;
        let mut args = exec.split_whitespace();

        let mut process = Command::new(
            args.next()
                .ok_or_eyre("WEBDRIVER_EXEC should be a command")?,
        )
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
        info!("spawned server process");

        time::sleep(Duration::from_secs(1)).await;
        info!("waited for the server process to start");

        if let Some(exit_code) = process.try_wait()? {
            bail!("server process has died (code = {exit_code})");
        }

        Ok(Self { process })
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        match self.process.kill() {
            Ok(_) => info!("killed server process"),
            Err(err) => error!("failed to kill server process: {err}"),
        }
    }
}

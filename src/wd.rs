//! Wrappers for WebDriver functionality.

use std::{
    io,
    ops::Deref,
    process::{Child, Command, Stdio},
    time::Duration,
};

use fantoccini::{
    actions::{InputSource, KeyAction, KeyActions},
    elements::Element,
    error::CmdError,
    key::Key,
};
use hyper_util::client::legacy::connect::HttpConnector;
use serde_json::json;
use tokio::{runtime, time};
use tracing::{error, info};

/// Wrapper for spawning and killing a WebDriver server.
///
/// This struct spawns a server on creation, and kills it on [`Drop`].
pub struct Server {
    process: Child,
}

/// A wrapper over [`fantoccini::Client`] with easy configuration.
pub struct Client {
    inner: fantoccini::Client,
    keepalive: bool,
}

/// Extension trait for [`fantoccini::elements::Element`].
pub trait ElementExt {
    async fn scroll_into_view(&self) -> Result<(), CmdError>;
    async fn send_key(&self, key: Key) -> Result<(), CmdError>;
}

impl Server {
    /// Spawn a new [`Server`] by running the provided exec.
    pub fn spawn(exec: &str) -> io::Result<Self> {
        let mut args = exec.split_whitespace();

        Ok(Server {
            process: Command::new(args.next().unwrap_or_default())
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?,
        })
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        match self.process.kill() {
            Ok(()) => info!("killed server process"),
            Err(err) => error!("failed to kill server process: {err}"),
        }
    }
}

impl Client {
    /// Create a new [`Client`].
    ///
    /// This method takes a [`Server`] as proof one is already running.
    pub async fn new(
        _server: &Server,
        headless: bool,
        keepalive: bool,
    ) -> Result<Self, fantoccini::error::NewSessionError> {
        const RETRY_DELAY: Duration = Duration::from_millis(250);

        let caps = match headless {
            false => json!({}),
            true => json!({
                "moz:firefoxOptions": { "args": ["-headless"] },
                "goog:chromeOptions": { "args": ["headless", "disable-gpu"] },
                "ms:edgeOptions": { "args": ["--headless"] }
            }),
        }
        .as_object()
        .cloned()
        .expect("hardcoded JSON should be correct");

        let mut builder = fantoccini::ClientBuilder::new(HttpConnector::new());
        let builder = builder.capabilities(caps);
        Ok(Self {
            inner: loop {
                match builder.connect("http://localhost:4444").await {
                    Ok(client) => break client,
                    Err(err) => {
                        error!("failed to connect to WebDriver server, retrying: {err}");
                        time::sleep(RETRY_DELAY).await;
                    }
                }
            },
            keepalive,
        })
    }
}

impl Deref for Client {
    type Target = fantoccini::Client;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        match self.keepalive {
            true => info!("keepalive is true, leaving WebDriver session as-is"),
            false => {
                info!("keepalive is false, closing WebDriver session");
                runtime::Handle::current().spawn(self.inner.clone().close());
            }
        }
    }
}

impl ElementExt for Element {
    async fn scroll_into_view(&self) -> Result<(), CmdError> {
        const JS: &str = "arguments[0].scrollIntoView();";

        self.clone()
            .client()
            .execute(JS, vec![serde_json::to_value(self)?])
            .await
            .map(drop)
    }

    async fn send_key(&self, key: Key) -> Result<(), CmdError> {
        const KP_PAUSE: Duration = Duration::from_millis(50);

        self.clone()
            .client()
            .perform_actions(
                KeyActions::new("send_key".to_string())
                    .then(KeyAction::Down { value: key.into() })
                    .then(KeyAction::Pause { duration: KP_PAUSE })
                    .then(KeyAction::Up { value: key.into() }),
            )
            .await
    }
}

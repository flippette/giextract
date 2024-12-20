//! Driver for API token extraction.

use std::env;

use fantoccini::{
    ClientBuilder, Locator,
    error::{CmdError, NewSessionError},
    key::Key,
};
use hyper_util::client::legacy::connect::HttpConnector;
use serde_json::json;
use thiserror::Error;
use tracing::info;

use crate::{ElementExt, Server};

/// Entry point for extracting the API token.
///
/// This function only takes a [`Server`] as proof that there is a server to
/// connect to.
///
/// This function uses the following environment variables: `GIMAIL`, `GIPASS`,
/// `WEBDRIVER_HEADLESS`, and `WEBDRIVER_KEEPALIVE`.
pub async fn get_token(_: &Server) -> Result<String, TokenError> {
    let email = env::var("GIMAIL")?;
    let passwd = env::var("GIPASS")?;

    let caps = match env::var("WEBDRIVER_HEADLESS") {
        Ok(s) => match s.parse() {
            Err(_) => Err(TokenError::EnvMalformed(
                String::from("WEBDRIVER_HEADLESS"),
                s,
            )),
            Ok(false) => Ok(json!({})),
            Ok(true) => {
                info!("WEBDRIVER_HEADLESS is set to true, running in headless mode");
                Ok(json!({
                    "goog:chromeOptions": { "args": ["headless", "disable-gpu"] },
                    "moz:firefoxOptions": { "args": ["-headless"] },
                    "ms:edgeOptions": { "args": ["--headless"] }
                }))
            }
        },
        Err(_) => Ok(json!({})),
    }?
    .as_object()
    .expect("hardcoded JSON should be correct")
    .to_owned();

    let driver = ClientBuilder::new(HttpConnector::new())
        .capabilities(caps)
        .connect("http://localhost:4444")
        .await?;

    driver.goto("https://britishcouncil.gelielts.com").await?;
    info!("went to gelielts login page");

    let email_field = driver
        .wait()
        .for_element(Locator::Css(r#"input[name="email"]"#))
        .await?;
    info!("found email field");
    email_field.scroll_into_view().await?;
    email_field.click().await?;
    email_field.send_keys(&email).await?;
    info!("filled email field");
    email_field.send_key(Key::Enter).await?;
    info!("submitted email");

    let use_passwd_btn = driver
        .wait()
        .for_element(Locator::Css(
            r#"button[data-testid="use_password_to_log_in"]"#,
        ))
        .await?;
    info!("found use password button");
    use_passwd_btn.scroll_into_view().await?;
    use_passwd_btn.click().await?;
    info!("clicked use password button");

    let passwd_field = driver
        .wait()
        .for_element(Locator::Css(r#"input[name="password"]"#))
        .await?;
    info!("found password field");
    passwd_field.scroll_into_view().await?;
    passwd_field.click().await?;
    passwd_field.send_keys(&passwd).await?;
    info!("filled password field");
    passwd_field.send_key(Key::Enter).await?;
    info!("submitted password");

    driver.wait().for_element(Locator::Id("homeHead")).await?;
    info!("login complete");

    let cookie = driver
        .get_named_cookie("IELTS_API_TOKEN")
        .await?
        .value()
        .to_string();

    match env::var("WEBDRIVER_KEEPALIVE")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(true) => {
            info!("WEBDRIVER_KEEPALIVE is set, keeping browser instance alive");
        }
        _ => {
            info!("WEBDRIVER_KEEPALIVE is unset/false, closing browser instance");
            driver.close().await?;
        }
    }

    Ok(cookie)
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("failed to read environment variable: {0}")]
    EnvRead(#[from] env::VarError),
    #[error("environment variable {0} malformed: {1:?}")]
    EnvMalformed(String, String),
    #[error("failed to create new WebDriver session: {0}")]
    SessionCreateError(#[from] NewSessionError),
    #[error("WebDriver command error: {0}")]
    WebDriverCommandError(#[from] CmdError),
}

impl TokenError {
    /// Checks if the error is a [`CmdError::WaitTimeout`].
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::WebDriverCommandError(CmdError::WaitTimeout))
    }
}

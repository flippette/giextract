//! GEL IELTS API token extractor.

use fantoccini::{
    error::{CmdError, NewSessionError},
    key::Key,
    Locator,
};
use thiserror::Error;
use tracing::info;

use crate::wd::{self, ElementExt};

/// Retrieve an API token through a new WebDriver client created with the given
/// [`wd::Server`].
pub async fn get(
    server: &wd::Server,
    headless: bool,
    keepalive: bool,
    email: &str,
    passwd: &str,
) -> Result<String, TokenError> {
    let client = wd::Client::new(server, headless, keepalive).await?;

    client.goto("https://britishcouncil.gelielts.com").await?;
    info!("went to gelielts login page");

    let email_field = client
        .wait()
        .for_element(Locator::Css(r#"input[name="email"]"#))
        .await?;
    info!("found email field");
    email_field.scroll_into_view().await?;
    email_field.click().await?;
    email_field.send_keys(email).await?;
    info!("filled email field");
    email_field.send_key(Key::Enter).await?;
    info!("submitted email");

    let use_passwd_btn = client
        .wait()
        .for_element(Locator::Css(
            r#"button[data-testid="use_password_to_log_in"]"#,
        ))
        .await?;
    info!("found use password button");
    use_passwd_btn.scroll_into_view().await?;
    use_passwd_btn.click().await?;
    info!("clicked use password button");

    let passwd_field = client
        .wait()
        .for_element(Locator::Css(r#"input[name="password"]"#))
        .await?;
    info!("found password field");
    passwd_field.scroll_into_view().await?;
    passwd_field.click().await?;
    passwd_field.send_keys(passwd).await?;
    info!("filled password field");
    passwd_field.send_key(Key::Enter).await?;
    info!("submitted password");

    client.wait().for_element(Locator::Id("homeHead")).await?;
    info!("login complete");

    let cookie = client
        .get_named_cookie("IELTS_API_TOKEN")
        .await?
        .value()
        .to_string();

    Ok(cookie)
}

/// Error returned while retrieving API tokens.
///
/// This enum provides [`Self::is_timeout`] for retrying.
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("session error: {0}")]
    Session(#[from] NewSessionError),
    #[error("webdriver command error: {0}")]
    WebDriverCommand(#[from] CmdError),
}

impl TokenError {
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::WebDriverCommand(CmdError::WaitTimeout))
    }
}

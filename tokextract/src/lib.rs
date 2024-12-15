//! GEL IELTS API token extraction.
//!
//! This module interacts with a browser through the WebDriver API to extract
//! the cookie containing the API token.

#![allow(async_fn_in_trait)]

mod driver;
mod server;
mod util;

mod private {
    pub trait Sealed {}
}

pub use driver::{TokenError, get_token};
pub use server::Server;
pub use util::ElementExt;

/// Test that [`get_token`] works.
///
/// This test uses the environment variable `GITOK_HEAD`.
#[cfg(test)]
#[tokio::test]
async fn get_token_ok() {
    use std::{env, io};

    use tracing::{error, info};
    use tracing_subscriber::EnvFilter;

    let _ = dotenvy::from_filename(".env");
    let _ = dotenvy::from_filename(".envrc");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(io::stderr)
        .compact()
        .init();

    let server = Server::from_env()
        .await
        .expect("failed to start WebDriver server");
    info!("WebDriver server started");
    let token = loop {
        match get_token(&server).await {
            Ok(tok) => break tok,
            Err(err) if err.is_timeout() => error!("get_token timed out, retrying"),
            Err(err) => panic!("get_token error: {err}"),
        }
    };
    info!("got token");
    let expected = env::var("GITOK_HEAD").expect("GITOK_HEAD should be the head token fragment");

    assert_eq!(&token[..expected.len()], expected);
}

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

pub use driver::get_token;
pub use server::Server;
pub use util::ElementExt;

/// Test that [`get_token`] works.
///
/// This test uses the environment variable `GITOK_HEAD`.
#[cfg(test)]
#[tokio::test]
async fn get_token_ok() {
    use std::env;

    let _ = dotenvy::from_filename(".env");
    let _ = dotenvy::from_filename(".envrc");

    let server = Server::from_env()
        .await
        .expect("failed to start WebDriver server");
    let token = get_token(&server)
        .await
        .expect("get_token should successfully return");
    let expected = env::var("GITOK_HEAD").expect("GITOK_HEAD should be the head token fragment");

    assert_eq!(&token[..expected.len()], expected);
}

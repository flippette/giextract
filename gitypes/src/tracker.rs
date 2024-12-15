//! Types returned by an `/ielts/tracker` API call.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Tracker {
    pub history: Vec<Exercise>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "actionType")]
pub enum Exercise {
    IeltsExercise { id: u32 },
    IeltsPlaylist { id: u32 },
    Speaking { id: String },
}

/// Tests that `Tracker` can be deserialized correctly.
#[cfg(test)]
#[tokio::test]
async fn tracker() {
    use std::{env, io};

    use reqwest::Client;
    use tokextract::{Server, get_token};
    use tracing::{error, info};
    use tracing_subscriber::EnvFilter;

    const API_URL: &str = "https://api-britishcouncil.gelielts.com";

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

    let client = Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .expect("reqwest client should build successfully");
    info!("built reqwest client");

    let res = client
        .get(format!("{API_URL}/ielts/tracker"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("tracker request should succeed");
    info!("got tracker response");

    let tracker = res.json::<Tracker>().await;
    info!("got tracker json body: {tracker:?}");

    assert!(
        tracker.is_ok(),
        "tracker response should deserialize into Tracker"
    );
}

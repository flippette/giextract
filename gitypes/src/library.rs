//! Types returned by an `/ielts/library/practice-tests/reading` API call.

use std::num::ParseIntError;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Library {
    #[serde(rename = "data")]
    pub groups: Vec<TestGroup>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestGroup {
    #[serde(rename = "testsGroups")]
    pub exercise_groups: Vec<ExerciseGroup>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExerciseGroup {
    #[serde(rename = "testGroupName")]
    pub name: String,
    #[serde(rename = "testGroupExercises")]
    pub exercises: Vec<Exercise>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "ExerciseRaw")]
pub struct Exercise {
    pub id: u32,
    pub questions: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExerciseRaw {
    id: String,
    questions: u32,
}

impl TryFrom<ExerciseRaw> for Exercise {
    type Error = ParseIntError;

    fn try_from(raw: ExerciseRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            id: raw.id.parse()?,
            questions: raw.questions,
        })
    }
}

/// Tests that `Library` can be deserialized correctly.
#[cfg(test)]
#[tokio::test]
async fn library() {
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
        .get(format!("{API_URL}/ielts/library/practice-tests/reading"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("library request should succeed");
    info!("got library response");

    let library = res.json::<Library>().await;
    info!("got library json body: {library:?}");

    assert!(
        library.is_ok(),
        "library response should deserialize into Library"
    );
}

//! Types returned by an `/ielts/library/practice-tests/reading` API call.

use std::num::ParseIntError;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Library {
    #[serde(rename = "data")]
    pub groups: Vec<TestGroup>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestGroup {
    #[serde(rename = "testsGroupId")]
    id: u32,
    #[serde(rename = "testsCompleted")]
    completed: u32,
    #[serde(rename = "testsAvailable")]
    available: u32,
    #[serde(rename = "testsGroups")]
    exercise_groups: Vec<ExerciseGroup>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExerciseGroup {
    #[serde(rename = "testGroupName")]
    name: String,
    #[serde(rename = "testGroupCompleted")]
    completed: u32,
    #[serde(rename = "testGroupExercisesCount")]
    available: u32,
    #[serde(rename = "testGroupExercises")]
    exercises: Vec<ExerciseRaw>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "ExerciseRaw", into = "ExerciseRaw")]
pub struct Exercise {
    title: String,
    id: u32,
    started: bool,
    completed: bool,
    score: Option<u32>,
    questions: u32,
    time: u32,
    image: String,
    viewed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseRaw {
    title: String,
    action_type: String,
    id: String,
    started: u32,
    completed: bool,
    score: Option<u32>,
    questions: u32,
    time: u32,
    #[serde(rename = "type")]
    ty: String,
    image: String,
    context_id: u32,
    context_name: String,
    area: String,
    viewed: bool,
}

#[derive(Clone, Debug, Error)]
pub enum ExerciseRawError {
    #[error("parse int error: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("started should be 0 or 1, got {0} instead")]
    Started(u32),
}

impl TryFrom<ExerciseRaw> for Exercise {
    type Error = ExerciseRawError;

    fn try_from(value: ExerciseRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            title: value.title,
            id: value.id.parse()?,
            started: match value.started {
                0 => Ok(false),
                1 => Ok(true),
                other => Err(ExerciseRawError::Started(other)),
            }?,
            completed: value.completed,
            score: value.score,
            questions: value.questions,
            time: value.time,
            image: value.image,
            viewed: value.viewed,
        })
    }
}

impl From<Exercise> for ExerciseRaw {
    fn from(value: Exercise) -> Self {
        Self {
            title: value.title,
            action_type: "exercise".to_string(),
            id: value.id.to_string(),
            started: if value.started { 1 } else { 0 },
            completed: value.completed,
            score: value.score,
            questions: value.questions,
            time: value.time,
            ty: "Exercise".to_string(),
            image: value.image,
            context_id: 0,
            context_name: String::new(),
            area: String::new(),
            viewed: value.viewed,
        }
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

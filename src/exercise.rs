//! Exercise data extraction.

use fantoccini::error::{CmdError, NewSessionError};
use serde::Deserialize;
use thiserror::Error;

use crate::{API_URL, REFERER};

/// Exercise data extracted from the `/exercise/{id}` endpoint and optional
/// browser interaction.
#[derive(Clone, Debug)]
pub struct Exercise {
    pub instructions: String,
    pub general_question: String,
    pub interaction: Interaction,
    pub answers: Option<Vec<String>>,
}

/// The `interaction` field of an exercise API response.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Interaction {
    #[serde(rename = "interaction_type")]
    pub ty: InteractionType,
    pub questions: Vec<Question>,
}

/// The `interaction_type` field of an [`Interaction`].
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    Cloze,
}

/// A `question` field entry of an [`Interaction`].
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Question {
    #[serde(rename = "question_id")]
    pub id: u32,
    #[serde(rename = "question_data")]
    pub data: QuestionData,
    pub wordlist: Vec<String>,
}

/// The `question_text` field of a [`Question`].
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QuestionData {
    #[serde(rename = "question_text")]
    pub text: String,
}

/// Error returned during exercise fetching.
#[derive(Debug, Error)]
pub enum ExerciseError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("webdriver session error: {0}")]
    WebDriverSession(#[from] NewSessionError),
    #[error("webdriver command error: {0}")]
    WebDriverCommand(#[from] CmdError),
}

impl Exercise {
    pub async fn fetch_id(
        client: &reqwest::Client,
        token: &str,
        id: u32,
    ) -> Result<Self, ExerciseError> {
        #[derive(Deserialize)]
        struct ApiData {
            instructions: String,
            general_question: String,
            interaction: Interaction,
        }

        let api_data = client
            .get(format!("{API_URL}/exercise/{id}"))
            .bearer_auth(token)
            .header("referer", REFERER)
            .send()
            .await?
            .json::<ApiData>()
            .await?;

        Ok(Exercise {
            instructions: api_data.instructions,
            general_question: api_data.general_question,
            interaction: api_data.interaction,
            answers: None,
        })
    }
}

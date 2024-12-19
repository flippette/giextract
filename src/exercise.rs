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
    pub answers: Option<Vec<String>>,
}

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
        rq_client: &reqwest::Client,
        token: &str,
        id: u32,
    ) -> Result<Self, ExerciseError> {
        #[derive(Deserialize)]
        struct ApiData {
            instructions: String,
            general_question: String,
        }

        let api_data = rq_client
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
            answers: None,
        })
    }
}

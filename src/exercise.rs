//! Exercise data extraction.

use serde::Deserialize;

use crate::{API_URL, REFERER};

/// Exercise data extracted from the `/exercise/{id}` endpoint and optional
/// browser interaction.
#[derive(Clone, Debug)]
pub struct Exercise {
    pub instructions: String,
    pub general_question: String,
    pub answers: Option<Vec<String>>,
}

impl Exercise {
    pub async fn fetch_id(
        client: &reqwest::Client,
        token: &str,
        id: u32,
    ) -> Result<Self, reqwest::Error> {
        #[derive(Deserialize)]
        struct ApiData {
            instructions: String,
            general_question: String,
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
            answers: None,
        })
    }
}

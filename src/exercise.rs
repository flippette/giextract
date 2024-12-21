//! Exercise data extraction.

#![allow(dead_code)]

use std::string::FromUtf8Error;

use base64::prelude::*;
use fantoccini::error::{CmdError, NewSessionError};
use serde::Deserialize;
use thiserror::Error;

use crate::{API_URL, REFERER};

/// Key discovered from the GEL IELTS "source code".
const XXTEA_KEY: &str = "12345";

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
    pub correct: Vec<Correct>,
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

/// A `correct` field entry of an [`Interaction`].
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Correct {
    #[serde(rename = "question_id")]
    pub id: u32,
    pub answers: Vec<Vec<XxteaEncrypted>>,
}

/// [`Deserialize`] wrapper for an [`xxtea`]-encrypted [`String`].
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct XxteaEncrypted(pub String);

impl TryFrom<String> for XxteaEncrypted {
    type Error = ExerciseError;

    fn try_from(ciphertext: String) -> Result<Self, Self::Error> {
        let bytes = BASE64_STANDARD.decode(ciphertext)?;
        let mut plaintext = xxtea::decrypt_raw(&bytes, XXTEA_KEY);
        plaintext.retain(|b| b.is_ascii_alphanumeric() || b.is_ascii_punctuation() || *b == b' ');
        Ok(Self(String::from_utf8(plaintext)?))
    }
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
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("decrypted cipher not valid utf-8: {0}")]
    Utf8(#[from] FromUtf8Error),
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

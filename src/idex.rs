//! GEL IELTS exercise ID (library + tracker) extractor.

use std::num::ParseIntError;

use serde::Deserialize;

const API_URL: &str = "https://api-britishcouncil.gelielts.com";
const REFERER: &str = "https://britishcouncil.gelielts.com";

/// Extract the list of exercise IDs from the
/// `/ielts/library/practice-tests/reading` endpoint.
#[rustfmt::skip]
pub async fn library(client: &reqwest::Client, token: &str) -> Result<Vec<u32>, reqwest::Error> {
    #[derive(Deserialize)]
    struct Library {
        #[serde(rename = "data")]
        groups: Vec<LibraryGroup>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct LibraryGroup { tests_groups: Vec<TestGroup> }
    #[derive(Deserialize)]
    struct TestGroup {
        #[serde(rename = "testGroupExercises")]
        exercises: Vec<Exercise>,
    }
    #[derive(Deserialize)]
    #[serde(try_from = "ExerciseRaw")]
    struct Exercise { id: u32 }
    #[derive(Deserialize)]
    struct ExerciseRaw { id: String }

    impl TryFrom<ExerciseRaw> for Exercise {
        type Error = ParseIntError;

        fn try_from(raw: ExerciseRaw) -> Result<Self, Self::Error> {
            Ok(Self { id: raw.id.parse()? })
        }
    }

    let library = client
        .get(format!("{API_URL}/ielts/library/practice-tests/reading"))
        .bearer_auth(token)
        .header("referer", REFERER)
        .send().await?
        .json::<Library>().await?;

    Ok(library.groups
        .into_iter()
        .flat_map(|group| group.tests_groups
            .into_iter()
            .flat_map(|test_group| test_group.exercises
                .into_iter()
                .map(|exercise| exercise.id)))
        .collect())
}

/// Extract the list of exercise IDs from the `/ielts/tracker` endpoint.
#[rustfmt::skip]
pub async fn tracker(client: &reqwest::Client, token: &str) -> Result<Vec<u32>, reqwest::Error> {
    #[derive(Deserialize)]
    struct Tracker { history: Vec<Exercise> }
    #[allow(clippy::enum_variant_names, dead_code)]
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", tag = "actionType")]
    enum Exercise {
        IeltsExercise { id: u32 },
        IeltsPlaylist { id: u32 },
        Speaking { id: String },
    }

    let tracker = client
        .get(format!("{API_URL}/ielts/tracker"))
        .bearer_auth(token)
        .header("referer", REFERER)
        .send().await?
        .json::<Tracker>().await?;

    Ok(tracker.history
        .into_iter()
        .filter_map(|exercise| match exercise {
            Exercise::Speaking { .. } => None,
            Exercise::IeltsExercise { id } | Exercise::IeltsPlaylist { id } => Some(id)
        })
        .collect())
}

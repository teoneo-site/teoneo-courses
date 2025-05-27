use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::db;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ProgressStatus {
    #[serde(rename = "EVAL")]
    Eval,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "MAX_ATTEMPTS")]
    MaxAttempts,
}

impl Display for ProgressStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eval => write!(f, "EVAL"),
            Self::Failed => write!(f, "FAILED"),
            Self::Success => write!(f, "SUCCESS"),
            Self::MaxAttempts => write!(f, "MAX_ATTEMPTS"),
        }
    }
}

impl From<String> for ProgressStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "eval" => Self::Eval,
            "failed" => Self::Failed,
            "success" => Self::Success,
            "max_attempts" => Self::MaxAttempts,
            _ => panic!("Unknown task type"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Progress {
    id: u32,
    user_id: u32,
    task_id: i32,
    status: ProgressStatus,
    submission: serde_json::Value,
    score: f32,
    attempts: i32,
    updated_at: DateTime<Utc>,
}

impl Progress {
    pub fn new(
        id: u32,
        user_id: u32,
        task_id: i32,
        status: ProgressStatus,
        submission: serde_json::Value,
        score: f32,
        attempts: i32,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            task_id,
            status,
            submission,
            score,
            attempts,
            updated_at,
        }
    }
}

pub async fn update_or_insert_status(
    pool: &MySqlPool,
    user_id: u32,
    task_id: i32,
    status: ProgressStatus,
    submission: String,
    score: f32,
    attempts: i32,
) -> anyhow::Result<()> {
    db::progressdb::update_or_insert(pool, user_id, task_id, status, submission, score, attempts)
        .await?;
    Ok(())
}

pub async fn get_task_progress(
    pool: &MySqlPool,
    user_id: u32,
    task_id: i32,
) -> anyhow::Result<Progress> {
    let progress = db::progressdb::fetch_task_progress(pool, user_id, task_id).await?;
    Ok(progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_status_str() {
        let status = ProgressStatus::Eval;
        assert_eq!(status.to_string(), "EVAL");

        let status = ProgressStatus::Success;
        assert_eq!(status.to_string(), "SUCCESS");

        let status = ProgressStatus::Failed;
        assert_eq!(status.to_string(), "FAILED");

        let status = ProgressStatus::MaxAttempts;
        assert_eq!(status.to_string(), "MAX_ATTEMPTS");
    }

    #[test]
    fn test_str_into_progress_status() {
        let str = String::from("EVAL");
        let status: ProgressStatus = str.into();
        assert_eq!(status, ProgressStatus::Eval);

        let str = String::from("SUCCESS");
        let status: ProgressStatus = str.into();
        assert_eq!(status, ProgressStatus::Success);

        let str = String::from("FAILED");
        let status: ProgressStatus = str.into();
        assert_eq!(status, ProgressStatus::Failed);

        let str = String::from("MAX_ATTEMPTS");
        let status: ProgressStatus = str.into();
        assert_eq!(status, ProgressStatus::MaxAttempts);
    }
}

use chrono::DateTime;
use chrono::Utc;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::progress::Progress;
use crate::controllers::progress::ProgressStatus;

pub async fn update_or_insert(
    pool: &MySqlPool,
    user_id: u32,
    task_id: i32,
    status: ProgressStatus,
    submission: String,
    score: f32,
    attempts: i32,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO task_progress (user_id, task_id, status, submission, score, attempts) VALUES (?, ?, ?, ?, ?, ?) 
    ON DUPLICATE KEY UPDATE status = VALUES(status), submission = VALUES(submission), score = VALUES(score), attempts = IF(VALUES(status) = 'EVAL', attempts, attempts + 1), updated_at = CURRENT_TIMESTAMP
    ")
        .bind(user_id)
        .bind(task_id)
        .bind(status.to_string())
        .bind(submission)
        .bind(score)
        .bind(attempts).execute(pool).await?;
    // TODO: if "EVAL" dont increment
    Ok(())
}

pub async fn fetch_task_progress(pool: &MySqlPool, user_id: u32, task_id: i32) -> anyhow::Result<Progress> {
    let row = sqlx::query("SELECT id, status, submission, score, attempts, updated_at FROM task_progress WHERE user_id = ? AND task_id = ?")
        .bind(user_id)
        .bind(task_id)
        .fetch_one(pool).await?;

    let id: u32 = row.try_get("id")?;
    let status: ProgressStatus = row.try_get::<String, _>("status")?.into();
    let submission: serde_json::Value = row.try_get("submission")?;
    let score: f32 = row.try_get("score")?;
    let attempts: i32 = row.try_get("attempts")?;
    let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

    let progress = Progress::new(id, user_id, task_id, status, submission, score, attempts, updated_at);

    Ok(progress)
}

use sqlx::MySqlPool;

use crate::controllers::progress::Progress;
use crate::controllers::progress::ProgressStatus;
use crate::AppState;


pub async fn update_or_insert(
    state: &AppState,
    user_id: u32,
    task_id: i32,
    status: ProgressStatus,
    submission: String,
    score: f32,
    attempts: i32,
) -> anyhow::Result<()> {
    sqlx::query!("INSERT INTO task_progress (user_id, task_id, status, submission, score, attempts) VALUES (?, ?, ?, ?, ?, ?)
    ON DUPLICATE KEY UPDATE status = VALUES(status), submission = VALUES(submission), score = VALUES(score), attempts = IF(VALUES(status) = 'EVAL', attempts, attempts + 1), updated_at = CURRENT_TIMESTAMP
    ", user_id, task_id, status.to_string(), submission, score, attempts)
        .execute(&state.pool).await?;
    Ok(())
}

pub async fn fetch_task_progress(
    state: &AppState,
    user_id: u32,
    task_id: i32,
) -> anyhow::Result<Progress> {
    let progress = sqlx::query_as!(
        Progress,
        r#"
        SELECT 
            id,
            user_id,
            task_id,
            status,
            submission,
            score,
            attempts,
            updated_at
        FROM task_progress
        WHERE user_id = ? AND task_id = ?
        "#,
        user_id,
        task_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(progress)
}


pub async fn get_prompt_task_attemps(
    pool: &MySqlPool,
    user_id: u32,
    task_id: i32,
) -> anyhow::Result<(i32, i32)> {
    let row = sqlx::query!(
        r#"
        SELECT t.attempts, p.max_attempts 
        FROM task_progress t 
        LEFT JOIN prompts p ON t.task_id = p.task_id 
        WHERE t.user_id = ? AND t.task_id = ?
        "#,
        user_id,
        task_id
    )
    .fetch_one(pool)
    .await?;

    Ok((row.attempts, row.max_attempts.unwrap_or(3)))
}

use chrono::DateTime;
use chrono::Utc;
use redis::Commands;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::progress::Progress;
use crate::controllers::progress::ProgressStatus;
use crate::AppState;


impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for Progress {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            task_id: row.try_get("task_id")?,
            status: row.try_get::<String, _>("status")?.into(),
            submission: row.try_get("submission")?,
            score: row.try_get("score")?,
            attempts: row.try_get("attempts")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}


pub async fn update_or_insert(
    state: &AppState,
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
        .bind(attempts).execute(&state.pool).await?;

    
    let mut conn = state.redis.get().unwrap(); // If it failes to get a connection here it's very bad and data will be outdated, so panic better
    // TODO: избавиться от этого позора
    let cache_key = format!("progress:{}:{}", user_id, task_id);
    let cache_key_task = format!("task:{}", task_id);
    let cache_key_info_all = format!("user:info:all:{}", user_id);
    let cache_key_info_courses = format!("user:info:courses:{}", user_id);
    let cache_key_stats = format!("user:stats:{}", user_id);
    redis::cmd("DEL").arg(cache_key).arg(cache_key_task).arg(cache_key_info_all).arg(cache_key_info_courses).arg(cache_key_stats).query(&mut conn).unwrap_or(());
    Ok(())
}

pub async fn fetch_task_progress(
    state: &AppState,
    user_id: u32,
    task_id: i32,
) -> anyhow::Result<Progress> {
    let cache_key = format!("progress:{}:{}", user_id, task_id);
    if let Ok(mut conn) = state.redis.get() { 
         // Пробуем взять из кэша
        if let Ok(cached) = conn.get::<_, String>(&cache_key) {
            if let Ok(progress) = serde_json::from_str::<Progress>(&cached) {
                return Ok(progress);
            }
        }
    }
    // Если не найдено — берём из базы
    let progress = sqlx::query_as::<_, Progress>(
        "SELECT id, user_id, task_id, status, submission, score, attempts, updated_at FROM task_progress
         WHERE user_id = ? AND task_id = ?"
    )
    .bind(user_id)
    .bind(task_id)
    .fetch_one(&state.pool)
    .await?;

    if let Ok(mut conn) = state.redis.get() { 
        let _: () = conn
        .set_ex(&cache_key, serde_json::to_string(&progress)?, 300)
        .unwrap_or(()); // Ошибку кэша можно игнорировать
    }
    
    Ok(progress)
}


pub async fn get_prompt_task_attemps(
    pool: &MySqlPool,
    user_id: u32,
    task_id: i32,
) -> anyhow::Result<(i32, i32)> {
    let attempts = sqlx::query_as::<_, (i32, i32)>("SELECT t.attempts, p.max_attempts FROM task_progress t LEFT JOIN prompts p ON t.task_id = p.task_id WHERE t.user_id = ? AND t.task_id = ?")
        .bind(user_id)
        .bind(task_id)
        .fetch_one(pool).await?;
    Ok((attempts.0, attempts.1))
}

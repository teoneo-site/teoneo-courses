use redis::Commands;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::progress::ProgressStatus;
use crate::controllers::task::Task;
use crate::controllers::task::TaskShortInfo;
use crate::controllers::task::TaskType;
use crate::AppState;

pub async fn fetch_tasks_for_module(
    state: &AppState,
    module_id: i32,
    user_id: Option<i32>,
) -> anyhow::Result<Vec<TaskShortInfo>> {
    let cache_key = format!("module:{}:tasks:all", module_id);
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<&str, String>(&cache_key) {
            if let Ok(parsed_tasks) = serde_json::from_str::<Vec<TaskShortInfo>>(&val) {
                return Ok(parsed_tasks)
            }
        }
    }

    let rows = if let Some(user_id) = user_id {
        sqlx::query("SELECT t.id, t.title, t.type, tp.status AS status FROM tasks t LEFT JOIN task_progress tp ON tp.task_id = t.id AND tp.user_id = ? WHERE t.module_id = ?")
        .bind(user_id)
        .bind(module_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query("SELECT id, title, type FROM tasks WHERE module_id = ?") // Todo: Pagination with LIMIT
            .bind(module_id)
            .fetch_all(&state.pool)
            .await?
    };
    let mut result = Vec::new(); // Vec of Courses

    for row in rows {
        let id: i32 = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let task_type: TaskType = row.try_get::<String, _>("type")?.into();
        let status: Option<ProgressStatus> = row
            .try_get::<Option<String>, _>("status")
            .map(|opt| opt.map(Into::into))
            .map_err(|_| ())
            .ok()
            .flatten();
        result.push(TaskShortInfo::new(id, module_id, title, task_type, status));
    }

    if let Ok(mut conn) = state.redis.get() { 
        let result_str = serde_json::to_string(&result).unwrap(); // Should not panic
        let _ : () = conn.set_ex(&cache_key, result_str, 3600).unwrap_or(());
    }
    Ok(result)
}

pub async fn fetch_task_type(pool: &MySqlPool, task_id: i32) -> anyhow::Result<TaskType> {
    let row = sqlx::query("SELECT type FROM tasks WHERE id = ?")
        .bind(task_id)
        .fetch_one(pool)
        .await?;

    let task_type: TaskType = row.try_get::<String, _>("type")?.into();
    Ok(task_type)
}

// Should not cache this since task_id is probably indexed so its a pretty fast search
pub async fn fetch_task_answers(
    pool: &MySqlPool,
    task_type: TaskType,
    task_id: i32,
) -> anyhow::Result<String> {
    let row = match task_type {
        TaskType::Quiz => {
            sqlx::query("SELECT answers FROM quizzes WHERE task_id = ?")
                .bind(task_id)
                .fetch_one(pool)
                .await
        }
        TaskType::Match => {
            sqlx::query("SELECT answers FROM matches WHERE task_id = ?")
                .bind(task_id)
                .fetch_one(pool)
                .await
        }
        _ => {
            panic!("Answers isn't supported for this TaskType")
        }
    }?;

    let answers: String = row.try_get(0)?;
    Ok(answers)
}

pub async fn fetch_task(
    state: &AppState,
    module_id: i32,
    task_id: i32,
    user_id: Option<i32>,
) -> anyhow::Result<Task> {
    let cache_key = format!("task:{}", task_id);
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<&str, String>(&cache_key) { // Cache is up to date (in terms of progress), becase we delete key, when updating progress
            if let Ok(parsed_task) = serde_json::from_str(&val) {
                return Ok(parsed_task)
            } 
        }
    }

    let row = if let Some(user_id) = user_id {
        sqlx::query(
            "SELECT 
            t.title, t.type,
            q.question as qquestion, q.possible_answers, q.is_multiple,
            l.text,
            m.question, m.left_items, m.right_items,
            p.question as pquestion, p.max_attempts,
            pr.status, pr.score
            FROM tasks t
                LEFT JOIN quizzes q ON t.id = q.task_id AND t.type = 'Quiz'
                LEFT JOIN lectures l ON t.id = l.task_id AND t.type = 'Lecture'
                LEFT JOIN matches m ON t.id = m.task_id AND t.type = 'Match'
                LEFT JOIN prompts p on t.id = p.task_id AND t.type = 'prompt'
                LEFT JOIN task_progress pr ON pr.task_id = t.id AND pr.user_id = ?
            WHERE t.id = ?",
        )
        .bind(user_id)
        .bind(task_id)
        .fetch_one(&state.pool)
        .await?
    } else {
        
        sqlx::query(
            "SELECT t.title, t.type,
                    q.question as qquestion, q.possible_answers, q.is_multiple,
                    l.text,
                    m.question, m.left_items, m.right_items,
                    p.question as pquestion, p.max_attempts
            FROM tasks t
                LEFT JOIN quizzes q ON t.id = q.task_id AND t.type = 'Quiz'
                LEFT JOIN lectures l ON t.id = l.task_id AND t.type = 'Lecture'
                LEFT JOIN matches m ON t.id = m.task_id AND t.type = 'Match'
                LEFT JOIN prompts p on t.id = p.task_id AND t.type = 'prompt'
            WHERE t.id = ?",
        )
        .bind(task_id)
        .fetch_one(&state.pool)
        .await?
    };

    let title: String = row.try_get("title")?;
    let task_type_str: String = row.try_get("type")?;
    let task_type: TaskType = task_type_str.into();
    let status: Option<ProgressStatus> = row
        .try_get::<Option<String>, _>("status")
        .map(|opt| opt.map(Into::into))
        .map_err(|_| ())
        .ok()
        .flatten();
    let score: Option<f32> = row
        .try_get::<Option<f32>, _>("score")
        .map_err(|_| ())
        .ok()
        .flatten();

    let content = match task_type {
        TaskType::Quiz => {
            let question: String = row.try_get("qquestion")?;
            println!("here");
            let possible_answers: String = row.try_get("possible_answers")?;
            let is_multiple: bool = row.try_get("is_multiple")?;

            serde_json::json!({
                "question": question,
                "possible_answers": possible_answers.split(';').collect::<Vec<&str>>(),
                "is_multiple": is_multiple,
            })
        }
        TaskType::Lecture => {
            let text: String = row.try_get("text")?;
            serde_json::json!({
                "text": text,
            })
        }
        TaskType::Match => {
            let question: String = row.try_get("question")?;
            let left_items: String = row.try_get("left_items")?;
            let right_items: String = row.try_get("right_items")?;

            serde_json::json!({
                "question": question,
                "left_items": left_items.split(';').collect::<Vec<&str>>(),
                "right_items": right_items.split(';').collect::<Vec<&str>>(),
            })
        }
        TaskType::Prompt => {
            let question: String = row.try_get("pquestion")?;
            let max_attempts: i32 = row.try_get("max_attempts")?;
            serde_json::json!({
                "question": question,
                "max_attempts": max_attempts
            })
        }
    };
    let task = Task::new(
        task_id, module_id, title, task_type, content, status, score,
    );

    if let Ok(mut conn) = state.redis.get() { 
        let task_str = serde_json::to_string(&task).unwrap(); // Should not panic
        let _ : () = conn.set_ex(&cache_key, task_str, 3600).unwrap_or(());
    }

    Ok(task)
}

pub async fn fetch_prompt_details(
    pool: &MySqlPool,
    task_id: i32,
) -> anyhow::Result<(String, Option<String>)> {
    let row = sqlx::query("SELECT question, additional_prompt FROM prompts WHERE task_id = ?")
        .bind(task_id)
        .fetch_one(pool)
        .await?;

    let question: String = row.try_get(0)?;
    let additional_prompt: Option<String> = row.try_get(1)?;
    Ok((question, additional_prompt))
}

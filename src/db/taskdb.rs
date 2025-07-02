use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::task::Task;
use crate::controllers::task::TaskShortInfo;
use crate::controllers::task::TaskType;
use crate::AppState;


pub async fn fetch_tasks_for_module(
    state: &AppState,
    module_id: i32,
    user_id: Option<i32>,
) -> anyhow::Result<Vec<TaskShortInfo>> {
    let tasks = if let Some(user_id) = user_id {
        sqlx::query_as::<_, TaskShortInfo>("SELECT t.id, t.module_id, t.title, t.type, tp.status AS status FROM tasks t LEFT JOIN task_progress tp ON tp.task_id = t.id AND tp.user_id = ? WHERE t.module_id = ?")
        .bind(user_id)
        .bind(module_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, TaskShortInfo>(
            "SELECT id, module_id, title, type FROM tasks WHERE module_id = ?",
        ) // Todo: Pagination with LIMIT
        .bind(module_id)
        .fetch_all(&state.pool)
        .await?
    };
    Ok(tasks)
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

    let task = if let Some(user_id) = user_id {
        sqlx::query_as::<_, Task>(
            "SELECT 
            t.id, t.module_id, t.title, t.type,
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
        sqlx::query_as::<_, Task>(
            "SELECT t.id, t.module_id, t.title, t.type,
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
    Ok(task)
}

pub async fn fetch_prompt_details(
    pool: &MySqlPool,
    task_id: i32,
) -> anyhow::Result<(String, Option<String>)> {
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT question, additional_prompt FROM prompts WHERE task_id = ?",
    )
    .bind(task_id)
    .fetch_one(pool)
    .await?;
    Ok((row.0, row.1))
}

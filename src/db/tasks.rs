use redis::Commands;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::tasks::Task;
use crate::controllers::tasks::TaskShortInfo;
use crate::controllers::tasks::TaskType;
use crate::BasicState;

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for Task {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let id: i32 = row.try_get("id")?;
        let module_id: i32 = row.try_get("module_id")?;
        let course_id: i32 = row.try_get("course_id")?;
        let title: String = row.try_get("title")?;
        let task_type: TaskType = row.try_get::<String, _>("task_type")?.into();
        let status: Option<String> = row.try_get("status")?;
        let score: Option<f32> = row.try_get("score")?;

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
        Ok(Self {
            id,
            module_id,
            course_id,
            title,
            task_type: task_type.to_string(),
            content,
            status,
            score,
        })
    }
}

pub async fn fetch_tasks_for_module(
    state: &BasicState,
    module_id: i32,
    user_id: Option<i32>,
) -> anyhow::Result<Vec<TaskShortInfo>> {
    
    
    let tasks = if let Some(user_id) = user_id {
        sqlx::query_as::<_, TaskShortInfo>("SELECT t.id, t.module_id, t.course_id, t.title, t.type as task_type, tp.status AS status FROM tasks t LEFT JOIN task_progress tp ON tp.task_id = t.id AND tp.user_id = ? WHERE t.module_id = ?")
        .bind(user_id)
        .bind(module_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        let cache_key = format!("modules:{}:tasks", module_id);
        if let Ok(mut conn) = state.redis.get() {
            if let Ok(val) = conn.get::<String, String>(cache_key.clone()) {
                if let Ok(parsed_module) = serde_json::from_str::<Vec<TaskShortInfo>>(&val) {
                    return Ok(parsed_module)
                }
            }
        }

        let tasks = sqlx::query_as::<_, TaskShortInfo>(
            "SELECT id, module_id, course_id, title, type as task_type FROM tasks WHERE module_id = ?",
        ) // Todo: Pagination with LIMIT
        .bind(module_id)
        .fetch_all(&state.pool)
        .await?;

        if let Ok(mut conn) = state.redis.get() {
            let result_str = serde_json::to_string(&tasks).unwrap();
            conn.set_ex(cache_key, result_str, 3600).unwrap_or(()); // user may buy a course, so expire date should be short
        }    

        tasks
    };

    

    Ok(tasks)
}

pub async fn get_tasks_passed(
    pool: &MySqlPool,
    course_id: i32,
    user_id: u32,
) -> anyhow::Result<i64> {
    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count"
        FROM task_progress tp
        JOIN tasks t ON tp.task_id = t.id
        WHERE tp.user_id = ? AND tp.status = 'SUCCESS' AND t.course_id = ?
        "#,
        user_id,
        course_id
    )
    .fetch_one(pool)
    .await?;

    Ok(total)
}

pub async fn fetch_tasks_total(pool: &MySqlPool, course_id: i32) -> anyhow::Result<i64> {
    let result = sqlx::query!("SELECT COUNT(*) AS tasks_total FROM tasks WHERE course_id = ?", course_id)   
        .fetch_one(pool)
        .await?;
    Ok(result.tasks_total)
}

pub async fn fetch_task_type(pool: &MySqlPool, task_id: i32) -> anyhow::Result<TaskType> {
    let raw_type = sqlx::query_scalar!(
        "SELECT type as task_type FROM tasks WHERE id = ?",
        task_id
    )
    .fetch_one(pool)
    .await?;

    Ok(TaskType::from(raw_type))
}

// Should not cache this since task_id is probably indexed so its a pretty fast search
pub async fn fetch_task_answers(
    pool: &MySqlPool,
    task_type: TaskType,
    task_id: i32,
) -> anyhow::Result<String> {
    let answers = match task_type {
        TaskType::Quiz => {
            sqlx::query_scalar!(
                "SELECT answers FROM quizzes WHERE task_id = ?",
                task_id
            )
            .fetch_one(pool)
            .await
        }
        TaskType::Match => {
            sqlx::query_scalar!(
                "SELECT answers FROM matches WHERE task_id = ?",
                task_id
            )
            .fetch_one(pool)
            .await
        }
        _ => panic!("Answers isn't supported for this TaskType"),
    }?;

    Ok(answers)
}

pub async fn fetch_task(
    state: &BasicState,
    task_id: i32,
    user_id: Option<i32>,
) -> anyhow::Result<Task> {

    let task = if let Some(user_id) = user_id {
        sqlx::query_as::<_, Task>(
            "SELECT 
            t.id, t.module_id, t.course_id, t.title, t.type as task_type,
            q.question AS qquestion, q.possible_answers, q.is_multiple,
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
        let cache_key = format!("tasks:{}", task_id);
        if let Ok(mut conn) = state.redis.get() {
            if let Ok(val) = conn.get::<String, String>(cache_key.clone()) {
                if let Ok(parsed_module) = serde_json::from_str::<Task>(&val) {
                    return Ok(parsed_module)
                }
            }
        }

        let task = sqlx::query_as::<_, Task>(
            "SELECT t.id, t.module_id, t.course_id, t.title, t.type as task_type,
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
        .await?;

        if let Ok(mut conn) = state.redis.get() {
            let result_str = serde_json::to_string(&task).unwrap();
            conn.set_ex(cache_key, result_str, 3600).unwrap_or(()); // user may buy a course, so expire date should be short
        }    

        task
    };
    Ok(task)
}


pub async fn fetch_courses_started(pool: &MySqlPool, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let rows = sqlx::query!("SELECT DISTINCT t.course_id 
             FROM task_progress tp
             JOIN tasks t ON tp.task_id = t.id
             WHERE tp.user_id = ?", user_id)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|element| element.course_id).collect())
}

pub async fn fetch_courses_completed(pool: &MySqlPool, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let rows = sqlx::query!("SELECT DISTINCT t.course_id
        FROM tasks t
        JOIN task_progress tp ON t.id = tp.task_id
        WHERE tp.user_id = ? AND tp.status = 'SUCCESS'
        GROUP BY t.course_id
        HAVING COUNT(DISTINCT t.id) = (
            SELECT COUNT(*) 
            FROM tasks t2 
            WHERE t2.course_id = t.course_id
        )", user_id)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|element| element.course_id).collect())
}


pub async fn fetch_prompt_details(
    pool: &MySqlPool,
    task_id: i32,
) -> anyhow::Result<(String, Option<String>)> {

    let row = sqlx::query!(
        "SELECT question, additional_prompt FROM prompts WHERE task_id = ?",
        task_id
    )
    .fetch_one(pool)
    .await?;

    Ok((row.question, row.additional_prompt))
}

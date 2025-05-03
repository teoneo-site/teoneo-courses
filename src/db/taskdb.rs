use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::task::Task;
use crate::controllers::task::TaskShortInfo;
use crate::controllers::task::TaskType;

pub async fn fetch_tasks_for_module(
    pool: &MySqlPool,
    module_id: i32,
) -> anyhow::Result<Vec<TaskShortInfo>> {
    let rows = sqlx::query("SELECT id, title, type FROM tasks WHERE id = ?") // Todo: Pagination with LIMIT
        .bind(module_id)
        .fetch_all(pool)
        .await?;

    let mut result = Vec::new(); // Vec of Courses

    for row in rows {
        let id: i32 = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let task_type: TaskType = row.try_get::<String, _>("type")?.into();

        result.push(TaskShortInfo::new(id, module_id, title, task_type));
    }

    Ok(result)
}

pub async fn fetch_task(pool: &MySqlPool, module_id: i32, task_id: i32) -> anyhow::Result<Task> {
    let row: sqlx::mysql::MySqlRow =
        sqlx::query("SELECT title, type, content FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_one(pool)
            .await?;

    let title: String = row.try_get("title")?;
    let task_type: TaskType = row.try_get::<String, _>("type")?.into();
    let content = row.try_get::<serde_json::Value, _>("content")?;

    Ok(Task::new(task_id, module_id, title, task_type, content))
}

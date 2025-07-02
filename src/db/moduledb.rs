use sqlx::Row;

use crate::controllers::module::ModuleInfo;
use crate::AppState;

pub async fn fetch_modules_for_course(
    state: &AppState,
    course_id: i32,
) -> anyhow::Result<Vec<ModuleInfo>> {
    let modules = sqlx::query_as!(ModuleInfo, 
        "SELECT id, course_id, title, description, theory FROM modules WHERE course_id = ?", course_id) // Todo: Pagination with LIMIT
    .fetch_all(&state.pool)
    .await?;

    Ok(modules)
}

pub async fn fetch_module(
    state: &AppState,
    course_id: i32,
    module_id: i32,
) -> anyhow::Result<ModuleInfo> {
    let module = sqlx::query_as!(ModuleInfo, "SELECT id, course_id, title, description, theory FROM modules WHERE course_id = ? AND id = ?", course_id, module_id)
        .fetch_one(&state.pool)
        .await?;
    Ok(module)
}

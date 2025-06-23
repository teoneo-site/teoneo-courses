use redis::Commands;
use sqlx::Row;

use crate::controllers::module::ModuleInfo;
use crate::AppState;

pub async fn fetch_modules_for_course(
    state: &AppState,
    course_id: i32,
) -> anyhow::Result<Vec<ModuleInfo>> {
    let cache_key = format!("course:{}:modules:all", course_id);
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(val) = conn.get::<&str, String>(&cache_key) {
            if let Ok(parsed_modules) = serde_json::from_str::<Vec<ModuleInfo>>(&val) {
                return Ok(parsed_modules);
            }
        }
    }

    let modules = sqlx::query_as!(ModuleInfo, 
        "SELECT id, course_id, title, description, theory FROM modules WHERE course_id = ?", course_id) // Todo: Pagination with LIMIT
    .fetch_all(&state.pool)
    .await?;
    if let Ok(mut conn) = state.redis.get() {
        let result_str = serde_json::to_string(&modules).unwrap(); // Not supposed to panic
        let _: () = conn.set_ex(&cache_key, result_str, 3600).unwrap_or(());
    }

    Ok(modules)
}

pub async fn fetch_module(
    state: &AppState,
    course_id: i32,
    module_id: i32,
) -> anyhow::Result<ModuleInfo> {
    let cache_key = format!("course:{}:module:{}", course_id, module_id);
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(val) = conn.get::<&str, String>(&cache_key) {
            if let Ok(parsed_module) = serde_json::from_str(&val) {
                return Ok(parsed_module);
            }
        }
    }
    let module = sqlx::query_as!(ModuleInfo, "SELECT id, course_id, title, description, theory FROM modules WHERE course_id = ? AND id = ?", course_id, module_id)
        .fetch_one(&state.pool)
        .await?;

    if let Ok(mut conn) = state.redis.get() {
        let module_str = serde_json::to_string(&module).unwrap(); // Not supposed to panic
        conn.set_ex(&cache_key, module_str, 3600).unwrap_or(());
    }
    Ok(module)
}

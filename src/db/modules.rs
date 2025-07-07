use redis::Commands;

use crate::controllers::modules::{ModuleInfo, ModuleShortInfo};
use crate::BasicState;


pub async fn fetch_modules_for_course(
    state: &BasicState,
    course_id: i32,
) -> anyhow::Result<Vec<ModuleShortInfo>> {
    let cache_key = format!("courses:{}:modules", course_id);
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(val) = conn.get::<String, String>(cache_key.clone()) {
            if let Ok(parsed_courses) = serde_json::from_str::<Vec<ModuleShortInfo>>(&val) {
                return Ok(parsed_courses)
            }
        }
    }

    let modules = sqlx::query_as!(ModuleShortInfo, 
        "SELECT id, course_id, title, description FROM modules WHERE course_id = ?", course_id) // Todo: Pagination with LIMIT
    .fetch_all(&state.pool)
    .await?;

    if let Ok(mut conn) = state.redis.get() {
        let result_str = serde_json::to_string(&modules).unwrap();
        conn.set_ex(cache_key, result_str, 3600).unwrap_or(()); // user may buy a course, so expire date should be short
    } 

    Ok(modules)
}

pub async fn fetch_module(
    state: &BasicState,
    course_id: i32,
    module_id: i32,
) -> anyhow::Result<ModuleInfo> {
    let cache_key = format!("courses:{}:modules:{}", course_id, module_id);
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(val) = conn.get::<String, String>(cache_key.clone()) {
            if let Ok(parsed_module) = serde_json::from_str::<ModuleInfo>(&val) {
                return Ok(parsed_module)
            }
        }
    }

    let module = sqlx::query_as!(ModuleInfo, "SELECT id, course_id, title, description, theory FROM modules WHERE course_id = ? AND id = ?", course_id, module_id)
        .fetch_one(&state.pool)
        .await?;

    if let Ok(mut conn) = state.redis.get() {
        let result_str = serde_json::to_string(&module).unwrap();
        conn.set_ex(cache_key, result_str, 3600).unwrap_or(()); // user may buy a course, so expire date should be short
    } 
    Ok(module)
}

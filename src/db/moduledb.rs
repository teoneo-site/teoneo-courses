use redis::Commands;
use sqlx::Row;

use crate::controllers::module::ModuleInfo;
use crate::AppState;

pub async fn fetch_modules_for_course(
    state: &AppState,
    course_id: i32,
) -> anyhow::Result<Vec<ModuleInfo>> {
    let mut conn = state.redis.get().unwrap();
    let cache_key = format!("course:{}:modules:all", course_id);
    if let Ok(val) = conn.get::<&str, String>(&cache_key) {
        if let Ok(parsed_modules) = serde_json::from_str::<Vec<ModuleInfo>>(&val) {
            return Ok(parsed_modules)
        }
    }

    let rows =
        sqlx::query("SELECT id, title, description, theory, picture_url, video_url FROM modules WHERE course_id = ?")// Todo: Pagination with LIMIT
            .bind(course_id)
            .fetch_all(&state.pool)
            .await?;
    let mut result = Vec::new(); // Vec of Courses
    for row in rows {
        let id: i32 = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let description: String = row.try_get("description")?;
        let theory: String = row.try_get("theory")?;
        let picture_url: String = row.try_get("picture_url")?;
        let video_url: String = row.try_get("video_url")?;

        result.push(ModuleInfo::new(
            id,
            course_id,
            title,
            description,
            theory,
            picture_url,
            video_url,
        ));
    }
    let result_str = serde_json::to_string(&result).unwrap(); // Not supposed to panic
    let _ : () = conn.set_ex(&cache_key, result_str, 3600).unwrap_or(());

    Ok(result)
}

pub async fn fetch_module(
    state: &AppState,
    course_id: i32,
    module_id: i32,
) -> anyhow::Result<ModuleInfo> {
    let mut conn = state.redis.get().unwrap();
    let cache_key = format!("course:{}:module:{}", course_id, module_id);
    if let Ok(val) = conn.get::<&str, String>(&cache_key) {
        if let Ok(parsed_module) = serde_json::from_str(&val) {
            return Ok(parsed_module)
        }
    }

    let row = sqlx::query("SELECT title, description, theory, picture_url, video_url FROM modules WHERE course_id = ? AND id = ?")
        .bind(course_id)
        .bind(module_id)
        .fetch_one(&state.pool)
        .await?;

    let title: String = row.try_get("title")?;
    let description: String = row.try_get("description")?;
    let theory: String = row.try_get("theory")?;
    let picture_url: String = row.try_get("picture_url")?;
    let video_url: String = row.try_get("video_url")?;
    let module = ModuleInfo::new(
        module_id,
        course_id,
        title,
        description,
        theory,
        picture_url,
        video_url,
    );
    let module_str = serde_json::to_string(&module).unwrap(); // Not supposed to panic
    let _ : () = conn.set_ex(&cache_key, module_str, 3600).unwrap_or(());

    Ok(module)
}

use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::module::ModuleInfo;

pub async fn fetch_modules_for_course(
    pool: &MySqlPool,
    course_id: i32,
) -> anyhow::Result<Vec<ModuleInfo>> {
    let rows =
        sqlx::query("SELECT id, title, description, theory, picture_url, video_url FROM modules WHERE course_id = ?")// Todo: Pagination with LIMIT
            .bind(course_id)
            .fetch_all(pool)
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

    Ok(result)
}

pub async fn fetch_module(
    pool: &MySqlPool,
    course_id: i32,
    module_id: i32,
) -> anyhow::Result<ModuleInfo> {
    let row = sqlx::query("SELECT title, description, theory, picture_url, video_url FROM modules WHERE course_id = ? AND id = ?")
        .bind(course_id)
        .bind(module_id)
        .fetch_one(pool)
        .await?;

    let title: String = row.try_get("title")?;
    let description: String = row.try_get("description")?;
    let theory: String = row.try_get("theory")?;
    let picture_url: String = row.try_get("picture_url")?;
    let video_url: String = row.try_get("video_url")?;

    Ok(ModuleInfo::new(
        module_id,
        course_id,
        title,
        description,
        theory,
        picture_url,
        video_url,
    ))
}

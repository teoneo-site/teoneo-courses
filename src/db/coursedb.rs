use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::course::CourseInfo;

pub async fn fetch_courses(pool: &MySqlPool) -> anyhow::Result<Vec<CourseInfo>> {
    let rows = sqlx::query("SELECT id, title, description, tags, picture_url FROM courses") // Todo: Pagination with LIMIT
        .fetch_all(pool)
        .await?;

    let mut result = Vec::new(); // Vec of Courses

    for row in rows {
        let id: i32 = row.try_get(0)?;
        let title: String = row.try_get(1)?;
        let description: String = row.try_get(2)?;
        let tags = row
            .try_get::<String, _>(3)?
            .split(",")
            .map(|str| str.to_owned())
            .collect::<Vec<String>>(); // tags are stored like "python,ai,cock" we transform it into ["py...", "ai", "cock"]

        let picture_url: String = row.try_get(4)?;
        result.push(CourseInfo::new(id, title, description, tags, picture_url));
    }

    Ok(result)
}

pub async fn fetch_course(pool: &MySqlPool, id: i32) -> anyhow::Result<CourseInfo> {
    let row = sqlx::query("SELECT title, description, tags, picture_url FROM courses WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    let title: String = row.try_get("title")?;
    let description: String = row.try_get("description")?;
    let tags = row
        .try_get::<String, _>("tags")?
        .split(",")
        .map(|str| str.to_owned())
        .collect::<Vec<String>>(); // tags are stored like "python,ai,cock" we transform it into ["py...", "ai", "cock"]
    let picture_url: String = row.try_get("picture_url")?;

    Ok(CourseInfo::new(id, title, description, tags, picture_url))
}

pub async fn validate_course_ownership(pool: &MySqlPool, user_id: i32, course_id: i32) -> anyhow::Result<bool> {
    let row = sqlx::query("SELECT * FROM payments_history WHERE user_id = ? AND course_id = ?")
        .bind(user_id)
        .bind(course_id)
        .fetch_one(pool).await?;
    Ok(!row.is_empty())
}
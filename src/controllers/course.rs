use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::db;

#[derive(Serialize, Deserialize)]
pub struct CourseInfo {
    id: i32,
    title: String,
    description: String,
    tags: Vec<String>,
    picture_url: String,
}

impl CourseInfo {
    pub fn new(
        id: i32,
        title: String,
        description: String,
        tags: Vec<String>,
        picture_url: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            tags,
            picture_url,
        }
    }
}

// Currently, there is really no need for this method in the controller,
// you can just call fetch from the handler,
// BUT maybe we'll need this in future for some settings kinda stuff
pub async fn get_all_courses(pool: &MySqlPool) -> anyhow::Result<Vec<CourseInfo>> {
    let courses = db::coursedb::fetch_courses(pool).await?;
    Ok(courses)
}

// Currently, there is really no need for this method in the controller,
// you can just call fetch from the handler,
// BUT maybe we'll need this in future for some settings kinda stuff
pub async fn get_course(pool: &MySqlPool, id: i32) -> anyhow::Result<CourseInfo> {
    let course = db::coursedb::fetch_course(pool, id).await?;
    Ok(course)
}

pub async fn verify_ownership(pool: &MySqlPool, user_id: i32, course_id: i32) -> anyhow::Result<bool> {
    Ok(db::coursedb::validate_course_ownership(pool, user_id, course_id).await?)
}
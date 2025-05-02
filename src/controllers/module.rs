use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::db;

#[derive(Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: i32,
    pub course_id: i32,
    pub title: String,
    pub description: String,
    pub theory: String,
    pub picture_url: String,
    pub video_url: String,
}

impl ModuleInfo {
    pub fn new(
        id: i32,
        course_id: i32,
        title: String,
        description: String,
        theory: String,
        picture_url: String,
        video_url: String,
    ) -> Self {
        Self {
            id,
            course_id,
            title,
            description,
            theory,
            picture_url,
            video_url,
        }
    }
}

// Functions may be later used to implement pagination or something
pub async fn get_modules_for_course(
    pool: &MySqlPool,
    course_id: i32,
) -> anyhow::Result<Vec<ModuleInfo>> {
    let modules = db::moduledb::fetch_modules_for_course(pool, course_id).await?;
    Ok(modules)
}

pub async fn get_module(
    pool: &MySqlPool,
    course_id: i32,
    module_id: i32,
) -> anyhow::Result<ModuleInfo> {
    let module = db::moduledb::fetch_module(pool, course_id, module_id).await?;
    Ok(module)
}

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{db, BasicState};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ModuleInfo {
    pub id: i32,
    pub course_id: i32,
    pub title: String,
    pub description: String,
    pub theory: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ModuleShortInfo {
    pub id: i32,
    pub course_id: i32,
    pub title: String,
    pub description: String
}


// Functions may be later used to implement pagination or something
pub async fn get_modules_for_course(
    state: &BasicState,
    course_id: i32,
) -> anyhow::Result<Vec<ModuleShortInfo>> {
    let modules = db::modules::fetch_modules_for_course(state, course_id).await?;
    Ok(modules)
}


pub async fn get_module(
    state: &BasicState,
    course_id: i32,
    module_id: i32,
) -> anyhow::Result<ModuleInfo> {
    let module = db::modules::fetch_module(state, course_id, module_id).await?;
    Ok(module)
}

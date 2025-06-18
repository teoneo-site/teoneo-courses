use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{db, AppState};

#[derive(Default, Serialize, Deserialize, ToSchema)]
pub struct UserInfoFull {
    pub username: String,
    pub email: String,
    pub courses: Vec<i32>,
}


#[derive(Default, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    pub username: String,
    pub email: String
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserStats {
    pub courses_owned: i64,
    pub courses_started: i64,
    pub courses_completed: i64,
}

pub async fn get_user_info_all(app_state: &AppState, user_id: u32) -> anyhow::Result<UserInfoFull> {
    let info = db::userdb::get_user_info_all(app_state, user_id).await?;
    Ok(info)
}
pub async fn get_courses_info(app_state: &AppState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let info = db::userdb::get_courses_info(app_state, user_id).await?;
    Ok(info)
}
pub async fn get_user_info(app_state: &AppState, user_id: u32) -> anyhow::Result<UserInfo> {
    let info = db::userdb::get_user_info(app_state, user_id).await?;
    Ok(info)
}

pub async fn get_user_stats(app_state: &AppState, user_id: u32) -> anyhow::Result<UserStats> {
    let stats = db::userdb::get_user_stats(app_state, user_id).await?;
    Ok(stats)
} 
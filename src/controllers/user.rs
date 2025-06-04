use serde::{Deserialize, Serialize};

use crate::{db, AppState};

use super::course::ShortCourseInfo;

#[derive(Default, Serialize, Deserialize)]
pub struct UserInfoFull {
    pub username: String,
    pub email: String,
    pub courses: Vec<ShortCourseInfo>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct CoursesInfo {
    pub courses: Vec<ShortCourseInfo>
}

#[derive(Default, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String
}

pub async fn get_user_info_all(app_state: &AppState, user_id: u32) -> anyhow::Result<UserInfoFull> {
    let info = db::userdb::get_user_info_all(app_state, user_id).await?;
    Ok(info)
}
pub async fn get_courses_info(app_state: &AppState, user_id: u32) -> anyhow::Result<CoursesInfo> {
    let info = db::userdb::get_course_info(app_state, user_id).await?;
    Ok(info)
}
pub async fn get_user_info(app_state: &AppState, user_id: u32) -> anyhow::Result<UserInfo> {
    let info = db::userdb::get_user_info(app_state, user_id).await?;
    Ok(info)
}
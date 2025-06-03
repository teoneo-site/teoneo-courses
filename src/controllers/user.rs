use serde::{Deserialize, Serialize};

use crate::{db, AppState};

use super::course::ShortCourseInfo;

#[derive(Default, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub courses: Vec<ShortCourseInfo>,
}

pub async fn get_user_info(app_state: &AppState, user_id: u32) -> anyhow::Result<UserInfo> {
    let info = db::userdb::get_user_info(app_state, user_id).await?;
    Ok(info)
}
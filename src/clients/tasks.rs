use crate::{controllers, AppState};




pub async fn get_tasks_total(state: &AppState, course_id: i32) -> anyhow::Result<i64> {
    let total = controllers::tasks::get_tasks_total(state, course_id).await?;
    Ok(total)
}

pub async fn get_tasks_passed(state: &AppState, course_id: i32, user_id: u32) -> anyhow::Result<i64> {
    let total = controllers::tasks::get_tasks_passed(state, course_id, user_id).await?;
    Ok(total)
}
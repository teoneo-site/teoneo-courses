use crate::{controllers, BasicState};

pub async fn get_tasks_total(state: &BasicState, course_id: i32) -> anyhow::Result<i64> {
    let total = controllers::tasks::get_tasks_total(state, course_id).await?;
    Ok(total)
}

pub async fn get_tasks_passed(
    state: &BasicState,
    course_id: i32,
    user_id: u32,
) -> anyhow::Result<i64> {
    let total = controllers::tasks::get_tasks_passed(state, course_id, user_id).await?;
    Ok(total)
}

pub async fn get_started_courses(state: &BasicState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let course_started = controllers::tasks::get_courses_started(state, user_id).await?;
    Ok(course_started)
}

pub async fn get_completed_courses(state: &BasicState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let courses_completed = controllers::tasks::get_courses_completed(state, user_id).await?;
    Ok(courses_completed)
}

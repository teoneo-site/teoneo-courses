use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

use crate::{clients, db, AppState};
#[derive(Serialize, Deserialize, ToSchema, FromRow)]
pub struct BasicCourseInfo {
    pub id: i32,
    pub title: String,
    pub brief_description: String,
    pub full_description: String,
    pub tags: String,
    pub picture_url: String,
    pub price: f64,
}


#[derive(Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct ExtendedCourseInfo {
    pub id: i32,
    pub title: String,
    pub brief_description: String,
    pub full_description: String,
    pub tags: String,
    pub picture_url: String,
    pub price: f64,
    pub has_course: bool,
    pub tasks_passed: Option<i64>,
    pub tasks_total: Option<i64>,
}
#[derive(Serialize, Deserialize)]
pub struct ShortCourseInfo {
    course_id: i32,
    title: String,
    brief_description: String,
    picture_url: String,
    tasks_passed: i32,
    tasks_total: i32,
}

#[derive(Serialize, Deserialize, Default, ToSchema)]
pub struct CourseProgress {
    pub course_id: Option<i32>,
    pub tasks_passed: Option<i64>,
    pub tasks_total: Option<i64>,
}

// Currently, there is really no need for this method in the controller,
// you can just call fetch from the handler,
// BUT maybe we'll need this in future for some settings kinda stuff
pub async fn get_all_courses(pool: &AppState) -> anyhow::Result<Vec<i32>> {
    let courses = db::courses::fetch_all_courses(pool).await?;
    Ok(courses)
}

pub async fn add_course_to_favourite(
    pool: &AppState,
    user_id: u32,
    course_id: i32,
) -> anyhow::Result<()> {
    db::courses::add_course_to_favourite(pool, user_id, course_id).await
}


pub async fn get_favourite_courses(pool: &AppState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let ids = db::courses::get_favourite_courses(&pool, user_id).await?;
    Ok(ids)
}

pub async fn get_courses_by_ids(
    state: &AppState,
    ids: Vec<i32>,
    user_id: Option<u32>,
) -> anyhow::Result<Vec<ExtendedCourseInfo>> {
    let mut courses = db::courses::fetch_courses_by_ids(state, ids, user_id).await?;

    if let Some(user_id) = user_id {
        for course in &mut courses {
            if let Ok(_) = verify_ownership(state, user_id, course.id).await {
                course.has_course = true;

                if let Ok(passed) = clients::tasks::get_tasks_passed(state, course.id, user_id).await {
                    course.tasks_passed = Some(passed);
                }
            }
            if let Ok(total) = clients::tasks::get_tasks_total(state, course.id).await {
                course.tasks_total = Some(total);
            }
        }
    }

    Ok(courses)
}


pub async fn get_course_progress(
    state: &AppState,
    course_id: i32,
    user_id: u32,
) -> anyhow::Result<CourseProgress> {
    let tasks_total = clients::tasks::get_tasks_total(state, course_id).await?;
    let tasks_passed = clients::tasks::get_tasks_passed(state, course_id, user_id).await?;

    Ok(CourseProgress { 
        course_id: Some(course_id), 
        tasks_passed: Some(tasks_passed), 
        tasks_total: Some(tasks_total) 
    })
}

// Currently, there is really no need for this method in the controller,
// you can just call fetch from the handler,
// BUT maybe we'll need this in future for some settings kinda stuff
pub async fn get_course(
    state: &AppState,
    id: i32,
    user_id: Option<u32>,
) -> anyhow::Result<ExtendedCourseInfo> {
    let mut course = db::courses::fetch_course(state, id, user_id).await?;
    if let Some(user_id) = user_id {
        if let Ok(_) = verify_ownership(state, user_id, course.id).await {
            course.has_course = true;

            if let Ok(passed) = clients::tasks::get_tasks_passed(state, course.id, user_id).await {
                course.tasks_passed = Some(passed);
            }
        }
        if let Ok(total) = clients::tasks::get_tasks_total(state, course.id).await {
            course.tasks_total = Some(total);
        }
    }

    Ok(course)
}


pub async fn verify_ownership(
    state: &AppState,
    user_id: u32,
    course_id: i32,
) -> anyhow::Result<()> {
    Ok(db::courses::validate_course_ownership(state, user_id, course_id).await?)
}

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{db, AppState};
#[derive(Serialize, Deserialize, ToSchema)]
pub struct BasicCourseInfo {
    pub id: i32,
    pub title: String,
    pub brief_description: String,
    pub full_description: String,
    pub tags: Vec<String>,
    pub picture_url: String,
    pub price: f64,
}

impl BasicCourseInfo {
    pub fn new(
        id: i32,
        title: String,
        brief_description: String,
        full_description: String,
        tags: Vec<String>,
        picture_url: String,
        price: f64
    ) -> Self {
        Self {
            id,
            title,
            brief_description,
            full_description,
            tags,
            picture_url,
            price,
        }
    }
}


#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ExtendedCourseInfo {
    pub id: i32,
    pub title: String,
    pub brief_description: String,
    pub full_description: String,
    pub tags: Vec<String>,
    pub picture_url: String,
    pub price: f64,
    pub has_course: bool,
    pub tasks_passed: Option<i32>,
    pub tasks_total: Option<i32>,
}
impl ExtendedCourseInfo {
    pub fn new(
        id: i32,
        title: String,
        brief_description: String,
        full_description: String,
        tags: Vec<String>,
        picture_url: String,
        price: f64,
        has_course: bool,
        tasks_passed: Option<i32>,
        tasks_total: Option<i32>,
    ) -> Self {
        Self {
            id,
            title,
            brief_description,
            full_description,
            tags,
            picture_url,
            price,
            has_course,
            tasks_passed,
            tasks_total,
        }
    }
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
impl ShortCourseInfo {
    pub fn new(course_id: i32, title: String, brief_description: String, picture_url: String, tasks_passed: i32, tasks_total: i32) -> Self {
        Self { 
            course_id, 
            title, 
            brief_description,
            picture_url,
            tasks_passed,
            tasks_total
        }
    }
}

#[derive(Serialize, Deserialize, Default, sqlx::FromRow, ToSchema)]
pub struct CourseProgress {
    pub course_id: i32,
    pub tasks_passed: i32,
    pub tasks_total: i32,
}



// Currently, there is really no need for this method in the controller,
// you can just call fetch from the handler,
// BUT maybe we'll need this in future for some settings kinda stuff
pub async fn get_all_courses(pool: &AppState) -> anyhow::Result<Vec<i32>> {
    let courses = db::coursedb::fetch_all_courses(pool).await?;
    Ok(courses)
}

pub async fn add_course_to_favourite(pool: &AppState, user_id: u32, course_id: i32) -> anyhow::Result<()> {
    db::coursedb::add_course_to_favourite(pool, user_id, course_id).await
}
pub async fn get_favourite_courses(pool: &AppState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let ids = db::coursedb::get_favourite_courses(&pool, user_id).await?;
    Ok(ids)
}

pub async fn get_courses_by_ids_expanded(pool: &AppState, ids: Vec<i32>, user_id: u32) -> anyhow::Result<Vec<ExtendedCourseInfo>> {
    let courses = db::coursedb::fetch_courses_by_ids_expanded(pool, ids, user_id).await?;
    Ok(courses)
}

pub async fn get_courses_by_ids_basic(pool: &AppState, ids: Vec<i32>) -> anyhow::Result<Vec<BasicCourseInfo>> {
    let courses = db::coursedb::fetch_courses_by_ids_basic(pool, ids).await?;
    Ok(courses)
}

pub async fn get_course_progress(pool: &AppState, course_id: i32, user_id: u32) -> anyhow::Result<CourseProgress> {
    let progress = db::coursedb::get_course_progress(pool, user_id, course_id).await?;
    Ok(progress)
}

// Currently, there is really no need for this method in the controller,
// you can just call fetch from the handler,
// BUT maybe we'll need this in future for some settings kinda stuff
pub async fn get_course_extended(state: &AppState, id: i32, user_id: u32) -> anyhow::Result<ExtendedCourseInfo> {
    let course = db::coursedb::fetch_course_extended(state, id, user_id).await?;
    Ok(course)
}
pub async fn get_course_basic(state: &AppState, id: i32) -> anyhow::Result<BasicCourseInfo> {
    let course = db::coursedb::fetch_course_basic(state, id).await?;
    Ok(course)
}

pub async fn verify_ownership(
    state: &AppState,
    user_id: i32,
    course_id: i32,
) -> anyhow::Result<()> {
    Ok(db::coursedb::validate_course_ownership(state, user_id, course_id).await?)
}

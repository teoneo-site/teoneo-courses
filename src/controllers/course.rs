use serde::{Deserialize, Serialize};

use crate::{db, AppState};

#[derive(Serialize, Deserialize)]
pub struct CourseInfo {
    id: i32,
    title: String,
    brief_description: String,
    full_description: String,
    tags: Vec<String>,
    picture_url: String,
    price: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ShortCourseInfo {
    course_id: i32,
    title: String,
    brief_description: String
}

impl ShortCourseInfo {
    pub fn new(course_id: i32, title: String, brief_description: String) -> Self {
        Self { 
            course_id: course_id, 
            title: title, 
            brief_description: brief_description 
        }
    }
}


impl CourseInfo {
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

// Currently, there is really no need for this method in the controller,
// you can just call fetch from the handler,
// BUT maybe we'll need this in future for some settings kinda stuff
pub async fn get_all_courses(pool: &AppState) -> anyhow::Result<Vec<CourseInfo>> {
    let courses = db::coursedb::fetch_all_courses(pool).await?;
    Ok(courses)
}

pub async fn get_courses_by_ids(pool: &AppState, ids: Vec<i32>) -> anyhow::Result<Vec<CourseInfo>> {
    let courses = db::coursedb::fetch_courses_by_ids(pool, ids).await?;
    Ok(courses)
}

// Currently, there is really no need for this method in the controller,
// you can just call fetch from the handler,
// BUT maybe we'll need this in future for some settings kinda stuff
pub async fn get_course(state: &AppState, id: i32) -> anyhow::Result<CourseInfo> {
    let course = db::coursedb::fetch_course(state, id).await?;
    Ok(course)
}

pub async fn verify_ownership(
    state: &AppState,
    user_id: i32,
    course_id: i32,
) -> anyhow::Result<()> {
    Ok(db::coursedb::validate_course_ownership(state, user_id, course_id).await?)
}

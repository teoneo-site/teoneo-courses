use crate::controllers::courses::{BasicCourseInfo, CourseProgress, ExtendedCourseInfo};
use crate::AppState;

use sqlx::Row;

pub async fn fetch_courses_by_ids(
    state: &AppState,
    ids: Vec<i32>,
    _user_id: Option<u32>,  // не используется, но пусть будет
) -> anyhow::Result<Vec<ExtendedCourseInfo>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let query = format!(
        r#"
        SELECT 
            id,
            title,
            brief_description,
            full_description,
            tags,
            picture_url,
            price,
            false AS has_course,
            NULL AS tasks_total,
            NULL AS tasks_passed
        FROM courses
        WHERE id IN ({})
        "#,
        placeholders.join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, ExtendedCourseInfo>(&query);
    for id in &ids {
        query_builder = query_builder.bind(id);
    }

    let courses = query_builder.fetch_all(&state.pool).await?;
    Ok(courses)
}

pub async fn fetch_user_courses(state: &AppState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let rows = sqlx::query!(
        r#"
        SELECT c.id AS course_id
        FROM users u
        LEFT JOIN user_courses p ON p.user_id = u.id
        LEFT JOIN courses c ON p.course_id = c.id
        WHERE u.id = ?
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await?;
    let courses = rows
        .into_iter()
        .filter_map(|r| r.course_id)
        .collect();

    Ok(courses)
}

pub async fn fetch_all_courses(state: &AppState) -> anyhow::Result<Vec<i32>> {
    let courses_ids = sqlx::query_scalar!(
        "SELECT id FROM courses"
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(courses_ids)
}


pub async fn fetch_course(
    state: &AppState,
    id: i32,
    _user_id: Option<u32>,
) -> anyhow::Result<ExtendedCourseInfo> {
    let course = sqlx::query_as!(ExtendedCourseInfo, "SELECT 
            id,
            title,
            brief_description,
            full_description,
            tags,
            picture_url,
            price,
            false AS `has_course: bool`,
            NULL AS `tasks_total: i64`,
            NULL AS `tasks_passed: i64`
        FROM courses
        WHERE id = ?", id)
        .fetch_one(&state.pool)
        .await?;

    Ok(course)
}


pub async fn validate_course_ownership(
    state: &AppState,
    user_id: u32,
    course_id: i32,
) -> anyhow::Result<()> {
    sqlx::query!("SELECT * FROM user_courses WHERE user_id = ? AND course_id = ? LIMIT 1", user_id, course_id) // Limit 1 for optimization
        .fetch_one(&state.pool)
        .await?; // returns Err(RowNotFound) if no row
    Ok(()) // At this point there is a row 100% which proves ownership
}



pub async fn add_course_to_favourite(
    state: &AppState,
    user_id: u32,
    course_id: i32,
) -> anyhow::Result<()> {
    sqlx::query!("INSERT INTO favorite_courses (user_id, course_id) VALUES (?, ?)", user_id, course_id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

pub async fn get_favourite_courses(state: &AppState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let rows = sqlx::query_scalar!("SELECT course_id FROM favorite_courses WHERE user_id = ?", user_id)
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?;
    Ok(rows)
}

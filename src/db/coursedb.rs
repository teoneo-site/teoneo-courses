use crate::controllers::course::{BasicCourseInfo, CourseProgress, ExtendedCourseInfo};
use crate::AppState;

use sqlx::Row;


pub async fn fetch_courses_by_ids(
    state: &AppState,
    ids: Vec<i32>,
    user_id: Option<u32>,
) -> anyhow::Result<Vec<ExtendedCourseInfo>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let query = if user_id.is_some() {
        format!(
            r#"
            SELECT 
                c.id,
                c.title,
                c.brief_description,
                c.full_description,
                c.tags,
                c.picture_url,
                c.price,
                EXISTS (
                    SELECT 1 FROM user_courses uc 
                    WHERE uc.user_id = ? AND uc.course_id = c.id
                ) AS has_course,
                (
                    SELECT COUNT(*) 
                    FROM tasks t
                    JOIN modules m ON t.module_id = m.id
                    WHERE m.course_id = c.id
                ) AS tasks_total,
                (
                    SELECT COUNT(*) 
                    FROM task_progress tp
                    JOIN tasks t ON tp.task_id = t.id
                    JOIN modules m ON t.module_id = m.id
                    WHERE tp.user_id = ? AND tp.status = 'SUCCESS' AND m.course_id = c.id
                ) AS tasks_passed
            FROM courses c
            WHERE c.id IN ({})
            "#,
            placeholders.join(", ")
        )
    } else {
        format!(
            r#"
            SELECT 
                c.id,
                c.title,
                c.brief_description,
                c.full_description,
                c.tags,
                c.picture_url,
                c.price,
                false AS has_course,
                NULL AS tasks_total,
                NULL AS tasks_passed
            FROM courses c
            WHERE c.id IN ({})
            "#,
            placeholders.join(", ")
        )
    };

    let mut query_builder = sqlx::query_as::<_, ExtendedCourseInfo>(&query);
    if let Some(uid) = user_id {
        query_builder = query_builder.bind(uid).bind(uid);
    }

    for id in &ids {
        query_builder = query_builder.bind(id);
    }

    let courses = query_builder.fetch_all(&state.pool).await?;
    Ok(courses)
}



pub async fn fetch_all_courses(state: &AppState) -> anyhow::Result<Vec<i32>> {
    let courses_ids: Vec<i32> = sqlx::query_scalar!("SELECT id FROM courses") // Todo: Pagination with LIMIT
        .fetch_all(&state.pool)
        .await?;
    Ok(courses_ids)
}

pub async fn fetch_course(
    state: &AppState,
    id: i32,
    user_id: Option<u32>,
) -> anyhow::Result<ExtendedCourseInfo> {
    let query = if user_id.is_some() {
        r#"
        SELECT 
            c.id,
            c.title,
            c.brief_description,
            c.full_description,
            c.tags,
            c.picture_url,
            c.price,
            EXISTS (
                SELECT 1 FROM user_courses uc 
                WHERE uc.user_id = ? AND uc.course_id = c.id
            ) AS has_course,
            (
                SELECT COUNT(*) 
                FROM tasks t
                JOIN modules m ON t.module_id = m.id
                WHERE m.course_id = c.id
            ) AS tasks_total,
            (
                SELECT COUNT(*) 
                FROM task_progress tp
                JOIN tasks t ON tp.task_id = t.id
                JOIN modules m ON t.module_id = m.id
                WHERE tp.user_id = ? AND tp.status = 'SUCCESS' AND m.course_id = c.id
            ) AS tasks_passed
        FROM courses c
        WHERE c.id = ?
        "#
    } else {
        r#"
        SELECT 
            c.id,
            c.title,
            c.brief_description,
            c.full_description,
            c.tags,
            c.picture_url,
            c.price,
            false AS has_course,
            NULL AS tasks_total,
            NULL AS tasks_passed
        FROM courses c
        WHERE c.id = ?
        "#
    };

    let mut query_builder = sqlx::query_as::<_, ExtendedCourseInfo>(query);

    if let Some(uid) = user_id {
        query_builder = query_builder.bind(uid).bind(uid);
    }

    query_builder = query_builder.bind(id);

    let course = query_builder.fetch_one(&state.pool).await?;

    Ok(course)
}


pub async fn validate_course_ownership(
    state: &AppState,
    user_id: i32,
    course_id: i32,
) -> anyhow::Result<()> {
    sqlx::query!("SELECT * FROM user_courses WHERE user_id = ? AND course_id = ? LIMIT 1", user_id, course_id) // Limit 1 for optimization
        .fetch_one(&state.pool)
        .await?; // returns Err(RowNotFound) if no row
    Ok(()) // At this point there is a row 100% which proves ownership
}

pub async fn get_course_progress(
    state: &AppState,
    user_id: u32,
    course_id: i32,
) -> anyhow::Result<CourseProgress> {
    let progress = sqlx::query_as!(CourseProgress, "SELECT 
        c.id AS course_id, 
        (
            SELECT COUNT(*)
            FROM modules m
            JOIN tasks t ON t.module_id = m.id
            WHERE m.course_id = c.id
        ) AS tasks_total,
        (
            SELECT COUNT(*)
            FROM modules m
            JOIN tasks t ON t.module_id = m.id
            JOIN task_progress tp ON tp.task_id = t.id
            WHERE m.course_id = c.id AND tp.user_id = u.id AND tp.status = 'SUCCESS'
        ) AS tasks_passed
    FROM users u
    LEFT JOIN user_courses p ON p.user_id = u.id
    LEFT JOIN courses c ON p.course_id = c.id
    WHERE u.id = ? AND c.id = ?", user_id, course_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(progress)
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

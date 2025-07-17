use redis::Commands;

use crate::controllers::courses::CourseInfo;
use crate::BasicState;

pub async fn fetch_courses_by_ids(
    state: &BasicState,
    ids: Vec<i32>,
    _user_id: Option<u32>, // не используется, но пусть будет
) -> anyhow::Result<Vec<CourseInfo>> {
    let mut ids_to_fetch = Vec::new();
    let mut courses = Vec::new();

    if let Ok(mut conn) = state.redis.get() {
        for id in &ids {
            let cache_key = format!("course:{}", id);
            if let Ok(val) = conn.get::<String, String>(cache_key) {
                if let Ok(parsed_course) = serde_json::from_str::<CourseInfo>(&val) {
                    courses.push(parsed_course);
                    continue;
                }
            }
            ids_to_fetch.push(*id);
        }
    }

    if !ids_to_fetch.is_empty() {
        let placeholders: Vec<String> = ids_to_fetch.iter().map(|_| "?".to_string()).collect();
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
                is_certificated,
                false AS has_course,
                NULL AS tasks_total,
                NULL AS tasks_passed
            FROM courses
            WHERE id IN ({})
            "#,
            placeholders.join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, CourseInfo>(&query);
        for id in &ids_to_fetch {
            query_builder = query_builder.bind(id);
        }
        courses = query_builder.fetch_all(&state.pool).await?;

        if let Ok(mut conn) = state.redis.get() {
            for course in &courses {
                let cache_key = format!("course:{}", course.id);
                let course_str = serde_json::to_string(&course).unwrap();
                conn.set_ex(cache_key, course_str, 3600).unwrap_or(());
            }
        }
    }

    Ok(courses)
}

pub async fn fetch_user_courses(state: &BasicState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let cache_key = format!("courses:users:{}", user_id);
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(val) = conn.get::<String, String>(cache_key.clone()) {
            if let Ok(parsed_course) = serde_json::from_str::<Vec<i32>>(&val) {
                return Ok(parsed_course);
            }
        }
    }

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
    let courses = rows.into_iter().filter_map(|r| r.course_id).collect();

    if let Ok(mut conn) = state.redis.get() {
        let result_str = serde_json::to_string(&courses).unwrap();
        conn.set_ex(cache_key, result_str, 120).unwrap_or(()); // user may buy a course, so expire date should be short
    }

    Ok(courses)
}

pub async fn fetch_all_courses(state: &BasicState) -> anyhow::Result<Vec<i32>> {
    let cache_key = format!("courses:all");
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(val) = conn.get::<String, String>(cache_key.clone()) {
            if let Ok(parsed_course) = serde_json::from_str::<Vec<i32>>(&val) {
                return Ok(parsed_course);
            }
        }
    }

    let courses_ids = sqlx::query_scalar!("SELECT id FROM courses")
        .fetch_all(&state.pool)
        .await?;

    if let Ok(mut conn) = state.redis.get() {
        let result_str = serde_json::to_string(&courses_ids).unwrap();
        conn.set_ex(cache_key, result_str, 3600).unwrap_or(()); // user may buy a course, so expire date should be short
    }

    Ok(courses_ids)
}

pub async fn fetch_course(
    state: &BasicState,
    id: i32,
    _user_id: Option<u32>,
) -> anyhow::Result<CourseInfo> {
    let cache_key = format!("courses:{}", id);
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(val) = conn.get::<String, String>(cache_key.clone()) {
            if let Ok(parsed_course) = serde_json::from_str::<CourseInfo>(&val) {
                return Ok(parsed_course);
            }
        }
    }

    let course = sqlx::query_as!(
        CourseInfo,
        "SELECT
            id,
            title,
            brief_description,
            full_description,
            tags,
            picture_url,
            price,
            is_certificated as `is_certificated: bool`,
            false AS `has_course: bool`,
            NULL AS `tasks_total: i64`,
            NULL AS `tasks_passed: i64`
        FROM courses
        WHERE id = ?",
        id
    )
    .fetch_one(&state.pool)
    .await?;

    if let Ok(mut conn) = state.redis.get() {
        let result_str = serde_json::to_string(&course).unwrap();
        conn.set_ex(cache_key, result_str, 3600).unwrap_or(()); // user may buy a course, so expire date should be short
    }

    Ok(course)
}

pub async fn validate_course_ownership(
    state: &BasicState,
    user_id: u32,
    course_id: i32,
) -> anyhow::Result<()> {
    let cache_key = format!("ownership:{}:{}", user_id, course_id);
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(true) = conn.exists(&cache_key) {
            return Ok(());
        }
    }

    sqlx::query!(
        "SELECT * FROM user_courses WHERE user_id = ? AND course_id = ? LIMIT 1",
        user_id,
        course_id
    ) // Limit 1 for optimization
    .fetch_one(&state.pool)
    .await?; // returns Err(RowNotFound) if no rows

    if let Ok(mut conn) = state.redis.get() {
        conn.set_ex(&cache_key, "has", 120).unwrap_or(()); // Set any value, which means the row will be there
    }
    Ok(()) // At this point there is a row 100% which proves ownership
}

pub async fn add_course_to_favourite(
    state: &BasicState,
    user_id: u32,
    course_id: i32,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO favorite_courses (user_id, course_id) VALUES (?, ?)",
        user_id,
        course_id
    )
    .execute(&state.pool)
    .await?;

    if let Ok(mut conn) = state.redis.get() {
        let cache_key = format!("courses:favourite:user:{}", user_id);
        conn.del(cache_key).unwrap_or(());
    }

    Ok(())
}

pub async fn get_favourite_courses(state: &BasicState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let cache_key = format!("courses:favourite:user:{}", user_id);
    if let Ok(mut conn) = state.redis.get() {
        if let Ok(val) = conn.get::<String, String>(cache_key.clone()) {
            if let Ok(parsed_courses) = serde_json::from_str::<Vec<i32>>(&val) {
                return Ok(parsed_courses);
            }
        }
    }

    let rows = sqlx::query_scalar!(
        "SELECT course_id FROM favorite_courses WHERE user_id = ?",
        user_id
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    if let Ok(mut conn) = state.redis.get() {
        let result_str = serde_json::to_string(&rows).unwrap();
        conn.set_ex(cache_key, result_str, 300).unwrap_or(()); // user may buy a course, so expire date should be short
    }
    Ok(rows)
}

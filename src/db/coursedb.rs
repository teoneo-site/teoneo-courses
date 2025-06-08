use redis::Commands;
use sqlx::Row;
use anyhow::anyhow;
use crate::controllers::course::{CourseInfo, CourseProgress, ShortCourseInfo};
use crate::controllers::user::CoursesInfo;
use crate::{controllers, AppState};

pub async fn fetch_courses_by_ids(state: &AppState, ids: Vec<i32>) -> anyhow::Result<Vec<CourseInfo>> {
    
    let mut courses = Vec::new();
    let mut ids_to_fetch = Vec::new();


    if let Ok(mut conn) = state.redis.get() {
        for id in &ids {
            if let Ok(val) = conn.get::<String, String>(format!("course:{}", id)) {
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
            "SELECT id, title, brief_description, full_description, tags, picture_url, price FROM courses WHERE id IN ({})",
            placeholders.join(", ")
        );

        let mut query_builder = sqlx::query(&query);
        for id in &ids_to_fetch {
            query_builder = query_builder.bind(id);
        }

        let rows = query_builder.fetch_all(&state.pool).await?;


        for row in rows {
            let id: i32 = row.try_get("id")?;
            let title: String = row.try_get("title")?;
            let brief_description: String = row.try_get("brief_description")?;
            let full_description: String = row.try_get("full_description")?;
            let tags = row
                .try_get::<String, _>("tags")?
                .split(",")
                .map(|str| str.to_owned())
                .collect::<Vec<String>>();
            let picture_url: String = row.try_get("picture_url")?;
            let price: f64 = row.try_get("price")?;
            let course = CourseInfo::new(id, title, brief_description, full_description, tags, picture_url, price);
            courses.push(course);
        }

        if let Ok(mut conn) = state.redis.get() {
            for course in courses.iter() {
                let course_str = serde_json::to_string(&course).unwrap(); // Isn't supposed to fail
                conn.set_ex(format!("course:{}", course.id), course_str, 3600).unwrap_or(());
            }
        }
    }

    Ok(courses)
}

pub async fn fetch_all_courses(state: &AppState) -> anyhow::Result<Vec<CourseInfo>> {
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<&str, String>("courses:all") { // If courses are cached
            if let Ok(parsed_vec) = serde_json::from_str::<Vec<CourseInfo>>(&val) { // Get them from redis
                println!("Cachedcourses");
                return Ok(parsed_vec)
            }
        }
    }

    let rows = sqlx::query("SELECT id, title, brief_description, full_description, tags, picture_url, price FROM courses") // Todo: Pagination with LIMIT
        .fetch_all(&state.pool)
        .await?;
    let mut result = Vec::new(); // Vec of Courses
    for row in rows {
        let id: i32 = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let brief_description: String = row.try_get("brief_description")?;
        let full_description: String = row.try_get("full_description")?;
        let tags = row
            .try_get::<String, _>("tags")?
            .split(",")
            .map(|str| str.to_owned())
            .collect::<Vec<String>>(); // tags are stored like "python,ai,cock" we transform it into ["py...", "ai", "cock"]
        let picture_url: String = row.try_get("picture_url")?;
        let price: f64 = row.try_get("price")?;
        result.push(CourseInfo::new(id, title, brief_description, full_description, tags, picture_url, price));
    }   

    // If courses aren't cached -> cache them for an hour
    if let Ok(mut conn) = state.redis.get() { 
        let result_str = serde_json::to_string(&result).unwrap();  // Isn't supposed to fail
        conn.set_ex("courses:all", result_str, 3600).unwrap_or(()); // Ignore error, because we dont really care, can't afford to break when cant set smth
    }
    Ok(result)
}

pub async fn fetch_course(state: &AppState, id: i32) -> anyhow::Result<CourseInfo> {
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<String, String>(format!("course:{}", id)) {
            if let Ok(parsed_course) = serde_json::from_str::<CourseInfo>(&val) {
                return Ok(parsed_course)
            }
        }
    }

    let row = sqlx::query("SELECT title, brief_description, full_description, tags, picture_url, price FROM courses WHERE id = ?")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    let title: String = row.try_get("title")?;
    let brief_description: String = row.try_get("brief_description")?;
    let full_description: String = row.try_get("full_description")?;
    let tags = row
        .try_get::<String, _>("tags")?
        .split(",")
        .map(|str| str.to_owned())
        .collect::<Vec<String>>(); // tags are stored like "python,ai,cock" we transform it into ["py...", "ai", "cock"]
    let picture_url: String = row.try_get("picture_url")?;
    let price: f64 = row.try_get("price")?;
    let course = CourseInfo::new(id, title, brief_description, full_description, tags, picture_url, price);

    if let Ok(mut conn) = state.redis.get() { 
        let course_str = serde_json::to_string(&course).unwrap(); // Isn't supposed to fail
        conn.set_ex(format!("course:{}", id), course_str, 3600).unwrap_or(());
    }

    Ok(course)
}

pub async fn validate_course_ownership(
    state: &AppState,
    user_id: i32,
    course_id: i32,
) -> anyhow::Result<()> {
    let cache_key = format!("ownership:{}:{}", user_id, course_id);
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(true) = conn.exists(&cache_key) {
            return Ok(())
        }
    }
    
    sqlx::query("SELECT * FROM payments_history WHERE user_id = ? AND course_id = ? LIMIT 1") // Limit 1 for optimization
        .bind(user_id)
        .bind(course_id)
        .fetch_one(&state.pool)
        .await?;

    if let Ok(mut conn) = state.redis.get() { 
        conn.set_ex(&cache_key, "has", 300).unwrap_or(()); // Set any value, which means the row will be there
    }
    Ok(()) // At this point there is a row 100% which proves ownership
}


pub async fn get_course_progress(state: &AppState, user_id: u32, course_id: i32) -> anyhow::Result<CourseProgress> {
    let query = "SELECT 
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
    LEFT JOIN payments_history p ON p.user_id = u.id
    LEFT JOIN courses c ON p.course_id = c.id
    WHERE u.id = ? AND c.id = ?";
    let row = sqlx::query(query)
        .bind(user_id)
        .bind(course_id)
        .fetch_one(&state.pool)
        .await?;
    let tasks_passed: i32 = row.try_get("tasks_passed")?;
    let tasks_total: i32 = row.try_get("tasks_total")?;

    let course_progress = CourseProgress {
        course_id,
        tasks_passed,
        tasks_total
    };  
    Ok(course_progress)
}
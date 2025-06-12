use redis::Commands;
use sqlx::Row;
use crate::controllers::course::{BasicCourseInfo, CourseProgress, ExpandedCourseInfo};
use crate::AppState;

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for BasicCourseInfo {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let tags_str: String = row.try_get("tags")?;
        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            brief_description: row.try_get("brief_description")?,
            full_description: row.try_get("full_description")?,
            tags: tags_str.split(',').map(|s| s.trim().to_owned()).collect(),
            picture_url: row.try_get("picture_url")?,
            price: row.try_get("price")?,
        })
    }
}
impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for ExpandedCourseInfo {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let has_course: bool = row.try_get("has_course")?;
        let tasks_passed: Option<i32> = if has_course { row.try_get("tasks_passed")? } else { None };
        let tasks_total: Option<i32> = if has_course { row.try_get("tasks_total")? } else { None };
        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            brief_description: row.try_get("brief_description")?,
            full_description: row.try_get("full_description")?,
            tags: row
            .try_get::<String, _>("tags")?
            .split(",")
            .map(|str| str.to_owned())
            .collect::<Vec<String>>(),
            picture_url: row.try_get("picture_url")?,
            price: row.try_get("price")?,
            has_course,
            tasks_passed,
            tasks_total
        })
    }
}


pub async fn fetch_courses_by_ids_basic(state: &AppState, ids: Vec<i32>) -> anyhow::Result<Vec<BasicCourseInfo>> {
    
    let mut courses = Vec::new();
    let mut ids_to_fetch = Vec::new();

    if let Ok(mut conn) = state.redis.get() {
        for id in &ids {
            if let Ok(val) = conn.get::<String, String>(format!("course:{}", id)) {
                if let Ok(parsed_course) = serde_json::from_str::<BasicCourseInfo>(&val) {
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
        let mut query_builder = sqlx::query_as::<_, BasicCourseInfo>(&query);
        for id in &ids_to_fetch {
            query_builder = query_builder.bind(id);
        }
        courses = query_builder.fetch_all(&state.pool).await?;

        if let Ok(mut conn) = state.redis.get() {
            for course in courses.iter() {
                let course_str = serde_json::to_string(&course).unwrap(); // Isn't supposed to fail
                conn.set_ex(format!("course:{}", course.id), course_str, 3600).unwrap_or(());
            }
        }
    }

    Ok(courses)
}

pub async fn fetch_courses_by_ids_expanded(state: &AppState, ids: Vec<i32>, user_id: u32) -> anyhow::Result<Vec<ExpandedCourseInfo>> {
    
    let mut courses: Vec<ExpandedCourseInfo> = Vec::new();
    let mut ids_to_fetch = Vec::new();


    if let Ok(mut conn) = state.redis.get() {
        for id in &ids {
            if let Ok(val) = conn.get::<String, String>(format!("exp-course:{}:user:{}", id, user_id)) {
                if let Ok(parsed_course) = serde_json::from_str::<ExpandedCourseInfo>(&val) {
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
        );

        let mut query_builder = sqlx::query_as::<_, ExpandedCourseInfo>(&query)
            .bind(user_id)
            .bind(user_id);

        for id in &ids_to_fetch {
            query_builder = query_builder.bind(id);
        }

        courses = query_builder.fetch_all(&state.pool).await?;

        if let Ok(mut conn) = state.redis.get() {
            for course in courses.iter() {
                let course_str = serde_json::to_string(&course).unwrap(); // Isn't supposed to fail
                conn.set_ex(format!("exp-course:{}:user:{}", course.id, user_id), course_str, 3600).unwrap_or(());
            }
        }
    }

    Ok(courses)
}

pub async fn fetch_all_courses(state: &AppState) -> anyhow::Result<Vec<BasicCourseInfo>> {
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<&str, String>("courses:all") { // If courses are cached
            if let Ok(parsed_vec) = serde_json::from_str::<Vec<BasicCourseInfo>>(&val) { // Get them from redis
                println!("Cachedcourses");
                return Ok(parsed_vec)
            }
        }
    }
    let courses = sqlx::query_as::<_, BasicCourseInfo>("SELECT id, title, brief_description, full_description, tags, picture_url, price FROM courses") // Todo: Pagination with LIMIT
        .fetch_all(&state.pool)
        .await?;

    // If courses aren't cached -> cache them for an hour
    if let Ok(mut conn) = state.redis.get() { 
        let result_str = serde_json::to_string(&courses).unwrap();  // Isn't supposed to fail
        conn.set_ex("courses:all", result_str, 3600).unwrap_or(()); // Ignore error, because we dont really care, can't afford to break when cant set smth
    }
    Ok(courses)
}

pub async fn fetch_course(state: &AppState, id: i32) -> anyhow::Result<BasicCourseInfo> {
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<String, String>(format!("course:{}", id)) {
            if let Ok(parsed_course) = serde_json::from_str::<BasicCourseInfo>(&val) {
                return Ok(parsed_course)
            }
        }
    }

    let course = sqlx::query_as::<_, BasicCourseInfo>("SELECT title, brief_description, full_description, tags, picture_url, price FROM courses WHERE id = ?")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    
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
    
    sqlx::query("SELECT * FROM user_courses WHERE user_id = ? AND course_id = ? LIMIT 1") // Limit 1 for optimization
        .bind(user_id)
        .bind(course_id)
        .fetch_one(&state.pool)
        .await?; // returns Err(RowNotFound) if no rows

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
    LEFT JOIN user_courses p ON p.user_id = u.id
    LEFT JOIN courses c ON p.course_id = c.id
    WHERE u.id = ? AND c.id = ?";
    let progress = sqlx::query_as::<_, CourseProgress>(query)
        .bind(user_id)
        .bind(course_id)
        .fetch_one(&state.pool)
        .await?;

    Ok(progress)
}

pub async fn add_course_to_favourite(state: &AppState, user_id: u32, course_id: i32) -> anyhow::Result<()> {
    let query = "INSERT INTO favorite_courses (user_id, course_id) VALUES (?, ?)";
    sqlx::query(query)
        .bind(user_id)
        .bind(course_id)
        .execute(&state.pool).await?;
    Ok(())
}

pub async fn get_favourite_courses(state: &AppState, user_id: u32) -> anyhow::Result<Vec<i32>> {
    let query = "SELECT course_id FROM favorite_courses WHERE user_id = ?";
    let rows = sqlx::query(query)
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?;
    if rows.is_empty() {
        return Err(sqlx::Error::RowNotFound.into())
    }

    let course_ids: Vec<i32> = rows.into_iter().map(|row| row.try_get("course_id").unwrap()).collect();

    Ok(course_ids)
}
use anyhow::anyhow;
use redis::Commands;
use sqlx::Row;

use crate::controllers::course::ShortCourseInfo;
use crate::controllers::user::CoursesInfo;
use crate::controllers::user::UserInfo;
use crate::controllers::user::UserInfoFull;
use crate::AppState;


pub async fn get_user_info(state: &AppState, user_id: u32) -> anyhow::Result<UserInfo> {
    let cache_key = format!("user:{}", user_id);
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<&str, String>(&cache_key) {
            if let Ok(user_info_struct) = serde_json::from_str::<UserInfo>(&val) {
                return Ok(user_info_struct)
            }
        }
    }

    let query = "SELECT 
        u.username, 
        u.email
        FROM users u
        WHERE u.id = ?";
    let row = sqlx::query(query)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await?;

    let mut userinfo = UserInfo::default();
    userinfo.email = row.try_get("email").unwrap();
    userinfo.username = row.try_get("username").unwrap();

    if let Ok(mut conn) = state.redis.get() { 
        let result_str = serde_json::to_string(&userinfo).unwrap(); // Should not panic
        conn.set_ex(cache_key, result_str, 3600).unwrap_or(()); // Don't care if it fails
    }
    Ok(userinfo)
}

pub async fn get_user_info_all(state: &AppState, user_id: u32) -> anyhow::Result<UserInfoFull> {
    let cache_key = format!("user:info:all:{}", user_id);
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<&str, String>(&cache_key) {
            if let Ok(info_struct) = serde_json::from_str::<UserInfoFull>(&val) {
                return Ok(info_struct)
            }
        }
    }

    let query = "SELECT 
        u.username, 
        u.email, 
        c.id AS course_id, 
        c.title, 
        c.picture_url,
        c.brief_description,
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
    WHERE u.id = ?";
    let rows = sqlx::query(query)
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?;
    if rows.is_empty() {
        return Err(sqlx::Error::RowNotFound.into())
    }
    let mut userinfo = UserInfoFull::default();
    
    let first_element = rows.first().unwrap(); // We checked, that rows aren't empty
    let username: String = first_element.try_get("username").unwrap(); // It exists in the row 100%, because, rows aren't empty => user eists
    let email: String = first_element.try_get("email").unwrap(); // Same  here
    userinfo.email = email;
    userinfo.username = username;

    let mut courses = vec![];

    for row in rows.into_iter() {
        let course_id: Option<i32> = row.try_get("course_id")?;
        let title: Option<String> = row.try_get("title")?;
        let brief_description: Option<String> = row.try_get("brief_description")?;
        let picture_url: Option<String> = row.try_get("picture_url")?;
        let tasks_passed: Option<i32> = row.try_get("tasks_passed")?;
        let tasks_total: Option<i32> = row.try_get("tasks_total")?;

        if let (Some(course_id), Some(title), Some(brief_description), Some(picture_url), Some(tasks_passed), Some(tasks_total)) =
            (course_id, title, brief_description, picture_url, tasks_passed, tasks_total)
        {
            courses.push(ShortCourseInfo::new(course_id, title, brief_description, picture_url, tasks_passed, tasks_total));
        }
    }
    userinfo.courses = courses;

    if let Ok(mut conn) = state.redis.get() { 
        let result_str = serde_json::to_string(&userinfo).unwrap(); // Should not panic
        conn.set_ex(cache_key, result_str, 3600).unwrap_or(());
    }

    Ok(userinfo)
}


pub async fn get_course_info(state: &AppState, user_id: u32) -> anyhow::Result<CoursesInfo> {
    let cache_key = format!("user:info:courses:{}", user_id);
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<&str, String>(&cache_key) {
            if let Ok(courses_info_struct) = serde_json::from_str::<CoursesInfo>(&val) {
                return Ok(courses_info_struct)
            }
        }
    }

    let query = "SELECT 
        c.id AS course_id, 
        c.title, 
        c.brief_description,
        c.picture_url
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
    WHERE u.id = ?";
    let rows = sqlx::query(query)
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?;

    if rows.is_empty() {
        return Err(anyhow!("User does not exist"))
    }
    let mut coursesinfo = CoursesInfo::default();
    
    let mut courses = vec![];
    for row in rows.into_iter() {
        let course_id: Option<i32> = row.try_get("course_id")?;
        let title: Option<String> = row.try_get("title")?;
        let brief_description: Option<String> = row.try_get("brief_description")?;
        let picture_url: Option<String> = row.try_get("picture_url")?;
        let tasks_passed: Option<i32> = row.try_get("tasks_passed")?;
        let tasks_total: Option<i32> = row.try_get("tasks_total")?;

        if let (Some(course_id), Some(title), Some(brief_description), Some(picture_url), Some(tasks_passed), Some(tasks_total)) =
            (course_id, title, brief_description, picture_url, tasks_passed, tasks_total)
        {
            courses.push(ShortCourseInfo::new(course_id, title, brief_description, picture_url, tasks_passed, tasks_total));
        }
    }
    coursesinfo.courses = courses;
        
    if let Ok(mut conn) = state.redis.get() { 
        let result_str = serde_json::to_string(&coursesinfo).unwrap();
        conn.set_ex(cache_key, result_str, 3600).unwrap_or(());
    }
    Ok(coursesinfo)
}
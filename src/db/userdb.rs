use anyhow::anyhow;
use redis::Commands;
use sqlx::Row;

use crate::controllers;
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
        conn.set_ex(cache_key, result_str, 300).unwrap_or(()); // Don't care if it fails
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
    LEFT JOIN user_courses p ON p.user_id = u.id
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
        conn.set_ex(cache_key, result_str, 300).unwrap_or(());
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
        c.picture_url,
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
        conn.set_ex(cache_key, result_str, 120).unwrap_or(());
    }
    Ok(coursesinfo)
}

pub async fn get_user_stats(state: &AppState, user_id: u32) -> anyhow::Result<controllers::user::UserStats> {
    let cache_key = format!("user:stats:{}", user_id);
    if let Ok(mut conn) = state.redis.get() { 
        if let Ok(val) = conn.get::<&str, String>(&cache_key) {
            if let Ok(stats_info) = serde_json::from_str::<controllers::user::UserStats>(&val) {
                return Ok(stats_info)
            }
        }
    }

    let query = "
        SELECT 
            (SELECT COUNT(DISTINCT course_id) 
             FROM user_courses 
             WHERE user_id = ?) AS courses_owned,
            (SELECT COUNT(DISTINCT m.course_id) 
             FROM task_progress tp
             JOIN tasks t ON tp.task_id = t.id
             JOIN modules m ON t.module_id = m.id
             WHERE tp.user_id = ?) AS courses_started,
            (SELECT COUNT(DISTINCT m.course_id)
             FROM modules m
             JOIN (
                 SELECT t.module_id, COUNT(*) as total_tasks
                 FROM tasks t
                 GROUP BY t.module_id
             ) t ON m.id = t.module_id
             JOIN (
                 SELECT t.module_id, COUNT(*) as completed_tasks
                 FROM task_progress tp
                 JOIN tasks t ON tp.task_id = t.id
                 WHERE tp.user_id = ? AND tp.status = 'SUCCESS'
                 GROUP BY t.module_id
             ) tc ON m.id = tc.module_id
             WHERE t.total_tasks = tc.completed_tasks
             GROUP BY m.course_id
             HAVING COUNT(DISTINCT m.id) = (
                 SELECT COUNT(*) 
                 FROM modules m2 
                 WHERE m2.course_id = m.course_id
             )) AS courses_completed
    ";
    let row = sqlx::query(query)
        .bind(user_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await?;
    let courses_owned: i64 = row.try_get("courses_owned")?;
    let courses_started: i64 = row.try_get("courses_started")?;
    let courses_completed: i64 = row.try_get("courses_completed").unwrap_or(0);
    let info = controllers::user::UserStats { courses_owned, courses_started, courses_completed };
    if let Ok(mut conn) = state.redis.get() { 
        let result_str = serde_json::to_string(&info).unwrap();
        conn.set_ex(cache_key, result_str, 120).unwrap_or(());
    }
    Ok(info)
}
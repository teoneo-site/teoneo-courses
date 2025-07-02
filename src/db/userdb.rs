use anyhow::anyhow;
use sqlx::Row;

use crate::controllers;
use crate::controllers::user::UserInfo;
use crate::controllers::user::UserInfoFull;
use crate::AppState;

pub async fn get_user_info(state: &AppState, user_id: u32) -> anyhow::Result<UserInfo> {

    let query = "SELECT 
        u.username, 
        u.email
        FROM users u
        WHERE u.id = ?";
    let row = sqlx::query_as::<_, (String, String)>(query)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await?;

    let mut userinfo = UserInfo::default();
    userinfo.username = row.0;
    userinfo.email = row.1;

    Ok(userinfo)
}

pub async fn get_user_info_all(state: &AppState, user_id: u32) -> anyhow::Result<UserInfoFull> {

    let query = "SELECT 
        u.username, 
        u.email, 
        c.id AS course_id
    FROM users u
    LEFT JOIN user_courses p ON p.user_id = u.id
    LEFT JOIN courses c ON p.course_id = c.id
    WHERE u.id = ?";
    let rows = sqlx::query(query)
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?;
    if rows.is_empty() {
        return Err(sqlx::Error::RowNotFound.into());
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
        if let Some(course_id) = course_id {
            courses.push(course_id);
        }
    }
    userinfo.courses = courses;

    Ok(userinfo)
}

pub async fn get_courses_info(state: &AppState, user_id: u32) -> anyhow::Result<Vec<i32>> {

    let query = "SELECT 
        c.id AS course_id
    FROM users u
    LEFT JOIN user_courses p ON p.user_id = u.id
    LEFT JOIN courses c ON p.course_id = c.id
    WHERE u.id = ?";
    let rows = sqlx::query(query)
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?;

    let mut courses = vec![];
    for row in rows.into_iter() {
        let course_id: Option<i32> = row.try_get("course_id")?;
        if let Some(course_id) = course_id {
            courses.push(course_id);
        }
    }
    Ok(courses)
}

pub async fn get_user_stats(
    state: &AppState,
    user_id: u32,
) -> anyhow::Result<controllers::user::UserStats> {

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
             ))s AS courses_completed
    ";
    let row = sqlx::query_as::<_, (i64, i64, Option<i64>)>(query)
        .bind(user_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await?;

    let info = controllers::user::UserStats {
        courses_owned: row.0,
        courses_started: row.1,
        courses_completed: row.2.unwrap_or(0),
    };
    Ok(info)
}

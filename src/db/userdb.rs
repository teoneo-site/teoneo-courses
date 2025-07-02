use anyhow::anyhow;
use sqlx::Row;

use crate::controllers;
use crate::controllers::user::UserInfo;
use crate::controllers::user::UserInfoFull;
use crate::controllers::user::UserStats;
use crate::AppState;

pub async fn get_user_info(state: &AppState, user_id: u32) -> anyhow::Result<UserInfo> {
    let row = sqlx::query_as!(
        UserInfo,
        r#"
        SELECT username, email
        FROM users
        WHERE id = ?
        "#,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(row)
}

pub async fn get_user_info_all(state: &AppState, user_id: u32) -> anyhow::Result<UserInfoFull> {
    // Используем compile-time query, но т.к. возвращаем несколько строк — делаем query! и вручную маппим
    let rows = sqlx::query!(
        r#"
        SELECT 
            u.username, 
            u.email, 
            c.id AS course_id
        FROM users u
        LEFT JOIN user_courses p ON p.user_id = u.id
        LEFT JOIN courses c ON p.course_id = c.id
        WHERE u.id = ?
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await?;

    if rows.is_empty() {
        return Err(sqlx::Error::RowNotFound.into());
    }

    let first = &rows[0];
    let mut userinfo = UserInfoFull::default();
    userinfo.username = first.username.clone();
    userinfo.email = first.email.clone();

    userinfo.courses = rows
        .iter()
        .filter_map(|r| r.course_id)
        .collect();

    Ok(userinfo)
}

pub async fn get_courses_info(state: &AppState, user_id: u32) -> anyhow::Result<Vec<i32>> {
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

pub async fn get_user_stats(
    state: &AppState,
    user_id: u32,
) -> anyhow::Result<UserStats> {
    let row = sqlx::query_as!(
        UserStats,
        r#"
        SELECT 
            (SELECT COUNT(DISTINCT course_id) 
             FROM user_courses 
             WHERE user_id = ?) AS courses_owned,
            (SELECT COUNT(DISTINCT m.course_id) 
             FROM task_progress tp
             JOIN tasks t ON tp.task_id = t.id
             JOIN modules m ON t.module_id = m.id
             WHERE tp.user_id = ?) AS courses_started,
            (
             SELECT COUNT(DISTINCT m.course_id)
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
             )
            ) AS courses_completed
        "#,
        user_id,
        user_id,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(row)
}

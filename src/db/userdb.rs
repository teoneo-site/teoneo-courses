use anyhow::anyhow;
use sqlx::Row;

use crate::controllers::course::ShortCourseInfo;
use crate::controllers::user::CoursesInfo;
use crate::controllers::user::UserInfo;
use crate::controllers::user::UserInfoFull;
use crate::AppState;


pub async fn get_user_info(appstate: &AppState, user_id: u32) -> anyhow::Result<UserInfo> {
    let query = "SELECT 
        u.username, 
        u.email
        FROM users u
        WHERE u.id = ?";
    let row = sqlx::query(query)
        .bind(user_id)
        .fetch_one(&appstate.pool)
        .await?;
    let mut userinfo = UserInfo::default();
    let username: String = row.try_get("username").unwrap(); // It exists in the row 100%, because, rows aren't empty => user eists
    let email: String = row.try_get("email").unwrap(); // Same  here
    userinfo.email = email;
    userinfo.username = username;

    Ok(userinfo)
}

pub async fn get_user_info_all(appstate: &AppState, user_id: u32) -> anyhow::Result<UserInfoFull> {
    let query = "SELECT 
        u.username, 
        u.email, 
        c.id AS course_id, 
        c.title, 
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
        .fetch_all(&appstate.pool)
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
        let tasks_passed: Option<i32> = row.try_get("tasks_passed")?;
        let tasks_total: Option<i32> = row.try_get("tasks_total")?;

        if let (Some(course_id), Some(title), Some(brief_description), Some(tasks_passed), Some(tasks_total)) =
            (course_id, title, brief_description, tasks_passed, tasks_total)
        {
            courses.push(ShortCourseInfo::new(course_id, title, brief_description, tasks_passed, tasks_total));
        }
    }


    userinfo.courses = courses;
    Ok(userinfo)
}


pub async fn get_course_info(appstate: &AppState, user_id: u32) -> anyhow::Result<CoursesInfo> {
    let query = "SELECT 
        c.id AS course_id, 
        c.title, 
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
        .fetch_all(&appstate.pool)
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
        let tasks_passed: Option<i32> = row.try_get("tasks_passed")?;
        let tasks_total: Option<i32> = row.try_get("tasks_total")?;

        if let (Some(course_id), Some(title), Some(brief_description), Some(tasks_passed), Some(tasks_total)) =
            (course_id, title, brief_description, tasks_passed, tasks_total)
        {
            courses.push(ShortCourseInfo::new(course_id, title, brief_description, tasks_passed, tasks_total));
        }
    }
    coursesinfo.courses = courses;

    Ok(coursesinfo)
}
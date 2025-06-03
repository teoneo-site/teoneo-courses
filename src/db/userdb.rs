use anyhow::anyhow;
use redis::Commands;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::controllers::course::ShortCourseInfo;
use crate::controllers::user::UserInfo;
use crate::AppState;



pub async fn get_user_info(appstate: &AppState, user_id: u32) -> anyhow::Result<UserInfo> {
    let query = "SELECT u.username, u.email, c.id AS course_id, c.title, c.brief_description 
        FROM users u 
        LEFT JOIN payments_history p ON p.user_id = u.id 
        LEFT JOIN courses c ON p.course_id = c.id WHERE u.id = ?";
    let rows = sqlx::query(query)
        .bind(user_id)
        .fetch_all(&appstate.pool)
        .await?;

    if rows.is_empty() {
        return Err(anyhow!("User does not exist"))
    }
    let mut userinfo = UserInfo::default();
    
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

        if course_id.is_some() && title.is_some() && brief_description.is_some() { // If user subscribed to some course, add it
            courses.push(ShortCourseInfo::new(course_id.unwrap(), title.unwrap(), brief_description.unwrap()));
        }
    }

    userinfo.courses = courses;
    Ok(userinfo)
}
use sqlx::mysql::MySqlArguments;

use crate::{controllers::certs::{CertInfo, CertStatus}, BasicState};
use sqlx::Arguments;

pub async fn add_certs(
    state: &BasicState,
    user_id: u32,
    course_ids: Vec<i32>,
) -> anyhow::Result<()> {
    if course_ids.is_empty() {
        return Ok(());
    }

    let mut query = String::from("INSERT IGNORE INTO user_certs (user_id, course_id, status) VALUES ");
    let mut args = MySqlArguments::default();

    for (i, course_id) in course_ids.iter().enumerate() {
        if i > 0 {
            query.push_str(", ");
        }
        query.push_str("(?, ?, 'NOT_CREATED')");
        args.add(user_id);
        args.add(course_id);
    }

    sqlx::query_with(&query, args)
        .execute(&state.pool)
        .await?;
    Ok(())
}


pub async fn get_certs(state: &BasicState, user_id: u32) -> anyhow::Result<Vec<CertInfo>> {
    let certs = sqlx::query_as!(CertInfo, "SELECT 
    user_certs.id,
    courses.title AS course_title,
    user_certs.status
        FROM user_certs
        JOIN courses ON courses.id = user_certs.course_id
        WHERE user_certs.user_id = ?
    ", user_id)
        .fetch_all(&state.pool)
        .await?;
    Ok(certs)
}

pub async fn set_cert_status(state: &BasicState, cert_id: i32, status: CertStatus) -> anyhow::Result<()> {
    sqlx::query!("UPDATE user_certs SET status = ? WHERE id = ?", status.to_string(), cert_id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

pub async fn get_cert(state: &BasicState, cert_id: i32) -> anyhow::Result<CertInfo> {
    let cert = sqlx::query_as!(CertInfo, "SELECT 
    user_certs.id,
    courses.title AS course_title,
    user_certs.status
        FROM user_certs
        JOIN courses ON courses.id = user_certs.course_id
        WHERE user_certs.id = ?
    ", cert_id)
        .fetch_one(&state.pool)
        .await?;
    Ok(cert)
}


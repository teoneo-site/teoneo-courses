use std::fmt::Display;

use axum::body::Bytes;
use minio::s3::{builders::ObjectContent, segmented_bytes::SegmentedBytes, types::S3Api};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{clients, common::token::AuthHeader, controllers, db, BasicState};

const CERTS_BUCKET: &str = "user-certs";


#[derive(Serialize, Deserialize, ToSchema)]
pub enum CertStatus {
    Created,
    NotCreated
}

impl From<String> for CertStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "NOT_CREATED" => Self::NotCreated,
            "CREATED" => Self::Created,
            _ => Self::NotCreated
        }
    }
}
impl Display for CertStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "CREATED"),
            Self::NotCreated => write!(f, "NOT_CREATED")
        }
    }
}


#[derive(Serialize, Deserialize, ToSchema)]
pub struct CertInfo {
    pub id: i32,
    pub course_title: String,
    pub status: CertStatus
}




pub async fn get_certs(state: &BasicState, user_id: u32) -> anyhow::Result<Vec<CertInfo>> {
    let completed_courses = clients::tasks::get_completed_courses(state, user_id).await?;
    db::certs::add_certs(state, user_id, completed_courses).await?; // Shall not fail, since INSERT IGNORE fails silently

    let certs = db::certs::get_certs(state, user_id).await?;
    Ok(certs)
}


pub async fn create_cert(state: &BasicState, s3: minio::s3::Client, http_client: &reqwest::Client, auth_token: AuthHeader, cert_id: i32) -> anyhow::Result<()> {
    // User info
    let user_info = clients::auth::get_user_info(&http_client, &auth_token.token).await?;
    let cert_info = db::certs::get_cert(state, cert_id).await?;

    // Create pdf file
    let pdf_file_contens = user_info.username.as_bytes();

    println!("{}", pdf_file_contens.len());
    // 34/certs/23 # user 34 cert_id = 23
    // Saved the file, no need for record in db, since you can easily construct `object` from user_id and cert
    let object_str = format!("{}/certs/{}-certificate.pdf", auth_token.claims.id, cert_info.course_title);
    let _ = s3.put_object(CERTS_BUCKET, object_str, Bytes::copy_from_slice(&pdf_file_contens).into()).send().await?;

    // Change status
    db::certs::set_cert_status(state, cert_id, CertStatus::Created).await?;
    Ok(())
}

pub async fn get_cert_file(state: &BasicState, s3: minio::s3::Client, cert_id: i32, user_id: u32) -> anyhow::Result<ObjectContent> {
    let cert_info = db::certs::get_cert(&state, cert_id).await?;

    let object_str = format!("{}/certs/{}-certificate.pdf", user_id, cert_info.course_title);
    let resp: minio::s3::response::GetObjectResponse = s3.get_object(CERTS_BUCKET, object_str).send().await?;

    Ok(resp.content)
}
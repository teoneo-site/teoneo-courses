use axum::http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{clients, BasicState};

#[derive(Default, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
}

pub async fn get_user_info(client: &reqwest::Client, auth_token: &str) -> anyhow::Result<UserInfo> {
    let endpoint = std::env::var("AUTH_SERVICE_URL").unwrap() + &format!("/user/info?value=user");

    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", auth_token))?);

    let resp = clients::request::get_request_with_headers::<UserInfo>(client, &endpoint, headers).await?;
    Ok(resp)
}
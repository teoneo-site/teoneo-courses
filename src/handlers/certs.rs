use axum::{extract::{Path, State}, http::{HeaderMap, HeaderValue, StatusCode}, response::{IntoResponse, Response}, Json};
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{common::{error::{AppError, ErrorResponse, ErrorTypes}, token::AuthHeader}, controllers::{self, certs::CertInfo}, db, error_response, AppState, BasicState};




#[utoipa::path(
    get,
    description = "Возвращает список сертификатов",
    path = "/certificates",
    params(
        ("Authorization" = String, Header, description = "JWT"),
    ),
    responses(
        (status = 200, description = "Успешно", body = Vec<CertInfo>),
        (status = 500, description = "Что-то случилось", body = ErrorResponse),
    )
)]
pub async fn get_certs(State(state): State<BasicState>, auth_header: AuthHeader) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;
    let certs = controllers::certs::get_certs(&state, user_id).await?;

    let json = json!({
        "data": certs
    });
    Ok((StatusCode::OK, axum::Json(json)).into_response())
}

#[derive(Deserialize, ToSchema)]
pub struct CreateCertReq {
    pub id: i32, // id of cert from user_certs
}

#[utoipa::path(
    post,
    description = "Создать педофайл сертификата",
    path = "/certificates",
    params(
        ("Authorization" = String, Header, description = "JWT"),
    ),
    request_body = CreateCertReq,
    responses(
        (status = 200, description = "Успешно"),
        (status = 500, description = "Что-то случилось", body = ErrorResponse),
    )
)]
pub async fn create_cert(State(state): State<BasicState>, State(s3): State<minio::s3::Client>, State(http_client): State<reqwest::Client>, auth_header: AuthHeader, Json(data) : Json<CreateCertReq>) -> Result<Response, AppError> {
    controllers::certs::create_cert(&state, s3, &http_client, auth_header, data.id).await?;
    Ok((StatusCode::OK).into_response())
}

#[utoipa::path(
    get,
    description = "Получить педофайл сертификата. Возвращает файл",
    path = "/certificates/{cert_id}",
    params(
        ("Authorization" = String, Header, description = "JWT"),
        ("cert_id" = i32, Path, description = "Айдишник сертификата")
    ),
    responses(
        (status = 200, description = "Успешно", body = String),
        (status = 500, description = "Что-то случилось", body = ErrorResponse),
    )
)]
pub async fn get_cert_file(State(state): State<BasicState>, State(s3): State<minio::s3::Client>, auth_header: AuthHeader, Path(cert_id): Path<i32>) -> Result<Response, AppError> {
    match controllers::certs::get_cert_file(&state, s3, cert_id, auth_header.claims.id).await {
        Ok(obj_content) => {
            let mut headers = HeaderMap::with_capacity(2);
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
            headers.insert(CONTENT_DISPOSITION, HeaderValue::from_str(format!("attachment; filename=\"cert.pdf\"").as_str()).unwrap());

            let (stream, _) = obj_content.to_stream().await?;
            let body = axum::body::Body::from_stream(stream);

            return Ok((
                StatusCode::OK,
                headers,
                body
            ).into_response())
        },
        Err(why) => {
            tracing::error!("Such file doesnt exist: {}", why);
            return Ok(error_response!(
                StatusCode::NOT_FOUND,
                ErrorTypes::InternalError,
                "Such file does not exist"
            ));
        }
    }
}
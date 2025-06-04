
/*
JSON:
 data: { 
    username
    email
    courses: {
        id,
        title, 
        brief_description
    }
}
*/

use axum::{extract::{Path, Query, State}, http::StatusCode, response::{IntoResponse, Response}};
use axum_extra::extract::OptionalQuery;
use serde::{Deserialize, Serialize};

use crate::{common::token::Claims, controllers, handlers::{self, ErrorTypes}, AppState};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueInfo {
    Courses,
    User,
    All,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfoQuery {
    value: ValueInfo
}

pub async fn get_user_info_and_courses(
    State(state): State<AppState>,
    claims: Claims,
    Query(value) : Query<UserInfoQuery>,
) -> Result<Response, Response> {
    let user_id = claims.id;
    let mut json_obj = serde_json::Value::Object(serde_json::Map::new());

    match value.value {
        ValueInfo::All => {
            match controllers::user::get_user_info_all(&state, user_id).await {
                Ok(info) => {
                    json_obj["data"] = serde_json::to_value(&info).unwrap(); // Why would it panic?
                    return Ok((StatusCode::OK, axum::Json(json_obj)).into_response())
                },
                Err(why) => {
                    eprintln!("Why failed: {}", why);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Something happened",
                        )), // Should not panic, because struct is always valid for converting into JSON
                    )
                        .into_response());
                }
            }   
        }
        ValueInfo::Courses => {
            match controllers::user::get_courses_info(&state, user_id).await {
                Ok(info) => {
                    json_obj["data"] = serde_json::to_value(&info).unwrap(); // Why would it panic?
                    return Ok((StatusCode::OK, axum::Json(json_obj)).into_response())
                },
                Err(why) => {
                    eprintln!("Why failed: {}", why);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Something happened",
                        )), // Should not panic, because struct is always valid for converting into JSON
                    )
                        .into_response());
                }
            }
        }
        ValueInfo::User => {
            match controllers::user::get_user_info(&state, user_id).await {
                Ok(info) => {
                    json_obj["data"] = serde_json::to_value(&info).unwrap(); // Why would it panic?
                    return Ok((StatusCode::OK, axum::Json(json_obj)).into_response())
                },
                Err(why) => {
                    eprintln!("Why failed: {}", why);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(handlers::ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Something happened",
                        )), // Should not panic, because struct is always valid for converting into JSON
                    )
                        .into_response());
                }
            }
        }
    }
   
}
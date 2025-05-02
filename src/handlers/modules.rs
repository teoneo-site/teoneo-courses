use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    common, controllers,
    handlers::{self, ErrorTypes},
};

// PUBLCI GET /course/{course_id}/modules - Get info course's modules (id, course_id, title)
pub async fn get_modules_for_course(
    State(state): State<MySqlPool>,
    Path(course_id): Path<i32>,
) -> Result<Response, Response> {
    match controllers::module::get_modules_for_course(&state, course_id).await {
        Ok(modules) => {
            let mut json: serde_json::value::Value = serde_json::Value::Null;
            json["data"] = serde_json::Value::Array([].to_vec());

            for module in modules.into_iter() {
                if let Some(data_array) = json["data"].as_array_mut() {
                    let mut value: serde_json::Value =
                        serde_json::Value::Object(serde_json::Map::new());
                    value["id"] = module.id.into();
                    value["course_id"] = module.course_id.into();
                    value["title"] = module.title.into();
                    data_array.push(value);
                }
            }

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Ok((StatusCode::OK, headers, json.to_string()).into_response());
        }
        Err(why) => {
            eprintln!("Why: {}", why);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Err((
                StatusCode::BAD_REQUEST,
                headers,
                serde_json::to_string_pretty(&handlers::ErrorResponse::new(
                    &ErrorTypes::InternalError.to_string(),
                    "Could not fetch modules",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}

// PUBLCI GET /course/{course_id} - Get info about a single course
pub async fn get_module(
    State(state): State<MySqlPool>,
    headers: HeaderMap,
    Path((course_id, module_id)): Path<(i32, i32)>,
) -> Result<Response, Response> {
    let empty = HeaderValue::from_static("");
    let token = headers
        .get("Authorization")
        .unwrap_or(&empty)
        .to_str()
        .unwrap_or("")
        .split(" ")
        .last()
        .unwrap_or("");

    let is_subscribed_to_course = match common::token::verify_jwt_token(token) {
        Ok(user_id) => {
            // Check ownership TODO: API for verifying ownership of a course
            false
        }
        Err(why) => {
            true // Temporary!!!!! TODO: Remove

            // eprintln!("Why: {}", why);
            // return Err((
            //     StatusCode::UNAUTHORIZED, // Means refresh jwt token, not exit
            //     serde_json::to_string_pretty(&handlers::ErrorResponse::new(
            //         &ErrorTypes::JwtTokenExpired.to_string(),
            //         "Update JWT token on frontend",
            //     ))
            //     .unwrap(),
            // )
            //     .into_response());
        }
    };

    match controllers::module::get_module(&state, course_id, module_id).await {
        Ok(module) => {
            let body = if is_subscribed_to_course {
                json!({
                    "data": module,
                })
                .to_string()
            } else {
                let mut value: serde_json::Value = serde_json::Value::Object(serde_json::Map::new());
                value["id"] = module_id.into();
                value["course_id"] = course_id.into();
                value["title"] = module.title.into();
                value["description"] = module.description.into();

                json!({
                    "data": value
                }).to_string()
            };
            
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Ok((StatusCode::OK, headers, body).into_response());
        }
        Err(why) => {
            eprintln!("Why: {}", why);

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Err((
                StatusCode::BAD_REQUEST,
                headers,
                serde_json::to_string_pretty(&handlers::ErrorResponse::new(
                    &ErrorTypes::InternalError.to_string(),
                    "Could not fetch the module",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}

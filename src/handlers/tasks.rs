use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{common, controllers::{self, task::{QuizTask, TaskType}}, handlers::{self, ErrorTypes}};




pub async fn get_tasks_for_module(
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
            true
        }
        Err(why) => { // Since it aint working rn we comment it
            false
            // eprintln!("Why: {}", why);
            // let mut headers = HeaderMap::new();
            // headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            // return Err((StatusCode::UNAUTHORIZED, headers, serde_json::to_string_pretty(&handlers::ErrorResponse::new(
            //     &ErrorTypes::JwtTokenExpired.to_string(),
            //     "Token update requested",
            // ))
            // .unwrap()).into_response())
        }
    };

    match controllers::task::get_tasks_for_module(&state, module_id).await {
        Ok(tasks) => {
            let body = json!({
                "data": tasks,
            })
            .to_string();

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
                    "Could not fetch tasks",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}



pub async fn get_task(
    State(state): State<MySqlPool>,
    headers: HeaderMap,
    Path((course_id, module_id, task_id)): Path<(i32, i32, i32)>,
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
            true
        }
        Err(why) => { // Since it aint working rn we comment it
            false
            // eprintln!("Why: {}", why);
            // let mut headers = HeaderMap::new();
            // headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            // return Err((StatusCode::UNAUTHORIZED, headers, serde_json::to_string_pretty(&handlers::ErrorResponse::new(
            //     &ErrorTypes::JwtTokenExpired.to_string(),
            //     "Token update requested",
            // ))
            // .unwrap()).into_response())
        }
    };

    match controllers::task::get_task(&state, module_id, task_id).await {
        Ok(mut task) => {
            if task.task_type == TaskType::Quiz {
                let mut quiz: QuizTask = serde_json::from_value(task.content).unwrap();
                quiz.answers.clear();
                task.content = serde_json::to_value(quiz).unwrap();
            } 
            
            let body = json!({
                "data": task,
            })
            .to_string();

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
                    "Could not fetch the task",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}


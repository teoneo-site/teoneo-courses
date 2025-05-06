use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    common,
    controllers::{
        self,
        progress::ProgressStatus,
        task::{QuizTask, QuizUserAnswer, TaskType},
    },
    db,
    handlers::{self, ErrorTypes},
};

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
        Err(why) => {
            // Since it aint working rn we comment it
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
            eprintln!("Why failed: {}", why);

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
        Err(why) => {
            // Since it aint working rn we comment it
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
        Ok(task) => {
            let body = json!({
                "data": task,
            })
            .to_string();

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Ok((StatusCode::OK, headers, body).into_response());
        }
        Err(why) => {
            eprintln!("Why task: {}", why);

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

#[derive(Serialize, Deserialize)]
pub struct SubmitPayload {
    pub data: serde_json::Value, // Which can be either QuizUserAnswer or MatchingUserAnswer
}

// POST /course/.../modules/.../tasks/.../submit
pub async fn submit_task(
    State(state): State<MySqlPool>,
    headers: HeaderMap,
    Path((course_id, module_id, task_id)): Path<(i32, i32, i32)>, // We dont really need module_id tho, just course (not necessary and)
    Json(user_answers): Json<serde_json::Value>,
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

    let user_id = match common::token::verify_jwt_token(token) {
        // TODO: Move to utilities (it repeats a lot)
        Ok(user_id) => user_id,
        Err(why) => {
            6 // test user id (exists in table)
            // Since it aint working rn we comment it
            // egprintln!("Why: {}", why);
            // let mut headers = HeaderMap::new();
            // headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            // return Err((StatusCode::UNAUTHORIZED, headers, serde_json::to_string_pretty(&handlers::ErrorResponse::new(
            //     &ErrorTypes::JwtTokenExpired.to_string(),
            //     "Token update requested",
            // ))
            // .unwrap()).into_response())
        }
    };
    let is_subscribe_to_course = false; // TODO: Validation

    let task_type = match db::taskdb::fetch_task_type(&state, task_id).await {
        Ok(task_type) => task_type,
        Err(why) => {
            eprintln!("Why: {}", why);
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Err((
                StatusCode::BAD_REQUEST,
                headers,
                serde_json::to_string_pretty(&handlers::ErrorResponse::new(
                    &ErrorTypes::InternalError.to_string(),
                    "Could not fetch the task type. Task doesnt exist",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };

    // Insert EVAL progress status
    // Frontend can query status at this point
    controllers::progress::update_or_insert_status(
        &state,
        user_id,
        task_id,
        ProgressStatus::Eval,
        "".to_string(),
        0.0,
        0,
    )
    .await
    .unwrap(); // Careful

    match task_type {
        TaskType::Quiz => {
            let answers_str = db::taskdb::fetch_task_answers(&state, task_type, task_id)
                .await
                .unwrap(); // TODO: Handle
            let task_answers: Vec<u8> = answers_str
                .split(";")
                .map(|element| element.parse::<u8>().unwrap_or(0))
                .collect();
            let user_answers: QuizUserAnswer =
                serde_json::from_value(user_answers["data"].clone()).unwrap(); // TODO: Handle

            if task_answers.len() != user_answers.answers.len()
                || task_answers
                    .iter()
                    .zip(&user_answers.answers)
                    .filter(|&(a, b)| a == b)
                    .count()
                    != task_answers.len()
            {
                controllers::progress::update_or_insert_status(
                    &state,
                    user_id,
                    task_id,
                    ProgressStatus::Failed,
                    serde_json::to_string(&user_answers).unwrap(),
                    0.0,
                    1,
                )
                .await
                .unwrap(); // Careful
            } else {
                // Set status to SUCCESSS, submission to user_answers, score to 1.0, attempts to 1 if exists + 1
                controllers::progress::update_or_insert_status(
                    &state,
                    user_id,
                    task_id,
                    ProgressStatus::Success,
                    serde_json::to_string(&user_answers).unwrap(),
                    1.0,
                    1,
                )
                .await
                .unwrap(); // Careful
            }

            return Ok((StatusCode::ACCEPTED).into_response());
        }
        TaskType::Lecture => {}
        TaskType::Prompt => {}
    };

    return Ok((StatusCode::ACCEPTED).into_response());
}

// GET /course/.../modules/.../tasks/.../progress
pub async fn task_progress( 
    State(state): State<MySqlPool>,
    headers: HeaderMap,
    Path((course_id, module_id, task_id)): Path<(i32, i32, i32)>
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

    let user_id = match common::token::verify_jwt_token(token) {
        // TODO: Move to utilities (it repeats a lot)
        Ok(user_id) => user_id,
        Err(why) => {
            6 // test user id (exists in table)
            // Since it aint working rn we comment it
            // egprintln!("Why: {}", why);
            // let mut headers = HeaderMap::new();
            // headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            // return Err((StatusCode::UNAUTHORIZED, headers, serde_json::to_string_pretty(&handlers::ErrorResponse::new(
            //     &ErrorTypes::JwtTokenExpired.to_string(),
            //     "Token update requested",
            // ))
            // .unwrap()).into_response())
        }
    };
    let is_subscribe_to_course = false; // TODO: Validation (FORBIDDEN if doesnt own the course)

    match controllers::progress::get_task_progress(&state, user_id, task_id).await {
        Ok(progress) => {
            let body = json!({
                "data": progress
            }).to_string();

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            return Ok((StatusCode::OK, headers, body).into_response());
        },
        Err(why) => {
            eprintln!("Could not get progress (handler): {}", why);
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            return Err((
                StatusCode::BAD_REQUEST,
                headers,
                serde_json::to_string_pretty(&handlers::ErrorResponse::new(
                    &ErrorTypes::InternalError.to_string(),
                    "Could not fetch the task progress",
                ))
                .unwrap(), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    }
}
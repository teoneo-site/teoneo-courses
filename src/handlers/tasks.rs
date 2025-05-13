use std::i32;

use axum::{
    extract::{FromRequestParts, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    common::{self, token::Claims},
    controllers::{
        self,
        progress::ProgressStatus,
        task::{QuizUserAnswer, TaskType},
    },
    db,
    handlers::{self, ErrorTypes},
    AppState,
};

use super::ErrorResponse;


pub async fn get_tasks_for_module(
    State(state): State<AppState>,
    claims: Claims,
    Path((course_id, module_id)): Path<(i32, i32)>,
) -> Result<Response, Response> {
    match controllers::course::verify_ownership(&state.pool, claims.id as i32, course_id).await {
        Ok(val) if val == true => {}, // if does own nothing happens, just go on
        Err(_) | Ok(_) => { // Does not own the course
            // eprintln!("Could not verify course ownership {}", why);
            return Err((StatusCode::FORBIDDEN, axum::Json(ErrorResponse::new(ErrorTypes::CourseNotOwned, "User does not own this course"))).into_response())
        }
    }

    match controllers::task::get_tasks_for_module(&state.pool, module_id).await {
        Ok(tasks) => {
            let body = json!({
                "data": tasks,
            });

            return Ok((StatusCode::OK, axum::Json(body)).into_response());
        }
        Err(why) => {
            eprintln!("Why failed: {}", why);
            return Err((
                StatusCode::BAD_REQUEST,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch tasks",
                )), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };
}

pub async fn get_task(
    State(state): State<AppState>,
    claims: Claims,
    Path((course_id, module_id, task_id)): Path<(i32, i32, i32)>,
) -> Result<Response, Response> {
    match controllers::course::verify_ownership(&state.pool, claims.id as i32, course_id).await {
        Ok(val) if val == true => {}, // if does own nothing happens, just go on
        Err(_) | Ok(_) => { // Does not own the course
            // eprintln!("Could not verify course ownership {}", why);
            return Err((StatusCode::FORBIDDEN, axum::Json(ErrorResponse::new(ErrorTypes::CourseNotOwned, "User does not own this course"))).into_response())
        }
    }
    match controllers::task::get_task(&state.pool, module_id, task_id).await {
        Ok(task) => {
            let body = json!({
                "data": task,
            });
            return Ok((StatusCode::OK, axum::Json(body)).into_response());
        }
        Err(why) => {
            eprintln!("Why task fetch one: {}", why);
            return Err(ResponseBody::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                handlers::ErrorResponse::new(ErrorTypes::InternalError, "Could not fetch the task"), 
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
    State(state): State<AppState>,
    claims: Claims,
    Path((course_id, _module_id, task_id)): Path<(i32, i32, i32)>, // We dont really need module_id tho, just course (not necessary and)
    Json(user_answers): Json<serde_json::Value>,
) -> Result<Response, Response> {
    match controllers::course::verify_ownership(&state.pool, claims.id as i32, course_id).await {
        Ok(val) if val == true => {}, // if does own nothing happens, just go on
        Err(_) | Ok(_) => { // Does not own the course
            // eprintln!("Could not verify course ownership {}", why);
            return Err((StatusCode::FORBIDDEN, axum::Json(ErrorResponse::new(ErrorTypes::CourseNotOwned, "User does not own this course"))).into_response())
        }
    }

    let task_type = match db::taskdb::fetch_task_type(&state.pool, task_id).await {
        Ok(task_type) => task_type,
        Err(why) => {
            eprintln!("Why: {}", why);
            return Err((
                StatusCode::BAD_REQUEST,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch the task type. Task doesnt exist",
                )), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    };

    // Insert EVAL progress status
    // Frontend can query status at this point
    controllers::progress::update_or_insert_status(
        &state.pool,
        claims.id,
        task_id,
        ProgressStatus::Eval,
        "".to_string(),
        0.0,
        0,
    )
    .await
    .unwrap(); // Careful

    match task_type {
        TaskType::Quiz | TaskType::Match => {
            let answers_str = db::taskdb::fetch_task_answers(&state.pool, task_type, task_id)
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
                    &state.pool,
                    claims.id,
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
                    &state.pool,
                    claims.id,
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
        TaskType::Prompt => {
            let (attempts, max_attemps) =
                db::progressdb::get_prompt_task_attemps(&state.pool, claims.id, task_id) // Task is supposed to be prompt 100% at this point
                    .await
                    .unwrap(); // So unwrap() should not panic

            if attempts >= max_attemps {
                // Signal using 400 that max attempts is hit
                return Err((
                    StatusCode::BAD_REQUEST,
                    axum::Json(handlers::ErrorResponse::new(
                        ErrorTypes::MaxAttemptsSubmit,
                        "Try again later",
                    )), // Should not panic, because struct is always valid for converting into JSON
                )
                    .into_response());
            }

            tokio::spawn(async move {
                // Get attemps, max attemps and additional_field
                let pool = state.pool;
                let mut client = state.ai;

                let (question, add_prompt) = db::taskdb::fetch_prompt_details(&pool, task_id) // Again, task_id is 100% Prompt type
                    .await
                    .unwrap(); // This should not panic,only if Databse is broken, but then it will return 500 Server Internal Error on Panic
                let user_prompt = user_answers["data"]["user_prompt"]
                    .as_str()
                    .unwrap_or_default();

                let message = controllers::task::PROMPT_TEMPLATE
                    .replace("{question}", &question)
                    .replace("{user_prompt}", &user_prompt)
                    .replace(
                        "{additional_prompt}",
                        &add_prompt.unwrap_or("Нет доп. промпта".to_owned()),
                    );

                let reply = client.send_message(message.into()).await.unwrap(); // Should not panic under normal circumstances, only if gigachat is down, then it returns 500 Server internal error
                let reply_struct: controllers::task::PromptReply =
                    serde_json::from_str(&reply.content).unwrap(); // Panics on rate limit by gigachat, but 500 for this kind of situation is ok I guess?

                let mut json_submission: serde_json::Value =
                    serde_json::Value::Object(serde_json::Map::new());
                json_submission["reply"] = reply_struct.reply.into();
                json_submission["feedback"] = reply_struct.feedback.into();
                let score: f32 = reply_struct.score;

                controllers::progress::update_or_insert_status(
                    &pool,
                    claims.id,
                    task_id,
                    if score < 0.4 {
                        ProgressStatus::Failed
                    } else {
                        ProgressStatus::Success
                    },
                    json_submission.to_string(),
                    score,
                    0,
                )
                .await
                .unwrap(); // Should not panic, since at this point there is "eval" row that will get updated
            });
        }
        TaskType::Lecture => {}
    };

    return Ok((StatusCode::ACCEPTED).into_response());
}

// GET /course/.../modules/.../tasks/.../progress
pub async fn task_progress(
    State(state): State<AppState>,
    claims: Claims,
    Path((course_id, module_id, task_id)): Path<(i32, i32, i32)>,
) -> Result<Response, Response> {
    let user_id = claims.id;
    match controllers::course::verify_ownership(&state.pool, claims.id as i32, course_id).await {
        Ok(val) if val == true => {}, // if does own nothing happens, just go on
        Err(_) | Ok(_) => { // Does not own the course
            // eprintln!("Could not verify course ownership {}", why);
            return Err((StatusCode::FORBIDDEN, axum::Json(ErrorResponse::new(ErrorTypes::CourseNotOwned, "User does not own this course"))).into_response())
        }
    }
    match controllers::progress::get_task_progress(&state.pool, user_id, task_id).await {
        Ok(progress) => {
            let body = json!({
                "data": progress
            });
            return Ok((StatusCode::OK, axum::Json(body)).into_response());
        }
        Err(why) => {
            eprintln!("Could not get progress (handler): {}", why);
            return Err((
                StatusCode::BAD_REQUEST,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch the task progress",
                )), // Should not panic, because struct is always valid for converting into JSON
            )
                .into_response());
        }
    }
}

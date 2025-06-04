use std::i32;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::OptionalQuery;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    common::token::Claims,
    controllers::{
        self,
        progress::{self, ProgressStatus},
        task::{process_prompt_task, TaskType},
    },
    db,
    handlers::{self, ErrorTypes},
    AppState,
};

use super::ErrorResponse;

#[derive(Serialize, Deserialize)]
pub struct StatusQueryOptional {
    with_status: bool,
}

pub async fn get_tasks_for_module(
    State(state): State<AppState>,
    OptionalQuery(query_data): OptionalQuery<StatusQueryOptional>,
    claims: Claims,
    Path((course_id, module_id)): Path<(i32, i32)>,
) -> Result<Response, Response> {
    let user_id = claims.id as i32;
    if let Err(why) = controllers::course::verify_ownership(&state, claims.id as i32, course_id).await {
        eprintln!("Could not verify course ownership {}", why);
        return Err((
            StatusCode::FORBIDDEN,
            axum::Json(ErrorResponse::new(
                ErrorTypes::CourseNotOwned,
                "User does not own this course",
            )),
        )
            .into_response());
    }
    let should_display_status = query_data.map(|val| val.with_status).unwrap_or(false);
    match controllers::task::get_tasks_for_module(
        &state,
        module_id,
        if should_display_status {
            user_id.into()
        } else {
            None
        },
    )
    .await
    {
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

#[derive(Serialize, Deserialize)]
pub struct ProgressQueryOptional {
    with_progress: bool,
}

pub async fn get_task(
    State(state): State<AppState>,
    OptionalQuery(query_data): OptionalQuery<ProgressQueryOptional>,
    claims: Claims,
    Path((course_id, module_id, task_id)): Path<(i32, i32, i32)>,
) -> Result<Response, Response> {
    let user_id = claims.id as i32;
    if let Err(why) =  controllers::course::verify_ownership(&state, claims.id as i32, course_id).await {
        // Does not own the course
        eprintln!("Could not verify course ownership {}", why);
        return Err((
            StatusCode::FORBIDDEN,
            axum::Json(ErrorResponse::new(
                ErrorTypes::CourseNotOwned,
                "User does not own this course",
            )),
        )
            .into_response());
    }
    let should_display_progress = query_data.map(|val| val.with_progress).unwrap_or(false);
    match controllers::task::get_task(
        &state,
        module_id,
        task_id,
        if should_display_progress {
            user_id.into()
        } else {
            None
        },
    )
    .await
    {
        Ok(task) => {
            let body = json!({
                "data": task,
            });
            return Ok((StatusCode::OK, axum::Json(body)).into_response());
        }
        Err(why) => {
            eprintln!("Why task fetch one: {}", why);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(handlers::ErrorResponse::new(
                    ErrorTypes::InternalError,
                    "Could not fetch the task",
                )),
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
    let user_id = claims.id;
    if let Err(why) = controllers::course::verify_ownership(&state, user_id as i32, course_id).await {
        // Does not own the course
        eprintln!("Could not verify course ownership {}", why);
        return Err((
            StatusCode::FORBIDDEN,
            axum::Json(ErrorResponse::new(
                ErrorTypes::CourseNotOwned,
                "User does not own this course",
            )),
        )
            .into_response());
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
        &state,
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
            if let Err(why) = controllers::task::submit_quiz_task(
                &state,
                claims.id,
                task_id,
                task_type,
                user_answers,
            )
            .await
            {
                eprintln!("Could not submit quiz|match task: {}", why);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(handlers::ErrorResponse::new(
                        ErrorTypes::InternalError,
                        "Error when submiting a quiz/match task",
                    )),
                )
                    .into_response());
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

            if let Err(why) = process_prompt_task(state, claims.id, task_id, user_answers).await {
                eprintln!("Error submiting prompt task: {}", why);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(handlers::ErrorResponse::new(
                        ErrorTypes::InternalError,
                        "Error when submiting a prompt task",
                    )),
                )
                    .into_response());
            }
        }
        TaskType::Lecture => {
            progress::update_or_insert_status(
            &state,
            user_id,
            task_id,
            ProgressStatus::Success,
            "None".to_owned(),
            5.0,
            1,
            )
            .await.unwrap();
        }
    };

    return Ok((StatusCode::ACCEPTED).into_response());
}

// GET /course/.../modules/.../tasks/.../progress
pub async fn task_progress(
    State(state): State<AppState>,
    claims: Claims,
    Path((course_id, _module_id, task_id)): Path<(i32, i32, i32)>,
) -> Result<Response, Response> {
    let user_id = claims.id;
    if let Err(why) = controllers::course::verify_ownership(&state, claims.id as i32, course_id).await {
        // Does not own the course
        eprintln!("Could not verify course ownership {}", why);
        return Err((
            StatusCode::FORBIDDEN,
            axum::Json(ErrorResponse::new(
                ErrorTypes::CourseNotOwned,
                "User does not own this course",
            )),
        )
            .into_response());
    }
    match controllers::progress::get_task_progress(&state, user_id, task_id).await {
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

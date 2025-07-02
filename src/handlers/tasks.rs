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
    common::{
        error::{AppError, ErrorResponse, ErrorTypes},
        token::{AuthHeader},
    },
    controllers::{
        self,
        progress::{self, Progress, ProgressStatus},
        task::{process_prompt_task, Task, TaskShortInfo, TaskType},
    },
    db, error_response,
    AppState,
};


#[derive(Serialize, Deserialize)]
pub struct StatusQueryOptional {
    with_status: bool,
}

#[utoipa::path(
    get,
    description = "Возвращает задания модуля",
    path = "/courses/{course_id}/modules/{module_id}/tasks",
    params (
        ("Authorization" = String, Header, description = "JWT"),
        ("course_id" = i32, Path, description = "Айди курса"),
        ("module_id" = i32, Path, description = "Айди модуля"),
        ("with_status" = bool, Query, description = "(Optional) С true задание вернется со статусом прогресса")
    ),
    responses(
        (status = 200, description = "Успешно", body = TaskShortInfo),
        (status = 403, description = "Пользователь не владеет курсом", body = ErrorResponse),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_tasks_for_module(
    State(state): State<AppState>,
    OptionalQuery(query_data): OptionalQuery<StatusQueryOptional>,
    auth_header: AuthHeader,
    Path((course_id, module_id)): Path<(i32, i32)>,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id as i32;
    if let Err(why) =
        controllers::course::verify_ownership(&state, auth_header.claims.id as i32, course_id).await
    {
        tracing::error!("Could not verify course ownership {}", why);

        return Ok(error_response!(
            StatusCode::FORBIDDEN,
            ErrorTypes::CourseNotOwned,
            "User does not own this course"
        ));
    }
    let should_display_status = query_data.map(|val| val.with_status).unwrap_or(false);

    let tasks = controllers::task::get_tasks_for_module(
        &state,
        module_id,
        if should_display_status {
            user_id.into()
        } else {
            None
        },
    )
    .await?;
    let body = json!({
        "data": tasks,
    });

    return Ok((StatusCode::OK, axum::Json(body)).into_response());
}

#[derive(Serialize, Deserialize)]
pub struct ProgressQueryOptional {
    with_progress: bool,
}

#[utoipa::path(
    get,
    description = "Возвращает задание модуля",
    path = "/courses/{course_id}/modules/{module_id}/tasks/{task_id}",
    params (
        ("Authorization" = String, Header, description = "JWT"),
        ("course_id" = i32, Path, description = "Айди курса"),
        ("module_id" = i32, Path, description = "Айди модуля"),
        ("task_id" = i32, Path, description = "Айди задания"),
        ("with_progress" = bool, Query, description = "(Optional) С true задание вернется со статусом прогресса и score (0.0 - 1.0)")
    ),
    responses(
        (status = 200, description = "Успешно", body = Task),
        (status = 403, description = "Пользователь не владеет курсом", body = ErrorResponse),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn get_task(
    State(state): State<AppState>,
    OptionalQuery(query_data): OptionalQuery<ProgressQueryOptional>,
    auth_header: AuthHeader,
    Path((course_id, __, task_id)): Path<(i32, i32, i32)>,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id as i32;
    if let Err(why) =
        controllers::course::verify_ownership(&state, auth_header.claims.id as i32, course_id).await
    {
        // Does not own the course
        tracing::error!("Could not verify course ownership {}", why);
        return Ok(error_response!(
            StatusCode::FORBIDDEN,
            ErrorTypes::CourseNotOwned,
            "User does not own this course"
        ));
    }
    let should_display_progress = query_data.map(|val| val.with_progress).unwrap_or(false);

    let usr = if should_display_progress {
        user_id.into()
    } else {
        None
    };
    let task = controllers::task::get_task(&state, task_id, usr).await?;
    let body = json!({
        "data": task,
    });
    return Ok((StatusCode::OK, axum::Json(body)).into_response());
}

#[derive(Serialize, Deserialize)]
pub struct SubmitPayload {
    pub data: serde_json::Value, // Which can be either QuizUserAnswer or MatchingUserAnswer
}

// POST /course/.../modules/.../tasks/.../submit
#[utoipa::path(
    post,
    description = "Отправляет задание на `асинхронную` проверку. Фетчить результат проверки нужно `/progress`. Принимает JSON: data: { answers: [0, 0, 1] } для match и quiz. data: { user_prompt: 'string' } для prompt",
    path = "/courses/{course_id}/modules/{module_id}/tasks/{task_id}/submit",
    params (
        ("Authorization" = String, Header, description = "JWT"),
        ("course_id" = i32, Path, description = "Айди курса"),
        ("module_id" = i32, Path, description = "Айди модуля"),
        ("task_id" = i32, Path, description = "Айди задания"),
    ),
    responses(
        (status = 200, description = "Успешно"),
        (status = 403, description = "Пользователь не владеет курсом", body = ErrorResponse),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn submit_task(
    State(state): State<AppState>,
    auth_header: AuthHeader,
    Path((course_id, _module_id, task_id)): Path<(i32, i32, i32)>, // We dont really need module_id tho, just course (not necessary and)
    Json(user_answers): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;
    if let Err(why) = controllers::course::verify_ownership(&state, user_id as i32, course_id).await
    {
        // Does not own the course
        tracing::error!("Could not verify course ownership {}", why);
        return Ok(error_response!(
            StatusCode::FORBIDDEN,
            ErrorTypes::CourseNotOwned,
            "User does not own this course: {}",
            why
        ));
    }

    let task_type = db::taskdb::fetch_task_type(&state.pool, task_id).await?;

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
        TaskType::Quiz | TaskType::Match => {
            controllers::task::submit_quiz_task(
                &state,
                user_id,
                task_id,
                task_type,
                user_answers,
            )
            .await?;
            return Ok((StatusCode::ACCEPTED).into_response());
        }
        TaskType::Prompt => {
            let (attempts, max_attemps) =
                db::progressdb::get_prompt_task_attemps(&state.pool, user_id, task_id) // Task is supposed to be prompt 100% at this point
                    .await
                    .unwrap(); // So unwrap() should not panic
            if attempts >= max_attemps {
                // Signal using 400 that max attempts is hit
                return Ok(error_response!(
                    StatusCode::BAD_REQUEST,
                    ErrorTypes::MaxAttemptsSubmit,
                    "Try again later"
                ));
            }

            if let Err(why) = process_prompt_task(state, user_id, task_id, user_answers).await {
                tracing::error!("Error submiting prompt task: {}", why);

                return Ok(error_response!(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorTypes::InternalError,
                    "Error when submiting a prompt task: {}",
                    why
                ));
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
            .await
            .unwrap();
        }
    };

    return Ok((StatusCode::ACCEPTED).into_response());
}

// GET /course/.../modules/.../tasks/.../progress
#[utoipa::path(
    get,
    description = "Используется чтоб получить статус обработки задания",
    path = "/courses/{course_id}/modules/{module_id}/tasks/{task_id}/progress",
    params (
        ("Authorization" = String, Header, description = "JWT"),
        ("course_id" = i32, Path, description = "Айди курса"),
        ("module_id" = i32, Path, description = "Айди модуля"),
        ("task_id" = i32, Path, description = "Айди задания"),
    ),
    responses(
        (status = 200, description = "Успешно", body = Progress),
        (status = 403, description = "Пользователь не владеет курсом", body = ErrorResponse),
        (status = 500, description = "Что-то случилось", body = ErrorResponse)
    )
)]
pub async fn task_progress(
    State(state): State<AppState>,
    auth_header: AuthHeader,
    Path((course_id, _module_id, task_id)): Path<(i32, i32, i32)>,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;
    if let Err(why) =
        controllers::course::verify_ownership(&state, user_id as i32, course_id).await
    {
        // Does not own the course
        tracing::error!("Could not verify course ownership {}", why);
        return Ok(error_response!(
            StatusCode::FORBIDDEN,
            ErrorTypes::CourseNotOwned,
            "User does not own this course"
        ));
    }

    let progress = controllers::progress::get_task_progress(&state, user_id, task_id).await?;
    let body = json!({
        "data": progress
    });

    Ok((StatusCode::OK, axum::Json(body)).into_response())
}

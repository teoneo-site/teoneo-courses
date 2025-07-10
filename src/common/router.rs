use std::time::Duration;

use axum::{error_handling::HandleErrorLayer, http::StatusCode, BoxError, Router};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::{catch_panic::CatchPanicLayer, cors::CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{common::error::{ErrorResponse, ErrorTypes}, handlers, internal_server_error_handler, swagger, AppState};

fn courses_router() -> Router<AppState> {
    Router::new()
    .route(
        "/courses",
        axum::routing::get(handlers::courses::get_all_courses),
    )
    .route(
        "/courses/ids",
        axum::routing::get(handlers::courses::get_courses_by_ids),
    )
    .route(
        "/courses/{course_id}",
        axum::routing::get(handlers::courses::get_course),
    )
    .route(
        "/courses/{course_id}/progress",
        axum::routing::get(handlers::courses::get_course_progress),
    )
    .route(
        "/internal/courses/users/{user_id}",
        axum::routing::get(handlers::courses::get_user_courses)
    )
    .route(
        "/internal/courses/users/{user_id}/started",
        axum::routing::get(handlers::courses::get_user_courses_started)
    )
    .route(
        "/internal/courses/users/{user_id}/completed",
        axum::routing::get(handlers::courses::get_user_courses_completed)
    )
    .route(
        "/courses/{course_id}/favour",
        axum::routing::post(handlers::courses::add_course_to_favourite),
    )
    .route(
        "/courses/favourite",
        axum::routing::get(handlers::courses::get_favourite_courses),
    )
}

fn modules_router() -> Router<AppState> {
    Router::new()
    .route(
        "/courses/{course_id}/modules",
        axum::routing::get(handlers::modules::get_modules_for_course),
    )
    .route(
        "/courses/{course_id}/modules/{module_id}",
        axum::routing::get(handlers::modules::get_module),
    )
}

fn task_router() -> Router<AppState> {
    Router::new()
    .route(
        "/courses/{course_id}/modules/{module_id}/tasks",
        axum::routing::get(handlers::tasks::get_tasks_for_module),
    )
    .route(
        "/courses/{course_id}/modules/{module_id}/tasks/{task_id}",
        axum::routing::get(handlers::tasks::get_task),
    )
    .route(
        "/courses/{course_id}/modules/{module_id}/tasks/{task_id}/submit",
        axum::routing::post(handlers::tasks::submit_task),
    )
    .route(
        "/courses/{course_id}/modules/{module_id}/tasks/{task_id}/progress",
        axum::routing::get(handlers::tasks::task_progress),
    )
}

pub fn get_router(app_state: AppState) -> Router {
    let app = Router::new()
        .merge(courses_router())
        .merge(modules_router())
        .merge(task_router())
        .layer(CorsLayer::permissive().allow_origin(tower_http::cors::Any))
        .layer(CatchPanicLayer::custom(internal_server_error_handler))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    // So compiler wont complain about some Infallable Trait shit
                    tracing::error!("{}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Internal error occured",
                        )),
                    )
                }))
                .layer(BufferLayer::new(1024)) // Means it can process 1024 messages before backpressure is applied TODO: Adjust
                .layer(RateLimitLayer::new(10, Duration::from_secs(1))), // Rate limti does not impl Clone, so we need to use BufferLayer TODO: Adjust
        ) // Makes layers run in the background and messages are sent through the channels to them
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", swagger::ApiDoc::openapi()))
        .with_state(app_state);
    app
}
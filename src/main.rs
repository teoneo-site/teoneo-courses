use std::{any::Any, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::{IntoResponse, Response},
    BoxError, Router,
};
use gigalib::{
    controllers::client::{ClientBuilder, GigaClient},
    http::message::{MessageConfig, MessageConfigBuilder},
};
use handlers::ErrorTypes;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::{catch_panic::CatchPanicLayer, cors::CorsLayer};

mod common;
mod controllers;
mod db;
mod handlers;

#[derive(Clone)]
struct AppState {
    pool: Pool<MySql>,
    ai: GigaClient,
}

fn internal_server_error_handler(err: Box<dyn Any + Send + 'static>) -> Response {
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic message".to_string()
    };
    println!("Internal server error catched: {}", details);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(handlers::ErrorResponse::new(
            ErrorTypes::InternalError,
            &details,
        )), // Should not panic, because struct is always valid for converting into JSON
    )
        .into_response()
}


async fn get_db_connection() -> anyhow::Result<Pool<MySql>> {
    let connect_str = "mysql://root:root@localhost:3306/teoneo";
    let mysql_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(connect_str)
        .await?;
    Ok(mysql_pool)
}

async fn get_gigachat_client() -> anyhow::Result<GigaClient> {
    let config: MessageConfig = MessageConfigBuilder::new().set_model("GigaChat").build();
    let client: GigaClient = ClientBuilder::new()
        .set_basic_token(&std::env::var("GIGACHAT_TOKEN").unwrap())
        .set_msg_cfg(config)
        .build();
    Ok(client)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let app_state = AppState {
        pool: get_db_connection().await.unwrap(),
        ai: get_gigachat_client().await.unwrap(),
    };

    #[rustfmt::skip]
    let app = Router::new()
        .route("/courses", axum::routing::get(handlers::courses::get_all_courses))
        .route("/courses/{course_id}", axum::routing::get(handlers::courses::get_course))
        .route("/courses/{course_id}/modules", axum::routing::get(handlers::modules::get_modules_for_course))
        .route("/courses/{course_id}/modules/{module_id}", axum::routing::get(handlers::modules::get_module))
        .route("/courses/{course_id}/modules/{module_id}/tasks", axum::routing::get(handlers::tasks::get_tasks_for_module))
        .route("/courses/{course_id}/modules/{module_id}/tasks/{task_id}", axum::routing::get(handlers::tasks::get_task))
        .route("/courses/{course_id}/modules/{module_id}/tasks/{task_id}/submit", axum::routing::post(handlers::tasks::submit_task))
        .route("/courses/{course_id}/modules/{module_id}/tasks/{task_id}/progress", axum::routing::get(handlers::tasks::task_progress))
        .layer(CorsLayer::permissive().allow_origin(tower_http::cors::Any))
        .layer(CatchPanicLayer::custom(internal_server_error_handler))
        .layer(
            ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|err: BoxError| async move { // So compiler wont complain about some Infallable Trait shit
                eprintln!("{}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(handlers::ErrorResponse::new(
                        ErrorTypes::InternalError,
                        "Internal error occured",
                    ))
                )
            }))
            .layer(BufferLayer::new(1024)) // Means it can process 1024 messages before backpressure is applied TODO: Adjust
            .layer(RateLimitLayer::new(10, Duration::from_secs(1))), // Rate limti does not impl Clone, so we need to use BufferLayer TODO: Adjust
            ) // Makes layers run in the background and messages are sent through the channels to them
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use std::{any::Any, time::Duration};

use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Router,
};
use gigalib::{
    controllers::client::{ClientBuilder, GigaClient},
    http::message::{MessageConfig, MessageConfigBuilder},
};
use handlers::ErrorTypes;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use tower_http::{
    catch_panic::{CatchPanic, CatchPanicLayer},
    cors::CorsLayer,
};

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

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        headers,
        serde_json::to_string_pretty(&handlers::ErrorResponse::new(
            ErrorTypes::InternalError,
            &details,
        ))
        .unwrap(), // Should not panic, because struct is always valid for converting into JSON
    )
        .into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let connect_str = "mysql://root:root@localhost:3306/teoneo";
    let mysql_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(connect_str)
        .await
        .expect("Could not connect to the database");

    let config: MessageConfig = MessageConfigBuilder::new().set_model("GigaChat").build();
    let client: GigaClient = ClientBuilder::new()
        .set_basic_token(&std::env::var("GIGACHAT_TOKEN").unwrap())
        .set_msg_cfg(config)
        .build();

    let app_state = AppState {
        pool: mysql_pool,
        ai: client,
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
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

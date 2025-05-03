use std::time::Duration;

use axum::Router;
use sqlx::mysql::MySqlPoolOptions;
use tower_http::cors::CorsLayer;

mod common;
mod controllers;
mod db;
mod handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let connect_str = "mysql://klewy:root@localhost:3306/teoneo";
    let mysql_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(connect_str)
        .await
        .expect("Could not connect to the database");

    #[rustfmt::skip]
    let app = Router::new()
        .route("/courses", axum::routing::get(handlers::courses::get_all_courses))
        .route("/courses/{course_id}", axum::routing::get(handlers::courses::get_course))
        .route("/courses/{course_id}/modules", axum::routing::get(handlers::modules::get_modules_for_course))
        .route("/courses/{course_id}/modules/{module_id}", axum::routing::get(handlers::modules::get_module))
        .route("/courses/{course_id}/modules/{module_id}/tasks", axum::routing::get(handlers::tasks::get_tasks_for_module))
        .route("/courses/{course_id}/modules/{module_id}/tasks/{task_id}", axum::routing::get(handlers::tasks::get_task))
        .layer(CorsLayer::permissive().allow_origin(tower_http::cors::Any))
        .with_state(mysql_pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

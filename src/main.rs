use std::time::Duration;

use axum::Router;
use gigalib::{
    controllers::client::{ClientBuilder, GigaClient},
    http::message::{MessageConfig, MessageConfigBuilder},
};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use tower_http::cors::CorsLayer;

mod common;
mod controllers;
mod db;
mod handlers;

#[derive(Clone)]
struct AppState {
    pool: Pool<MySql>,
    ai: GigaClient,
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
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

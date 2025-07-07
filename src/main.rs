use std::{any::Any, env, time::Duration};

use axum::{
    extract::FromRef, http::StatusCode, response::{IntoResponse, Response}
};
use gigalib::{
    controllers::client::{ClientBuilder, GigaClient},
    http::message::{MessageConfig, MessageConfigBuilder},
};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::common::error::{ErrorResponse, ErrorTypes};

mod common;
mod controllers;
mod db;
mod handlers;
mod swagger;
mod clients;

#[derive(Clone)]
struct BasicState {
    pool: Pool<MySql>,
    redis: r2d2::Pool<redis::Client>,
    
}

#[derive(Clone)]
struct AppState {
    basic: BasicState,
    ai: GigaClient,
}

impl FromRef<AppState> for GigaClient {
    fn from_ref(state: &AppState) -> Self {
        state.ai.clone()
    }
}

impl FromRef<AppState> for BasicState {
    fn from_ref(input: &AppState) -> Self {
        input.basic.clone()
    }
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
        axum::Json(ErrorResponse::new(
            ErrorTypes::InternalError,
            &details,
        )), // Should not panic, because struct is always valid for converting into JSON
    )
        .into_response()
}

async fn get_db_pool() -> anyhow::Result<Pool<MySql>> {
    let connect_str = env::var("DATABASE_URL").unwrap();
    let mysql_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&connect_str)
        .await?;
    Ok(mysql_pool)
}

async fn get_redis_pool() -> anyhow::Result<r2d2::Pool<redis::Client>> {
    let client = redis::Client::open(env::var("REDIS_URL").unwrap())?;
    let pool = r2d2::Pool::builder().build(client).unwrap();
    Ok(pool)
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
    tracing_subscriber::fmt().pretty().init();
    
    dotenv::dotenv().ok();

    let app_state = AppState {
        basic: BasicState { 
            pool: get_db_pool()
            .await
            .expect("Could not connect to the database"),
            redis: get_redis_pool().await.expect("Could not connect to redis"),
        },
        ai: get_gigachat_client()
            .await
            .expect("Could not connect to gigachat")
        
    };
    let router = common::router::get_router(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Started on port 8080");
    axum::serve(listener, router).await.unwrap();
}

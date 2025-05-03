use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::db;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    #[serde(alias = "quiz")]
    Quiz,
    #[serde(alias = "lecture")]
    Lecture,
    #[serde(alias = "prompt")]
    Prompt,
}

impl Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quiz => write!(f, "quiz"),
            Self::Lecture => write!(f, "lecture"),
            Self::Prompt => write!(f, "prompt"),
        }
    }
}

impl From<String> for TaskType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "quiz" => Self::Quiz,
            "lecture" => Self::Lecture,
            "prompt" => Self::Prompt,
            _ => panic!("Unknown task type"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TaskShortInfo {
    id: i32,
    module_id: i32,
    title: String,
    #[serde(alias = "type")]
    task_type: TaskType,
}

impl TaskShortInfo {
    pub fn new(id: i32, module_id: i32, title: String, task_type: TaskType) -> Self {
        Self {
            id,
            module_id,
            title,
            task_type,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub module_id: i32,
    pub title: String,
    pub task_type: TaskType,
    pub content: serde_json::Value, // содержимое задания
}

impl Task {
    pub fn new(
        id: i32,
        module_id: i32,
        title: String,
        task_type: TaskType,
        content: serde_json::Value,
    ) -> Self {
        Self {
            id,
            module_id,
            title,
            task_type,
            content,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct QuizTask {
    pub question: String,
    pub possible_answers: Vec<String>, // divided by ';'
    pub is_multiple: bool,
    pub answers: Vec<u8>, // string div by ';'
    pub picture_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct LectureTask {
    pub text: String, // Теоритический материал для лекции
    pub picture_url: String,
    pub video_url: String,
}

pub async fn get_tasks_for_module(
    pool: &MySqlPool,
    module_id: i32,
) -> anyhow::Result<Vec<TaskShortInfo>> {
    let tasks = db::taskdb::fetch_tasks_for_module(pool, module_id).await?;
    Ok(tasks)
}

pub async fn get_task(pool: &MySqlPool, module_id: i32, task_id: i32) -> anyhow::Result<Task> {
    let task = db::taskdb::fetch_task(pool, module_id, task_id).await?;
    Ok(task)
}

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::db;

use super::progress::ProgressStatus;

pub const PROMPT_TEMPLATE: &'static str = r#"
Ты выступаешь как система оценки качества промптов для ИИ. Пользователь должен был написать промпт, соответствующий заданной задаче. Вот описание задачи:

{question}

Вот промпт, написанный пользователем:
{user_prompt}

(Дополнительный контекст, который следует учитывать при оценке промпта:
{additional_prompt})

Оцени этот промпт по следующим критериям:
1. Насколько чётко и конкретно сформулирована задача.
2. Соответствует ли промпт цели задания.
3. Содержит ли промпт необходимую структуру, ключевые слова или примеры.
4. Есть ли грамматические или логические ошибки.
5. Насколько он эффективен с точки зрения получения правильного ответа от ИИ.

Верни ответ в формате JSON без лишних символов, т.е без `:
{{
  "score": <число от 0.0 до 1.0, где 0 - 0%, 1.0 - 100%>,
  "reply": <ответ на промпт пользователя, ты должен выполнить его задание, он не сильно распространенный, довольно краткий, но не сильно>,
  "feedback": "<краткий текстовый отзыв>"
}}
"#;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    #[serde(alias = "quiz")]
    Quiz,
    #[serde(alias = "lecture")]
    Lecture,
    #[serde(alias = "prompt")]
    Prompt,
    #[serde(alias = "match")]
    Match,
}

impl Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quiz => write!(f, "quiz"),
            Self::Lecture => write!(f, "lecture"),
            Self::Prompt => write!(f, "prompt"),
            Self::Match => write!(f, "match"),
        }
    }
}

impl From<String> for TaskType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "quiz" => Self::Quiz,
            "lecture" => Self::Lecture,
            "prompt" => Self::Prompt,
            "match" => Self::Match,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<ProgressStatus>
}

impl TaskShortInfo {
    pub fn new(id: i32, module_id: i32, title: String, task_type: TaskType, status: Option<ProgressStatus>) -> Self {
        Self {
            id,
            module_id,
            title,
            task_type,
            status
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ProgressStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

impl Task {
    pub fn new(
        id: i32,
        module_id: i32,
        title: String,
        task_type: TaskType,
        content: serde_json::Value,
        status: Option<ProgressStatus>,
        score: Option<f32>
    ) -> Self {
        Self {
            id,
            module_id,
            title,
            task_type,
            content,
            status,
            score
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
pub struct QuizUserAnswer {
    pub answers: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct LectureTask {
    pub text: String, // Теоритический материал для лекции
    pub picture_url: String,
    pub video_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct PromptReply {
    pub score: f32,
    pub reply: String,
    pub feedback: String,
}

pub async fn get_tasks_for_module(
    pool: &MySqlPool,
    module_id: i32,
    user_id: Option<i32>
) -> anyhow::Result<Vec<TaskShortInfo>> {
    let tasks = db::taskdb::fetch_tasks_for_module(pool, module_id, user_id).await?;
    Ok(tasks)
}

pub async fn get_task(pool: &MySqlPool, module_id: i32, task_id: i32, user_id: Option<i32>) -> anyhow::Result<Task> {
    let task: Task = db::taskdb::fetch_task(pool, module_id, task_id, user_id).await?;
    Ok(task)
}

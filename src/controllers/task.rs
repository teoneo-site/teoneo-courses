use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{db, AppState};

use super::progress::{self, ProgressStatus};

pub const PROMPT_TEMPLATE: &'static str = r#"
Ты выступаешь как система оценки качества промптов для ИИ. Пользователь должен был написать промпт, соответствующий заданной задаче. Вот описание задачи:

{question}

Вот промпт, написанный пользователем (ТЫ ДОЛЖЕН ОЦЕНИВАТЬ ЕГО):
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
{
  "score": <число от 0.0 до 1.0, где 0 - 0%, 1.0 - 100%>,
  "reply": <ответ на промпт пользователя, ты должен выполнить его задание, он не сильно распространенный, довольно краткий, но не сильно>,
  "feedback": "<краткий текстовый отзыв>"
}
"#;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum TaskType {
    #[serde(rename = "QUIZ")]
    Quiz,
    #[serde(rename = "LECTURE")]
    Lecture,
    #[serde(rename = "PROMPT")]
    Prompt,
    #[serde(rename = "MATCH")]
    Match,
}

impl Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quiz => write!(f, "QUIZ"),
            Self::Lecture => write!(f, "LECTURE"),
            Self::Prompt => write!(f, "PROMPT"),
            Self::Match => write!(f, "MATCH"),
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
    pub id: i32,
    pub module_id: i32,
    pub title: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ProgressStatus>,
}

impl TaskShortInfo {
    pub fn new(
        id: i32,
        module_id: i32,
        title: String,
        task_type: TaskType,
        status: Option<ProgressStatus>,
    ) -> Self {
        Self {
            id,
            module_id,
            title,
            task_type,
            status,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub module_id: i32,
    pub title: String,
    #[serde(rename = "type")]
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
        score: Option<f32>,
    ) -> Self {
        Self {
            id,
            module_id,
            title,
            task_type,
            content,
            status,
            score,
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
    state: &AppState,
    module_id: i32,
    user_id: Option<i32>,
) -> anyhow::Result<Vec<TaskShortInfo>> {
    let tasks = db::taskdb::fetch_tasks_for_module(state, module_id, user_id).await?;
    Ok(tasks)
}

pub async fn get_task(
    state: &AppState,
    module_id: i32,
    task_id: i32,
    user_id: Option<i32>,
) -> anyhow::Result<Task> {
    let task: Task = db::taskdb::fetch_task(state, module_id, task_id, user_id).await?;
    Ok(task)
}

pub async fn submit_quiz_task(
    state: &AppState,
    user_id: u32,
    task_id: i32,
    task_type: TaskType,
    user_answers: serde_json::Value,
) -> anyhow::Result<()> {
    let answers_str = db::taskdb::fetch_task_answers(&state.pool, task_type, task_id).await?;
    let task_answers: Vec<u8> = answers_str
        .split(";")
        .map(|element| element.parse::<u8>().unwrap_or(0))
        .collect();
    let user_answers: QuizUserAnswer = serde_json::from_value(user_answers["data"].clone())?; // TODO: Handle

    if task_answers.len() != user_answers.answers.len()
        || task_answers
            .iter()
            .zip(&user_answers.answers)
            .filter(|&(a, b)| a == b)
            .count()
            != task_answers.len()
    {
        progress::update_or_insert_status(
            state,
            user_id,
            task_id,
            ProgressStatus::Failed,
            serde_json::to_string(&user_answers).unwrap(),
            0.0,
            1,
        )
        .await?;
    } else {
        // Set status to SUCCESSS, submission to user_answers, score to 1.0, attempts to 1 if exists + 1
        progress::update_or_insert_status(
            state,
            user_id,
            task_id,
            ProgressStatus::Success,
            serde_json::to_string(&user_answers).unwrap(),
            1.0,
            1,
        )
        .await?;
    }
    Ok(())
}

pub async fn process_prompt_task(
    state: AppState,
    user_id: u32,
    task_id: i32,
    user_answers: serde_json::Value,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        // Get attemps, max attemps and additional_field
        let mut state = state;

        let (question, add_prompt) = db::taskdb::fetch_prompt_details(&state.pool, task_id) // Again, task_id is 100% Prompt type
            .await
            .unwrap(); // This should not panic,only if Databse is broken, but then it will return 500 Server Internal Error on Panic
        let user_prompt = user_answers["data"]["user_prompt"]
            .as_str()
            .unwrap_or_default();

        let message = PROMPT_TEMPLATE
            .replace("{question}", &question)
            .replace("{user_prompt}", &user_prompt)
            .replace(
                "{additional_prompt}",
                &add_prompt.unwrap_or("Нет доп. промпта".to_owned()),
            );

        let reply = state.ai.send_message(message.into()).await.unwrap(); // Should not panic under normal circumstances, only if gigachat is down, then it returns 500 Server internal error
        println!("GIGACHAT REPLY: {}", reply.content);

        let reply_struct: PromptReply = match serde_json::from_str(&reply.content) {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!("Could not parse gigachat reply: {}", err);
                return;
            }
        };

        let mut json_submission: serde_json::Value =
            serde_json::Value::Object(serde_json::Map::new());
        json_submission["reply"] = reply_struct.reply.into();
        json_submission["feedback"] = reply_struct.feedback.into();
        let score: f32 = reply_struct.score;

        progress::update_or_insert_status(
            &state,
            user_id,
            task_id,
            if score < 0.4 {
                ProgressStatus::Failed
            } else {
                ProgressStatus::Success
            },
            json_submission.to_string(),
            score,
            0,
        )
        .await
        .unwrap(); // Should not panic, since at this point there is "eval" row that will get updated
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tasktype_into_str() {
        let task_type = TaskType::Lecture;
        assert_eq!(task_type.to_string(), "LECTURE");

        let task_type = TaskType::Match;
        assert_eq!(task_type.to_string(), "MATCH");

        let task_type = TaskType::Prompt;
        assert_eq!(task_type.to_string(), "PROMPT");

        let task_type = TaskType::Quiz;
        assert_eq!(task_type.to_string(), "QUIZ");
    }

    #[test]
    fn test_str_into_tasktype() {
        let str = String::from("LECTURE");
        let task_type: TaskType = str.into();
        assert_eq!(task_type, TaskType::Lecture);

        let str = String::from("match");
        let task_type: TaskType = str.into();
        assert_eq!(task_type, TaskType::Match);

        let str = String::from("PROMPT");
        let task_type: TaskType = str.into();
        assert_eq!(task_type, TaskType::Prompt);

        let str = String::from("quiz");
        let task_type: TaskType = str.into();
        assert_eq!(task_type, TaskType::Quiz);
    }
}

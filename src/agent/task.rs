use crate::prompts::AGENT_TASK_PROMPT;
use project_oculus::utils::generate_ai_response;
use serde::{Deserialize, Serialize}; // Added Deserialize for completeness, though not strictly used for adding
use std::error::Error;
use std::sync::{Mutex, OnceLock};
// Function to handle the task agent logic
pub async fn task_agent(_last_step: String) -> Result<String, Box<dyn Error>> {
    let task = format!("{}{}", AGENT_TASK_PROMPT, _last_step);
    let response = generate_ai_response(&task, "").await;
    let mut string_response: String = String::new();

    match response {
        Ok(ai_response) => {
            println!("AI Response: {}", ai_response);
            string_response = ai_response;
        }
        Err(e) => {
            eprintln!("Error generating AI response: {}", e);
        }
    }

    // Parse the AI response to remove ````json` and `\n`
    string_response = string_response
        .replace("```json", "")
        .replace("```", "")
        .replace("\n", "");
    println!("Parsed AI Response: {}", string_response);

    Ok(string_response)
}

static TASK_HISTORY: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    id: u32,
    description: String,
    status: String,
    // You can add more fields like timestamp, user, etc.
}

pub async fn get_task_history() -> Result<Vec<String>, Box<dyn Error>> {
    if let Some(history) = TASK_HISTORY.get() {
        let tasks = history.lock().unwrap();
        Ok(tasks.clone())
    } else {
        let initial_history = Mutex::new(Vec::new());
        TASK_HISTORY.set(initial_history).unwrap();
        Ok(Vec::new())
    }
}

pub async fn add_task_to_history(task: String) -> Result<(), Box<dyn Error>> {
    if let Some(history) = TASK_HISTORY.get() {
        let mut tasks = history.lock().unwrap();
        tasks.push(task);
        Ok(())
    } else {
        Err("Task history not initialized".into())
    }
}

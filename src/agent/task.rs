use project_oculus::utils::generate_ai_response;

use crate::prompts::AGENT_TASK_PROMPT;
use std::error::Error;
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

pub async fn task_history() -> Result<String, Box<dyn Error>> {
    // This function can be used to retrieve the task history
    // For now, it just returns a placeholder string
    Ok("Task history is not implemented yet.".to_string())
}

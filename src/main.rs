use project_oculus::get_user_input;
mod agent;
use project_oculus::utils::generate_ai_response;
mod prompts;

use crate::prompts::PLANNER_PROMPT;
fn main() {
    let user_task = get_user_input("Enter the task you want to perform: ");
    println!("You entered: {}", user_task);

    let response = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(generate_ai_response(&user_task, PLANNER_PROMPT));
    match response {
        Ok(ai_response) => {
            println!("AI Response: {}", ai_response);
        }
        Err(e) => {
            eprintln!("Error generating AI response: {}", e);
        }
    }
    println!("Task completed successfully.");
}

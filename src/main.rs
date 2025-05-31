mod agent;
mod prompts;
mod utils;
use crate::agent::executor::execute_task;
use crate::agent::orchestrator::Orchestrator;
use crate::agent::planner::planner_agent;
use crate::agent::task::task_agent;
use crate::prompts::PLANNER_PROMPT;
use project_oculus::browser_control::actions::{extract_content, go_to_url};
use project_oculus::browser_control::browser_use::create_new_browser;
use project_oculus::browser_control::interactive_elements::{
    self, get_interactive_elements_in_hashmap,
};
use serde_json::{self, Value}; // Import Value
use std::io::{self};

fn main() {
    let driver = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(create_new_browser())
        .expect("Failed to create new browser");

    let mut orchestrator = Orchestrator::new();
    let final_answer = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(async { orchestrator.run(&driver).await })
        .expect("Orchestrator run failed");

    if let Some(answer) = final_answer {
        println!("Final Answer: {}", answer);
    } else {
        println!("No final answer found.");
    }
    // Close the browser
    tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(driver.quit())
        .expect("Failed to quit browser");
    println!("Task completed successfully.");
}

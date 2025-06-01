use crate::agent::executor::execute_task;
use crate::agent::planner::planner_agent;
use crate::agent::task::task_agent;
use project_oculus::browser_control::actions::{extract_content, go_to_url};
use project_oculus::browser_control::interactive_elements::get_interactive_elements_in_hashmap;
use serde_json::Value;
use thirtyfour::prelude::*;

/// The Orchestrator agent coordinates planning, tasking, and execution.
pub struct Orchestrator {
    pub task_history: Vec<String>,
    pub final_answer: Option<String>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            task_history: Vec::new(),
            final_answer: None,
        }
    }

    /// Run the orchestration loop. Returns the final answer if found.
    pub async fn run(&mut self, driver: &WebDriver) -> WebDriverResult<Option<String>> {
        // Initial planning
        let plan = planner_agent().await.ok();
        let mut string_response = plan.unwrap_or_default();
        string_response = string_response
            .replace("```json", "")
            .replace("```", "")
            .replace("\n", "");
        self.task_history
            .push(format!("Initial plan: {}", string_response));

        // Initial execution (if any)
        let json_res: Result<Value, _> = serde_json::from_str(&string_response);
        if let Ok(json_value) = json_res {
            if json_value["action"][0]["go_to_url"].is_object() {
                if let Some(url) = json_value["action"][0]["go_to_url"]["url"].as_str() {
                    go_to_url(driver, url).await.ok();
                }
            } else if json_value["action"][0]["extract_content"].is_object() {
                extract_content(driver).await.ok();
            }
        }

        // Main orchestration loop
        loop {
            let interactive_elements = get_interactive_elements_in_hashmap(driver).await;
            let last_step_string = match interactive_elements {
                Ok(elements) => elements
                    .iter()
                    .map(|(k, v)| format!("{}: {:?}", k, v))
                    .collect::<Vec<_>>()
                    .join(", "),
                Err(_) => String::from("No interactive elements found."),
            };
            let history_str = if self.task_history.is_empty() {
                String::from("")
            } else {
                format!("\n\nTask History:\n{}\n", self.task_history.join("\n"))
            };
            let prompt_with_history = format!("{}{}", history_str, last_step_string);
            // Get next task from task agent
            let ai_response = task_agent(prompt_with_history).await.unwrap_or_default();
            self.task_history.push(ai_response.clone());
            // Execute the task
            let result = execute_task(&ai_response, driver).await.unwrap_or_default();
            self.task_history
                .push(format!("Execution result: {}", result));
            if result.starts_with("FINAL_ANSWER:") {
                self.final_answer = Some(result.replace("FINAL_ANSWER: ", ""));
                break;
            } else if result == "TASK_COMPLETE" {
                break;
            } else if result == "ERROR_PARSING_JSON" {
                continue;
            }
        }
        Ok(self.final_answer.clone())
    }
}

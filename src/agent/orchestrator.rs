use crate::agent::executor::execute_task;
use crate::agent::planner::planner_agent;
use crate::agent::task::task_agent;
use crate::agent::task::{add_task_to_history, get_task_history};
use project_oculus::browser_control::browser_use::create_new_browser;
use project_oculus::browser_control::interactive_elements::get_interactive_elements_in_hashmap;
use serde_json::Value;
use std::error::Error;

pub async fn orchestrator_agent(_planner_response: String) -> Result<(), Box<dyn Error>> {
    let mut string_response: String = String::new();

    println!("Orchestrator starting...");

    let driver = match create_new_browser().await {
        Ok(d) => {
            println!("Browser initialized successfully.");
            d
        }
        Err(e) => {
            eprintln!("Failed to create new browser: {}", e);
            return Err(e.into());
        }
    };

    let mut final_answer: String = String::new();
    let max_steps = 10;
    let mut current_step = 0;

    while current_step < max_steps {
        current_step += 1;
        println!("\n--- Orchestrator Step {} ---", current_step);

        let interactive_elements = match get_interactive_elements_in_hashmap(&driver).await {
            Ok(elements) => elements,
            Err(e) => {
                eprintln!(
                    "Failed to get interactive elements: {}. Attempting to continue.",
                    e
                );
                Default::default()
            }
        };
        let elements_string = interactive_elements
            .iter()
            .map(|(k, v)| format!("{}: {:?}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let task_history = get_task_history().await.unwrap_or_default();
        let history_str = if task_history.is_empty() {
            String::from("No previous actions taken.")
        } else {
            format!(
                "Task History (most recent last):\n{}",
                task_history
                    .iter()
                    .rev()
                    .take(5)
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        // Fix: Await current_url and handle errors
        let current_url = match driver.current_url().await {
            Ok(url) => url.to_string(),
            Err(e) => {
                eprintln!("Failed to get current URL: {}", e);
                String::from("Unknown (error getting URL)")
            }
        };

        let task_agent_input = gen_prompt(_planner_response.clone(), current_url, history_str);
        println!(
            "Input for Task Agent:\n{}\n------------------------",
            task_agent_input
        );

        let task_agent_result = task_agent(task_agent_input.clone()).await;
        match task_agent_result {
            Ok(ai_response) => {
                println!("Task Agent AI Response: {}", ai_response);
                string_response = ai_response.clone();
                if let Err(e) = add_task_to_history(format!(
                    "Step {}: Task Agent decided: {}",
                    current_step, ai_response
                ))
                .await
                {
                    eprintln!("Failed to add task to history: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error generating task agent AI response: {}", e);
                if let Err(err) = add_task_to_history(format!(
                    "Step {}: Error from Task Agent: {}",
                    current_step, e
                ))
                .await
                {
                    eprintln!("Failed to add task to history: {}", err);
                }
                println!("Breaking loop due to task_agent error.");
                final_answer = format!("Error in task agent: {}", e);
                break;
            }
        }

        println!(
            "Step {}: Using planner's response for execution: {}",
            current_step, string_response
        );
        if let Err(e) = add_task_to_history(format!(
            "Step {}: Using planner's initial action: {}",
            current_step, string_response
        ))
        .await
        {
            eprintln!("Failed to add task to history: {}", e);
        }

        match execute_task(string_response.clone(), &driver).await {
            Ok(result) => {
                println!("Execution Result: {}", result);
                if let Err(e) = add_task_to_history(format!(
                    "Step {}: Execution result: {}",
                    current_step, result
                ))
                .await
                {
                    eprintln!("Failed to add task to history: {}", e);
                }

                if result.starts_with("FINAL_ANSWER:") {
                    final_answer = result.replace("FINAL_ANSWER: ", "").trim().to_string();
                    println!("Final answer received: {}", final_answer);
                    break;
                } else if result.to_uppercase() == "TASK_COMPLETE"
                    || result.to_uppercase() == "\"TASK_COMPLETE\""
                {
                    // Fixed double quotes
                    println!("Task marked as complete by executor.");
                    let history = get_task_history().await.unwrap_or_default();
                    final_answer = history
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "Task completed.".to_string());
                    break;
                } else if result.to_uppercase().contains("ERROR") {
                    println!(
                        "Executor reported an error: {}. Continuing cautiously.",
                        result
                    );
                    if result.contains("ERROR_PARSING_JSON") {
                        if let Err(e) = add_task_to_history(format!(
                            "Step {}: Agent response was not valid JSON. Previous response: {}",
                            current_step, string_response
                        ))
                        .await
                        {
                            eprintln!("Failed to add task to history: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Critical error executing task: {}", e);
                if let Err(err) = add_task_to_history(format!(
                    "Step {}: Critical error during execution: {}",
                    current_step, e
                ))
                .await
                {
                    eprintln!("Failed to add task to history: {}", err);
                }
                final_answer = format!("Critical error during execution: {}", e);
                break;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    if current_step >= max_steps {
        println!("Reached maximum steps ({}). Exiting loop.", max_steps);
        let history = get_task_history().await.unwrap_or_default();
        final_answer = format!(
            "Reached max steps. Last known state: {}",
            history.last().unwrap_or(&"N/A".to_string())
        );
    }

    println!("\n--- Orchestrator Finished ---");
    if !final_answer.is_empty() {
        println!("Orchestrator Final Answer: {}", final_answer);
    } else {
        println!(
            "Orchestrator: No explicit final answer was set. Task may have ended due to other reasons (e.g., max steps)."
        );
        println!("Last few history entries:");
        let history = get_task_history().await.unwrap_or_default();
        for entry in history.iter().rev().take(5) {
            println!("- {}", entry);
        }
    }

    println!("Quitting browser...");
    match driver.quit().await {
        Ok(_) => println!("Browser quit successfully."),
        Err(e) => eprintln!("Failed to quit browser: {}", e),
    }

    Ok(())
}

// Fix: Change _current_url type from Future to String
fn gen_prompt(_high_level_plan: String, _current_url: String, _task_history: String) -> String {
    format!(
        "Here is the high-level plan:
{_high_level_plan}

Here is the current URL:
{_current_url}

Here is the task history:
{_task_history}

Based on the high-level plan, the current URL, and the task history, determine the next action to take. If the plan is not perfect, you might have to change the plan based on the given page. Output your decision in the JSON format specified above."

    )
}

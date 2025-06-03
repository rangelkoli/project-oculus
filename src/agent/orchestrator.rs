use crate::agent::executor::execute_task;
use crate::agent::planner::planner_agent;
use crate::agent::task::task_agent;
use project_oculus::browser_control::browser_use::create_new_browser;
use project_oculus::browser_control::interactive_elements::get_interactive_elements_in_hashmap;
use serde_json::Value;
use std::error::Error;

pub async fn orchestrator_agent(_planner_response: String) -> Result<(), Box<dyn Error>> {
    let mut string_response: String = String::new();
    let mut task_history: Vec<String> = Vec::new();
    let mut objective: String = String::from("No initial objective set by planner yet.");

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

    match planner_agent().await {
        Ok(ai_response) => {
            println!("Planner AI Response: {}", ai_response);
            string_response = ai_response;
            task_history.push(format!("Initial plan from planner: {}", string_response));
        }
        Err(e) => {
            eprintln!("Error generating planner AI response: {}", e);
            if let Err(quit_err) = driver.quit().await {
                eprintln!("Failed to quit browser after planner error: {}", quit_err);
            }
            return Err(e.into());
        }
    }

    let mut parsed_planner_response = string_response
        .replace("```json", "")
        .replace("```", "")
        .replace("\n", "");

    println!("Parsed Planner AI Response: {}", parsed_planner_response);
    let json_res: Result<Value, serde_json::Error> = serde_json::from_str(&parsed_planner_response);

    match json_res {
        Ok(json_value) => {
            println!("Parsed Planner JSON: {:?}", json_value);
            if let Some(obj_str) = json_value.get("objective").and_then(|v| v.as_str()) {
                objective = obj_str.to_string();
                println!("Overall Objective set by Planner: {}", objective);
            } else if let Some(current_state) = json_value.get("current_state") {
                if let Some(next_goal) = current_state.get("next_goal").and_then(|v| v.as_str()) {
                    objective = format!("Planner's initial goal: {}", next_goal);
                    println!(
                        "Overall Objective (from planner's next_goal): {}",
                        objective
                    );
                }
            }
            println!("Planner provided initial actions. Proceeding to main loop.");
        }
        Err(e) => {
            eprintln!(
                "Error parsing planner JSON response: {}. Response was: {}",
                e, parsed_planner_response
            );
        }
    };

    let mut final_answer: String = String::new();
    let max_steps = 10;
    let mut current_step = 0;

    println!("Entering main task execution loop...");
    loop {
        if current_step >= max_steps {
            println!("Reached maximum steps ({}). Exiting loop.", max_steps);
            final_answer = format!(
                "Reached max steps. Last known state: {}",
                task_history.last().unwrap_or(&"N/A".to_string())
            );
            break;
        }
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

        let history_str = if task_history.is_empty() {
            String::from("No previous actions taken.")
        } else {
            format!(
                "Task History (most recent last):
{}",
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

        let task_agent_input = format!(
            "Overall Objective: {}\n\n{}\n\nCurrent Interactive Elements on page:\n{}\n\nBased on the objective, history, and current elements, decide the next action.",
            objective, history_str, elements_string
        );

        println!(
            "Input for Task Agent:\n{}\n------------------------",
            task_agent_input
        );

        if current_step > 1 || !parsed_planner_response.contains("action_name") {
            match task_agent(task_agent_input).await {
                Ok(ai_response) => {
                    println!("Task Agent AI Response: {}", ai_response);
                    string_response = ai_response.clone();
                    task_history.push(format!(
                        "Step {}: Task Agent decided: {}",
                        current_step, ai_response
                    ));
                }
                Err(e) => {
                    eprintln!("Error generating task agent AI response: {}", e);
                    task_history.push(format!(
                        "Step {}: Error from Task Agent: {}",
                        current_step, e
                    ));
                    println!("Breaking loop due to task_agent error.");
                    final_answer = format!("Error in task agent: {}", e);
                    break;
                }
            }
        } else {
            println!(
                "Step {}: Using planner's response for execution: {}",
                current_step, string_response
            );
            task_history.push(format!(
                "Step {}: Using planner's initial action: {}",
                current_step, string_response
            ));
        }

        match execute_task(&string_response, &driver).await {
            Ok(result) => {
                println!("Execution Result: {}", result);
                task_history.push(format!(
                    "Step {}: Execution result: {}",
                    current_step, result
                ));

                if result.starts_with("FINAL_ANSWER:") {
                    final_answer = result.replace("FINAL_ANSWER: ", "").trim().to_string();
                    println!("Final answer received: {}", final_answer);
                    break;
                } else if result.to_uppercase() == "TASK_COMPLETE"
                    || result.to_uppercase() == "\"TASK_COMPLETE\""
                {
                    // Fixed double quotes
                    println!("Task marked as complete by executor.");
                    final_answer = task_history
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
                        task_history.push(format!(
                            "Step {}: Agent response was not valid JSON. Previous response: {}",
                            current_step, string_response
                        ));
                    }
                }
            }
            Err(e) => {
                eprintln!("Critical error executing task: {}", e);
                task_history.push(format!(
                    "Step {}: Critical error during execution: {}",
                    current_step, e
                ));
                final_answer = format!("Critical error during execution: {}", e);
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("\n--- Orchestrator Finished ---");
    if !final_answer.is_empty() {
        println!("Orchestrator Final Answer: {}", final_answer);
    } else {
        println!(
            "Orchestrator: No explicit final answer was set. Task may have ended due to other reasons (e.g., max steps)."
        );
        println!("Last few history entries:");
        for entry in task_history.iter().rev().take(3).rev() {
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

fn gen_prompt(_high_level_plan: String, _current_url: &str, _task_history: String) -> String {
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

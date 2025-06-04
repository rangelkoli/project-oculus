use crate::agent::executor::execute_task;
use crate::agent::task::task_agent;
use crate::agent::task::{add_task_to_history, get_task_history};
use project_oculus::browser_control::browser_use::create_new_browser;
use project_oculus::browser_control::interactive_elements::get_interactive_elements_in_hashmap;
use std::error::Error;
use std::fmt::Display;

// Helper to add to history and log errors
async fn try_add_to_history<T: Display>(entry: T) {
    if let Err(e) = add_task_to_history(entry.to_string()).await {
        eprintln!("Failed to add task to history: {}", e);
    }
}

// Helper to get last history entry or fallback
async fn get_last_history_or(fallback: &str) -> String {
    get_task_history()
        .await
        .unwrap_or_default()
        .last()
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

/// Orchestrator agent: manages execution of a high-level plan by delegating web and non-web tasks, tracking state, and aggregating results.
pub async fn orchestrator_agent(_planner_response: String) -> Result<(), Box<dyn Error>> {
    println!("Orchestrator starting...");
    let driver = match create_new_browser().await {
        Ok(d) => {
            println!("Browser initialized successfully.");
            d.goto("https://www.duckduckgo.com").await?;
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
    let mut completed_tasks: Vec<String> = Vec::new();
    let mut plan_steps: Vec<String> = _planner_response
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    while current_step < max_steps && !plan_steps.is_empty() {
        current_step += 1;
        println!("\n--- Orchestrator Step {} ---", current_step);

        // 1. Decompose plan: take the next step
        let current_goal = plan_steps.remove(0);
        println!("Current goal: {}", current_goal);

        // 2. Gather state: interactive elements, history, current URL
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
        let interactive_elements_str = if interactive_elements.is_empty() {
            String::from("No interactive elements found.")
        } else {
            interactive_elements
                .iter()
                .map(|(k, v)| format!("{}: {:?}", k, v))
                .collect::<Vec<_>>()
                .join("\n")
        };
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
        let current_url = match driver.current_url().await {
            Ok(url) => url.to_string(),
            Err(e) => {
                eprintln!("Failed to get current URL: {}", e);
                String::from("Unknown (error getting URL)")
            }
        };

        // 3. Delegate task: create prompt for task agent
        let task_agent_input = gen_prompt(
            current_goal.clone(),
            current_url,
            history_str,
            interactive_elements_str,
        );
        println!(
            "Input for Task Agent:\n{}\n------------------------",
            task_agent_input
        );
        let task_agent_result = task_agent(task_agent_input.clone()).await;
        println!("Task Agent AI Response: {:?}", task_agent_result);

        match task_agent_result {
            Ok(ai_response) => {
                println!("Task Agent AI Response: {}", ai_response);
                try_add_to_history(format!(
                    "Step {}: Task Agent decided: {}",
                    current_step, ai_response
                ))
                .await;
                // 4. Execute delegated task (web or non-web)
                match execute_task(ai_response.clone(), &driver).await {
                    Ok(result) => {
                        println!("Execution Result: {}", result);
                        try_add_to_history(format!(
                            "Step {}: Execution result: {}",
                            current_step, result
                        ))
                        .await;
                        if result.starts_with("FINAL_ANSWER:") {
                            final_answer = result.replace("FINAL_ANSWER: ", "").trim().to_string();
                            println!("Final answer received: {}", final_answer);
                            completed_tasks.push(current_goal.clone());
                            break;
                        } else if result
                            .trim_matches('"')
                            .eq_ignore_ascii_case("TASK_COMPLETE")
                        {
                            println!("Task marked as complete by executor.");
                            final_answer = get_last_history_or("Task completed.").await;
                            completed_tasks.push(current_goal.clone());
                            break;
                        } else if result.to_uppercase().contains("ERROR") {
                            println!(
                                "Executor reported an error: {}. Continuing cautiously.",
                                result
                            );
                            if result.contains("ERROR_PARSING_JSON") {
                                try_add_to_history(format!(
                                    "Step {}: Agent response was not valid JSON. Previous response: {}",
                                    current_step, ai_response
                                ))
                                .await;
                            }
                        } else {
                            completed_tasks.push(current_goal.clone());
                        }
                    }
                    Err(e) => {
                        eprintln!("Critical error executing task: {}", e);
                        try_add_to_history(format!(
                            "Step {}: Critical error during execution: {}",
                            current_step, e
                        ))
                        .await;
                        final_answer = format!("Critical error during execution: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error generating task agent AI response: {}", e);
                try_add_to_history(format!(
                    "Step {}: Error from Task Agent: {}",
                    current_step, e
                ))
                .await;
                println!("Breaking loop due to task_agent error.");
                final_answer = format!("Error in task agent: {}", e);
                break;
            }
        }
        println!(
            "Step {}: Using planner's response for execution: {}",
            current_step, current_goal
        );
        try_add_to_history(format!(
            "Step {}: Using planner's initial action: {}",
            current_step, current_goal
        ))
        .await;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    if current_step >= max_steps || plan_steps.is_empty() {
        println!(
            "Reached maximum steps ({}), or plan completed. Exiting loop.",
            max_steps
        );
        let last = get_last_history_or("N/A").await;
        final_answer = format!(
            "Reached max steps or plan completed. Last known state: {}",
            last
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
fn gen_prompt(
    _high_level_plan: String,
    _current_url: String,
    _task_history: String,
    _interactive_elements: String,
) -> String {
    format!(
        "Here is the high-level plan:
{_high_level_plan}

Here is the current URL:
{_current_url}

Here is the task history:
{_task_history}

Here are the interactive elements on the page:
{_interactive_elements}

Based on the high-level plan, the current URL, the task history, and the interactive elements, determine the next action to take. If the plan is not perfect, you might have to change the plan based on the given page. Output your decision in the JSON format specified above."
    )
}

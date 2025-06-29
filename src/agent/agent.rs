use crate::agent::executor::execute_task;
use crate::agent::task::task_agent;
use crate::agent::task::{add_task_to_history, get_task_history};
use project_oculus::browser_control::interactive_elements::get_interactive_elements_in_hashmap;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use thirtyfour::WebDriver;
// Define the orchestrator trait that agents can call

#[derive(Debug, Clone)]
pub struct TaskRecord {
    pub step: usize,
    pub action: String,
    pub result: String,
    pub timestamp: std::time::SystemTime,
}

pub struct AIAgent {
    id: String,
    goal: String,
    description: String,
    tools: String,
    role: String,
    backstory: String,
    context: String,
    driver: WebDriver,
    task_history: Vec<TaskRecord>,
    extracted_urls: HashSet<String>,
    last_screenshot: Option<String>,
}

impl AIAgent {
    pub fn new(
        id: String,
        goal: String,
        description: String,
        tools: String,
        role: String,
        backstory: String,
        context: String,
        driver: WebDriver,
    ) -> Self {
        AIAgent {
            id,
            goal,
            description,
            tools,
            role,
            backstory,
            context,
            driver,
            task_history: Vec::new(),
            extracted_urls: HashSet::new(),
            last_screenshot: None,
        }
    }

    pub fn get_task_history(&self) -> &Vec<TaskRecord> {
        &self.task_history
    }

    pub fn clear_task_history(&mut self) {
        self.task_history.clear();
    }

    pub fn has_extracted_content_from_url(&self, url: &str) -> bool {
        self.extracted_urls.contains(url)
    }

    pub fn mark_url_as_extracted(&mut self, url: String) {
        self.extracted_urls.insert(url);
    }

    fn add_task_record(&mut self, step: usize, action: String, result: String) {
        let record = TaskRecord {
            step,
            action,
            result,
            timestamp: std::time::SystemTime::now(),
        };
        self.task_history.push(record);
    }

    // Helper to add to external task history
    async fn try_add_to_external_history<T: std::fmt::Display>(&self, entry: T) {
        if let Err(e) = add_task_to_history(entry.to_string()).await {
            eprintln!("Failed to add task to history: {}", e);
        }
    }

    // Helper to get last history entry or fallback
    async fn get_last_history_or(&self, fallback: &str) -> String {
        get_task_history()
            .await
            .unwrap_or_default()
            .last()
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }

    pub async fn process(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        self._invoke_loop().await
    }

    async fn _invoke_loop(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        println!(
            "Agent {} starting work towards goal: {}",
            self.id, self.goal
        );

        let mut final_answer: String = String::new();
        let max_steps = 25;
        let mut current_step = 0;
        let mut completed_tasks: Vec<String> = Vec::new();
        let mut plan_steps: Vec<String> = self
            .goal
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // If goal is a single line, treat it as one step
        if plan_steps.len() == 1 {
            plan_steps = vec![self.goal.clone()];
        }

        while current_step < max_steps && !plan_steps.is_empty() {
            current_step += 1;
            println!("\n--- Agent Step {} ---", current_step);

            // 1. Get the current goal/step
            let current_goal = if plan_steps.len() == 1 {
                plan_steps[0].clone()
            } else {
                plan_steps.remove(0)
            };
            println!("Current goal: {}", current_goal);

            // 2. Gather state: interactive elements, history, current URL
            let interactive_elements = match get_interactive_elements_in_hashmap(&self.driver).await
            {
                Ok(elements) => elements,
                Err(e) => {
                    eprintln!("Failed to get interactive elements: {}. Continuing.", e);
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

            let current_url = match self.driver.current_url().await {
                Ok(url) => url.to_string(),
                Err(e) => {
                    eprintln!("Failed to get current URL: {}", e);
                    String::from("Unknown (error getting URL)")
                }
            };
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            // 3. Create prompt for task agent
            let task_agent_input = self.gen_prompt(
                current_goal.clone(),
                current_url,
                history_str,
                interactive_elements_str,
            );

            println!(
                "Input for Task Agent:\n{}\n------------------------",
                task_agent_input
            );

            // 4. Get AI response from task agent
            let task_agent_result = task_agent(task_agent_input.clone()).await;

            match task_agent_result {
                Ok(ai_response) => {
                    println!("Task Agent AI Response: {}", ai_response);
                    self.try_add_to_external_history(format!(
                        "Step {}: Task Agent decided: {}",
                        current_step, ai_response
                    ))
                    .await;

                    // Parse the AI response to determine the next action
                    let next_action: String = match serde_json::from_str::<Value>(&ai_response) {
                        Ok(json_value) => {
                            if let Some(action) = json_value.get("next_action") {
                                action.to_string().trim_matches('"').to_string()
                            } else {
                                eprintln!("No 'next_action' found in AI response.");
                                "CONTINUE".to_string()
                            }
                        }
                        Err(e) => {
                            eprintln!("Error parsing AI response JSON: {}", e);
                            "CONTINUE".to_string()
                        }
                    };

                    // 5. Execute the action
                    match self.execute_action(ai_response.clone(), next_action).await {
                        Ok(result) => {
                            println!("Execution Result: {}", result);
                            self.add_task_record(
                                current_step,
                                current_goal.clone(),
                                result.clone(),
                            );
                            self.try_add_to_external_history(format!(
                                "Step {}: Execution result: {}",
                                current_step, result
                            ))
                            .await;

                            if result.starts_with("FINAL_ANSWER:") {
                                final_answer =
                                    result.replace("FINAL_ANSWER: ", "").trim().to_string();
                                println!("Final answer received: {}", final_answer);
                                completed_tasks.push(current_goal.clone());
                                break;
                            } else if result.trim_matches('"').eq_ignore_ascii_case("AGENT_DONE") {
                                println!("Task marked as complete by executor.");
                                final_answer = self.get_last_history_or("Task completed.").await;
                                completed_tasks.push(current_goal.clone());
                                if plan_steps.len() == 1 || plan_steps.is_empty() {
                                    break;
                                }
                            } else if result
                                .trim_matches('"')
                                .eq_ignore_ascii_case("CONTENT_ALREADY_EXTRACTED")
                            {
                                println!(
                                    "Content already extracted from current URL. Agent should try a different action."
                                );
                                self.try_add_to_external_history(format!(
                                    "Step {}: Content already extracted from current URL",
                                    current_step
                                ))
                                .await;
                            } else if result.to_uppercase().contains("ERROR") {
                                println!(
                                    "Executor reported an error: {}. Continuing cautiously.",
                                    result
                                );
                                if result.contains("ERROR_PARSING_JSON") {
                                    self.try_add_to_external_history(format!(
                                        "Step {}: Agent response was not valid JSON. Previous response: {}",
                                        current_step, ai_response
                                    )).await;
                                }
                            } else {
                                completed_tasks.push(current_goal.clone());
                            }
                        }
                        Err(e) => {
                            eprintln!("Critical error executing task: {}", e);
                            self.try_add_to_external_history(format!(
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
                    self.try_add_to_external_history(format!(
                        "Step {}: Error from Task Agent: {}",
                        current_step, e
                    ))
                    .await;
                    final_answer = format!("Error in task agent: {}", e);
                    break;
                }
            }

            // Highlight all interactive elements and take a screenshot
            let screenshot_path = format!("images/interactive_elements_step_{}.png", current_step);
            if !interactive_elements.is_empty() {
                match self
                    .highlight_and_screenshot_interactive_elements(
                        &interactive_elements,
                        &screenshot_path,
                    )
                    .await
                {
                    Ok(b64) => self.last_screenshot = Some(b64),
                    Err(e) => eprintln!("Failed to highlight elements or take screenshot: {}", e),
                }
            }

            // Small delay between steps
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        if current_step >= max_steps {
            println!("Reached maximum steps ({}). Exiting loop.", max_steps);
            let last = self.get_last_history_or("N/A").await;
            final_answer = format!("Reached max steps. Last known state: {}", last);
        }

        println!("\n--- Agent {} Finished ---", self.id);
        if !final_answer.is_empty() {
            println!("Agent Final Answer: {}", final_answer);
            Ok(final_answer)
        } else {
            let summary = format!(
                "Agent {} completed {} tasks",
                self.id,
                completed_tasks.len()
            );
            println!("{}", summary);
            Ok(summary)
        }
    }

    async fn execute_action(
        &mut self,
        string_response: String,
        action: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        println!("Executing action: {}", action);

        // Get current URL before executing
        let current_url = match self.driver.current_url().await {
            Ok(url) => url.to_string(),
            Err(e) => {
                eprintln!("Failed to get current URL: {}", e);
                String::from("Unknown")
            }
        };

        // Check if the action is extract_content and URL already extracted
        if let Ok(action_json) = serde_json::from_str::<serde_json::Value>(&action) {
            if action_json.get("extract_content").is_some() {
                if self.has_extracted_content_from_url(&current_url) {
                    println!(
                        "Content already extracted from URL: {}. Skipping extraction.",
                        current_url
                    );
                    return Ok("CONTENT_ALREADY_EXTRACTED".to_string());
                }
            }
        }

        let result = execute_task(string_response, &self.driver, action.clone()).await?;

        // If extract_content was successful, mark URL as extracted
        if let Ok(action_json) = serde_json::from_str::<serde_json::Value>(&action) {
            if action_json.get("extract_content").is_some() && result == "CONTINUE" {
                self.mark_url_as_extracted(current_url.clone());
                println!("Marked URL as extracted: {}", current_url);
            }
        }

        println!("Action executed successfully.");
        Ok(result)
    }

    fn gen_prompt(
        &self,
        high_level_plan: String,
        current_url: String,
        task_history: String,
        interactive_elements: String,
    ) -> String {
        // include screenshot data if available
        let screenshot_info = if let Some(ref b64) = self.last_screenshot {
            format!("\nScreenshot (base64):\n{}", b64)
        } else {
            String::new()
        };
        let extracted_urls_info = if self.extracted_urls.is_empty() {
            String::from("No URLs have had their content extracted yet.")
        } else {
            format!(
                "URLs that have already had their content extracted:\n{}",
                self.extracted_urls
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let current_url_extracted_status = if self.has_extracted_content_from_url(&current_url) {
            "\n\nIMPORTANT: Content has already been extracted from the current URL. Consider taking a different action instead of extracting content again."
        } else {
            ""
        };

        // Construct prompt with aligned placeholders and arguments
        format!(
            "Agent Role: {}\nAgent Backstory: {}\nAgent Goal: {}\nAgent Tools: {}\nContext: {}\n\n\
Here is the current task to accomplish:\n{}\n\n\
Here is the current URL:\n{}\n\n\
Here is the task history:\n{}\n\n\
Here are the interactive elements on the page:\n{}\n\n\
{}\n{}\n{}\n\n\
Based on your role, goal, and the current task, determine the next action to take. Use the available tools and context to make the best decision. Output your decision in the JSON format specified.",
            self.role,
            self.backstory,
            self.goal,
            self.tools,
            self.context,
            high_level_plan,
            current_url,
            task_history,
            interactive_elements,
            extracted_urls_info,
            current_url_extracted_status,
            screenshot_info
        )
    }

    /// Highlights all interactive elements with a colored border and overlays a styled label, then takes a screenshot.
    async fn highlight_and_screenshot_interactive_elements(
        &self,
        interactive_elements: &std::collections::HashMap<String, thirtyfour::By>,
        screenshot_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Collect all elements first to avoid multiple await calls
        let mut elements_to_highlight = Vec::new();

        for (index, locator) in interactive_elements.values().enumerate() {
            if let Ok(elements) = self.driver.find_all(locator.clone()).await {
                for element in elements {
                    elements_to_highlight.push((element, index));
                    // Only take the first 200 elements
                    if elements_to_highlight.len() >= 200 {
                        break;
                    }
                }
            }
            // Break outer loop if we have 200 elements
            if elements_to_highlight.len() >= 200 {
                break;
            }
        }

        // Remove previous highlights if any
        let cleanup_script = r#"
            const containerId = 'playwright-highlight-container';
            const container = document.getElementById(containerId);
            if (container) container.remove();
            // Remove all overlays and labels
            document.querySelectorAll('.playwright-highlight-label').forEach(e => e.remove());
        "#;
        let _ = self.driver.execute(cleanup_script, vec![]).await;

        // Create a single script to highlight all elements at once
        if !elements_to_highlight.is_empty() {
            let script = r#"
                const colors = [
                    '#FF0000', '#00FF00', '#0000FF', '#FFA500', '#800080', '#008080',
                    '#FF69B4', '#4B0082', '#FF4500', '#2E8B57', '#DC143C', '#4682B4'
                ];
                const containerId = 'playwright-highlight-container';
                let container = document.getElementById(containerId);
                if (!container) {
                    container = document.createElement('div');
                    container.id = containerId;
                    container.style.position = 'fixed';
                    container.style.pointerEvents = 'none';
                    container.style.top = '0';
                    container.style.left = '0';
                    container.style.width = '100%';
                    container.style.height = '100%';
                    container.style.zIndex = '2147483640';
                    container.style.backgroundColor = 'transparent';
                    document.body.appendChild(container);
                }
                for (let i = 0; i < arguments.length; i += 2) {
                    const element = arguments[i];
                    const index = arguments[i + 1];
                    const color = colors[index % colors.length];
                    const backgroundColor = color + '1A'; // 10% opacity
                    const rects = element.getClientRects();
                    if (!rects || rects.length === 0) continue;
                    // Draw overlays for each rect
                    for (const rect of rects) {
                        if (rect.width === 0 || rect.height === 0) continue;
                        const overlay = document.createElement('div');
                        overlay.style.position = 'fixed';
                        overlay.style.border = `2px solid ${color}`;
                        overlay.style.backgroundColor = backgroundColor;
                        overlay.style.pointerEvents = 'none';
                        overlay.style.boxSizing = 'border-box';
                        overlay.style.top = `${rect.top}px`;
                        overlay.style.left = `${rect.left}px`;
                        overlay.style.width = `${rect.width}px`;
                        overlay.style.height = `${rect.height}px`;
                        container.appendChild(overlay);
                    }
                    // Add a single label for the first rect
                    const firstRect = rects[0];
                    const label = document.createElement('div');
                    label.className = 'playwright-highlight-label';
                    label.textContent = index;
                    label.style.position = 'fixed';
                    label.style.background = color;
                    label.style.color = 'white';
                    label.style.padding = '1px 4px';
                    label.style.borderRadius = '4px';
                    label.style.fontSize = `${Math.min(12, Math.max(8, firstRect.height / 2))}px`;
                    label.style.fontWeight = 'bold';
                    label.style.zIndex = '2147483647';
                    // Position label at top-right of the first rect
                    let labelTop = firstRect.top + 2;
                    let labelLeft = firstRect.left + firstRect.width - 22;
                    if (firstRect.width < 24 || firstRect.height < 16) {
                        labelTop = firstRect.top - 16;
                        labelLeft = firstRect.left + firstRect.width - 22;
                        if (labelLeft < 0) labelLeft = firstRect.left;
                    }
                    label.style.top = `${Math.max(0, Math.min(labelTop, window.innerHeight - 16))}px`;
                    label.style.left = `${Math.max(0, Math.min(labelLeft, window.innerWidth - 22))}px`;
                    container.appendChild(label);
                }
            "#;

            // Prepare arguments array
            let mut args = Vec::new();
            for (element, index) in &elements_to_highlight {
                args.push(element.to_json()?);
                args.push(serde_json::json!(index));
            }

            // Execute single script with all elements
            let _ = self.driver.execute(script, args).await;
        }

        // Ensure the images directory exists
        if let Some(parent) = std::path::Path::new(screenshot_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Take screenshot
        let png_data = self.driver.screenshot_as_png().await?;
        std::fs::write(screenshot_path, &png_data)?;
        // convert to base64 for AI prompt
        let b64 = base64::encode(&png_data);
        Ok(b64)
    }
}

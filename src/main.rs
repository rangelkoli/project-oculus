mod agent;
mod prompts;
mod utils;
use crate::agent::executor::execute_task;
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
    let mut string_response: String = String::new();
    tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(async {
            // Initialize the browser and perform any necessary setup
            let response = planner_agent().await;
            match response {
                Ok(ai_response) => {
                    println!("AI Response: {}", ai_response);
                    string_response = ai_response;
                }
                Err(e) => {
                    eprintln!("Error generating AI response: {}", e);
                }
            }
        });
    let mut final_anser: String = String::new();

    // Parse the AI response to remove ````json` and `\n`
    string_response = string_response
        .replace("```json", "")
        .replace("```", "")
        .replace("\n", "");
    println!("Parsed AI Response: {}", string_response);
    let json_res: Result<Value, serde_json::Error> = serde_json::from_str(&string_response);

    let driver = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(create_new_browser())
        .expect("Failed to create new browser");

    match json_res {
        Ok(json_value) => {
            println!("Parsed JSON: {:?}", json_value);
            if json_value["action"][0]["go_to_url"].is_object() {
                tokio::runtime::Runtime::new()
                    .expect("Failed to create Tokio runtime")
                    .block_on(go_to_url(
                        &driver,
                        json_value["action"][0]["go_to_url"]["url"]
                            .as_str()
                            .expect("URL should be a string"),
                    ))
                    .expect("Failed to navigate to URL");
            } else if json_value["action"][0]["extract_content"].is_object() {
                println!("Opening a new tab.");

                tokio::runtime::Runtime::new()
                    .expect("msg")
                    .block_on(async {
                        let content = extract_content(&driver)
                            .await
                            .expect("Failed to extract content");
                        println!("Extracted Content: {}", content);
                    });
            } else {
                println!("No action to open a new tab.");
            }
        }
        Err(e) => {
            eprintln!("Error parsing JSON response: {}", e);
        }
    };

    loop {
        let interactive_elements = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(get_interactive_elements_in_hashmap(&driver))
            .expect("Failed to get interactive elements");
        let last_step_string = interactive_elements
            .iter()
            .map(|(k, v)| format!("{}: {:?}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        println!("Interactive Elements: {}", last_step_string);
        // Generate AI response based on the last step
        tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(async {
                let res = task_agent(format!("{}{}", PLANNER_PROMPT, last_step_string)).await;
                println!("Task agent completed successfully.");
                match res {
                    Ok(ai_response) => {
                        println!("AI Response: {}", ai_response);
                        string_response = ai_response;
                    }
                    Err(e) => {
                        eprintln!("Error generating AI response: {}", e);
                    }
                }
            });
        // Execute the task based on the AI response
        let execute_result = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(execute_task(&string_response, &driver));
        match execute_result {
            Ok(result) => {
                println!("Task executed successfully: {}", result);

                // Check for final answer
                if result.starts_with("FINAL_ANSWER:") {
                    final_anser = result.replace("FINAL_ANSWER: ", "");
                    println!("Final answer received: {}", final_anser);
                    break;
                }
                // Check for task completion
                else if result == "TASK_COMPLETE" {
                    println!("Task completed successfully.");
                    break;
                }
                // Check for errors
                else if result == "ERROR_PARSING_JSON" {
                    println!("Error parsing AI response, continuing...");
                    continue;
                }
                // Continue execution for "CONTINUE" or other responses
                else {
                    println!("Continuing task execution...");
                }

                // Remove these old checks as they're no longer needed
                // if result == "done" {
                //     break;
                // } else if result.contains("final_goal_reached") {
                //     println!("Final goal reached, exiting loop.");
                //     // Convert the result to a json object
                //     let result_json: Value =
                //         serde_json::from_str(&result).expect("Failed to parse result as JSON");
                //     if let Some(final_answer) = result_json["final_answer"].as_str() {
                //         final_anser = final_answer.to_string();
                //     }
                //     break;
                // }
            }
            Err(e) => {
                eprintln!("Error executing task: {}", e);
                break; // Exit on WebDriver errors
            }
        }

        // Parse the AI response to remove ````json` and `\n`
    }

    // // Wait for user to press 'q' to quit
    // println!("Press 'q' and Enter to quit the browser...");
    // let mut clickable_elements = Vec::new();
    // loop {
    //     let mut input = String::new();
    //     match io::stdin().read_line(&mut input) {
    //         Ok(_) => {
    //             if input.trim().to_lowercase() == "q" {
    //                 break;
    //             } else {
    //                 // tokio::runtime::Runtime::new()
    //                 //     .expect("msg")
    //                 //     .block_on(async {
    //                 //         let content = extract_content(&driver)
    //                 //             .await
    //                 //             .expect("Failed to extract content");
    //                 //         println!("Extracted Content: {}", content);
    //                 //     });

    //                 tokio::runtime::Runtime::new()
    //                     .expect("Failed to create Tokio runtime")
    //                     .block_on(async {
    //                         let interactive_elements =
    //                             get_interactive_elements_in_hashmap(&driver).await;
    //                         match interactive_elements {
    //                             Ok(elements) => {
    //                                 clickable_elements = elements.into_iter().collect::<Vec<_>>();
    //                             }
    //                             Err(e) => {
    //                                 eprintln!("Error retrieving interactive elements: {}", e);
    //                             }
    //                         }
    //                     });
    //             }
    //             println!("{}", clickable_elements.len());
    //             let first_element = clickable_elements.first().unwrap().clone();
    //             let element_locator = first_element.1;

    //             tokio::runtime::Runtime::new()
    //                 .expect("Failed to create Tokio runtime")
    //                 .block_on(async {
    //                     driver
    //                         .find(element_locator)
    //                         .await
    //                         .unwrap()
    //                         .click()
    //                         .await
    //                         .unwrap();
    //                     println!("Clicked on element with selector: {}", first_element.0)
    //                 });
    //         }

    //         Err(e) => {
    //             eprintln!("Error reading input: {}", e);
    //             break;
    //         }
    //     }
    // }

    // Print the final answer if available
    if !final_anser.is_empty() {
        println!("Final Answer: {}", final_anser);
    } else {
        println!("No final answer found.");
    }
    // Close the browser
    tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(driver.quit())
        .expect("Failed to quit browser");
    // Here you can use the `driver` to perform browser actions
    // For example, you can navigate to a URL or perform searches
    println!("Task completed successfully.");
}

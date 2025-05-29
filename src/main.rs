use project_oculus::get_user_input;
mod agent;
use project_oculus::utils::generate_ai_response;
mod prompts;
use crate::prompts::PLANNER_PROMPT;
use project_oculus::browser_control::actions::{extract_content, go_to_url};
use project_oculus::browser_control::browser_use::create_new_browser;
use serde_json::{self, Value}; // Import Value
use std::io::{self, Read};
fn main() {
    let user_task = get_user_input("Enter the task you want to perform: ");
    println!("You entered: {}", user_task);

    let response = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(generate_ai_response(&user_task, PLANNER_PROMPT));
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

    // Wait for user to press 'q' to quit
    println!("Press 'q' and Enter to quit the browser...");
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim().to_lowercase() == "q" {
                    break;
                } else {
                    tokio::runtime::Runtime::new()
                        .expect("msg")
                        .block_on(async {
                            let content = extract_content(&driver)
                                .await
                                .expect("Failed to extract content");
                            println!("Extracted Content: {}", content);
                        });
                }
            }

            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
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

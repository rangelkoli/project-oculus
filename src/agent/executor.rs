use project_oculus::browser_control::actions::{
    click_element, extract_content, extract_information, fill_form, go_to_url,
};
use serde_json::Value;
use std::io::{self, Write};
use thirtyfour::prelude::*;
// ...existing code...
pub async fn execute_task(_string_response: &str, driver: &WebDriver) -> WebDriverResult<String> {
    // Simulate task execution
    let json_res: Result<Value, serde_json::Error> = serde_json::from_str(&_string_response);
    match json_res {
        Ok(ref json_value) if json_value["done"].as_bool() == Some(true) => {
            println!("Task already done, skipping execution.");
            // Check if there's a final answer to return
            if let Some(answer) = json_value["final_answer"].as_str() {
                return Ok(format!("FINAL_ANSWER: {}", answer));
            }
            return Ok("TASK_COMPLETE".to_string());
        }
        Ok(json_value) => {
            println!("Parsed JSON: {:?}", json_value);

            // Check for stop condition first
            if json_value["action"][0]["stop"].as_bool() == Some(true) {
                println!("Stop condition reached.");
                if let Some(answer) = json_value["action"][0]["final_answer"].as_str() {
                    return Ok(format!("FINAL_ANSWER: {}", answer));
                }
                return Ok("TASK_COMPLETE".to_string());
            }

            // Check for final answer action
            if let Some(answer) = json_value["action"][0]["final_answer"]["answer"].as_str() {
                println!("Providing final answer: {}", answer);
                return Ok(format!("FINAL_ANSWER: {}", answer));
            }

            // ...existing code for other actions...
            if let Some(url) = json_value["action"][0]["go_to_url"]["url"].as_str() {
                println!("Navigating to URL: {}", url);
                go_to_url(&driver, url).await?;
            } else if json_value["action"][0]["extract_content"].is_object() {
                println!("Extracting content...");
                let content = extract_content(&driver).await?;
                println!("Extracted Content: {}", content);
            } else if let Some(selector) =
                json_value["action"][0]["click_element"]["selector"].as_str()
            {
                println!("Clicking element with selector: {}", selector);
                click_element(&driver, selector).await?;
            } else if let Some(info) = json_value["action"][0]["extract_information"].as_object() {
                println!("Extracting information from the current page.");
                let extracted_info =
                    extract_information(&driver, _string_response.to_string()).await?;
                let extracted_json: Value =
                    serde_json::from_str(&extracted_info).unwrap_or(Value::Null);
                println!("Extracted Information: {:?}", extracted_json);

                if let Some(final_goal_reached) = extracted_json["final_goal_reached"].as_bool() {
                    if final_goal_reached {
                        println!("Final goal reached, task completed.");
                        // Check if there's a final answer in the extracted info
                        if let Some(answer) = extracted_json["final_answer"].as_str() {
                            return Ok(format!("FINAL_ANSWER: {}", answer));
                        }
                        return Ok("TASK_COMPLETE".to_string());
                    } else {
                        println!("Final goal not reached, continuing execution.");
                    }
                }
            } else if let Some(form_data) = json_value["action"][0]["fill_form"]["data"].as_array()
            {
                println!("Filling form with provided data.");
                let mut form_data_vec = Vec::new();
                for item in form_data {
                    if let (Some(selector), Some(value)) =
                        (item["selector"].as_str(), item["value"].as_str())
                    {
                        form_data_vec.push((selector.to_string(), value.to_string()));
                    }
                }
                fill_form(&driver, &form_data_vec).await?;
            } else {
                println!("No valid action found in JSON.");
            }
        }
        Err(e) => {
            eprintln!("Error parsing JSON response: {}", e);
            return Ok("ERROR_PARSING_JSON".to_string());
        }
    }

    Ok("CONTINUE".to_string())
}

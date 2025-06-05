use project_oculus::browser_control::actions::{
    click_element, extract_content, extract_information, fill_form, go_back, go_to_url,
    search_query,
};
use serde_json::Value;
use thirtyfour::prelude::*;

pub async fn execute_task(_string_response: String, driver: &WebDriver) -> WebDriverResult<String> {
    let json_res: Result<Value, serde_json::Error> = serde_json::from_str(&_string_response);
    println!("Executing task with response: {}", _string_response);

    match json_res {
        Ok(ref json_value) if json_value["done"].as_bool() == Some(true) => {
            println!("Task already done, skipping execution.");
            match json_value["final_answer"].as_str() {
                Some(answer) => Ok(format!("FINAL_ANSWER: {}", answer)),
                None => Ok("TASK_COMPLETE".to_string()),
            }
        }
        Ok(json_value) => {
            println!("Parsed JSON: {:?}", json_value);

            // Check for stop condition first
            match json_value["next_action"][0]["stop"].as_bool() {
                Some(true) => {
                    println!("Stop condition reached.");
                    match json_value["next_action"][0]["final_answer"].as_str() {
                        Some(answer) => Ok(format!("FINAL_ANSWER: {}", answer)),
                        None => Ok("TASK_COMPLETE".to_string()),
                    }
                }
                _ => {
                    // Now match on the next_action fields
                    match (
                        json_value["next_action"]["search_query"]["query"].as_str(),
                        json_value["next_action"]["go_to_url"]["url"].as_str(),
                        json_value["next_action"]["extract_content"].is_object(),
                        json_value["next_action"]["click_element"]["selector"].as_str(),
                        json_value["next_action"]["extract_information"].as_object(),
                        json_value["next_action"]["fill_form"]["data"].as_array(),
                        json_value["next_action"][0]["final_answer"]["answer"].as_str(),
                        json_value["next_action"]["go_back"].is_object(),
                    ) {
                        (Some(query), _, _, _, _, _, _, _) => {
                            println!("Searching for query: {}", query);
                            search_query(&driver, query.to_string()).await?;
                            Ok("CONTINUE".to_string())
                        }
                        (_, Some(url), _, _, _, _, _, _) => {
                            println!("Navigating to URL: {}", url);
                            go_to_url(&driver, url).await?;
                            Ok("CONTINUE".to_string())
                        }
                        (_, _, true, _, _, _, _, _) => {
                            println!("Extracting content...");
                            let content = extract_content(&driver).await?;
                            println!("Extracted Content: {}", content);
                            Ok("CONTINUE".to_string())
                        }
                        (_, _, _, Some(selector), _, _, _, _) => {
                            println!("Clicking element with selector: {}", selector);
                            click_element(&driver, selector).await?;
                            Ok("CONTINUE".to_string())
                        }
                        (_, _, _, _, Some(_info), _, _, _) => {
                            println!("Extracting information from the current page.");
                            let extracted_info =
                                extract_information(&driver, _string_response.to_string()).await?;
                            let extracted_json: Value =
                                serde_json::from_str(&extracted_info).unwrap_or(Value::Null);
                            println!("Extracted Information: {:?}", extracted_json);

                            match extracted_json["final_goal_reached"].as_bool() {
                                Some(true) => match extracted_json["final_answer"].as_str() {
                                    Some(answer) => Ok(format!("FINAL_ANSWER: {}", answer)),
                                    None => Ok("TASK_COMPLETE".to_string()),
                                },
                                _ => {
                                    println!("Final goal not reached, continuing execution.");
                                    Ok("CONTINUE".to_string())
                                }
                            }
                        }
                        (_, _, _, _, _, Some(form_data), _, _) => {
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
                            Ok("CONTINUE".to_string())
                        }
                        (_, _, _, _, _, _, Some(answer), _) => {
                            println!("Providing final answer: {}", answer);
                            Ok(format!("FINAL_ANSWER: {}", answer))
                        }
                        (_, _, _, _, _, _, _, true) => {
                            println!("Going back to previous page.");
                            go_back(&driver).await?;
                            Ok("CONTINUE".to_string())
                        }
                        _ => {
                            println!("No valid action found in JSON.");
                            Ok("CONTINUE".to_string())
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing JSON response: {}", e);
            Ok("ERROR_PARSING_JSON".to_string())
        }
    }
}

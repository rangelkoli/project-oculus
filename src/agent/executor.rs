use project_oculus::browser_control::actions::{
    click_element, extract_content, extract_information, fill_form,
    fill_form_with_user_input_credentials, go_back, go_to_url, search_query,
};
use serde_json::Value;
use thirtyfour::prelude::*;

pub async fn execute_task(
    _string_response: String,
    driver: &WebDriver,
    _next_action: String,
) -> WebDriverResult<String> {
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

            // Parse _next_action parameter as JSON and extract the action type
            let next_action_json: Result<Value, serde_json::Error> =
                serde_json::from_str(&_next_action);

            match next_action_json {
                Ok(action_obj) => {
                    // Check which action is present in the JSON object
                    if action_obj.get("search_query").is_some() {
                        if let Some(query) = action_obj["search_query"]["query"].as_str() {
                            println!("Searching for query: {}", query);
                            search_query(&driver, query.to_string()).await?;
                        }
                        Ok("CONTINUE".to_string())
                    } else if action_obj.get("go_to_url").is_some() {
                        if let Some(url) = action_obj["go_to_url"]["url"].as_str() {
                            println!("Navigating to URL: {}", url);
                            go_to_url(&driver, url).await?;
                        }
                        Ok("CONTINUE".to_string())
                    } else if action_obj.get("extract_content").is_some() {
                        println!("Extracting content...");
                        let content = extract_content(&driver).await?;
                        println!("Extracted Content: {}", content);
                        Ok("CONTINUE".to_string())
                    } else if action_obj.get("click_element").is_some() {
                        if let Some(selector) = action_obj["click_element"]["selector"].as_str() {
                            println!("Clicking element with selector: {}", selector);
                            click_element(&driver, selector).await?;
                        }
                        Ok("CONTINUE".to_string())
                    } else if action_obj.get("extract_information").is_some() {
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
                    } else if action_obj.get("fill_form").is_some() {
                        if let Some(form_data) = action_obj["fill_form"]["data"].as_array() {
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
                        }
                        Ok("CONTINUE".to_string())
                    } else if action_obj.get("final_answer").is_some() {
                        if let Some(answer) = action_obj["final_answer"]["answer"].as_str() {
                            println!("Providing final answer: {}", answer);
                            Ok(format!("FINAL_ANSWER: {}", answer))
                        } else {
                            Ok("TASK_COMPLETE".to_string())
                        }
                    } else if action_obj.get("go_back").is_some() {
                        println!("Going back to previous page.");
                        go_back(&driver).await?;
                        Ok("CONTINUE".to_string())
                    } else if action_obj
                        .get("fill_form_with_user_input_credentials")
                        .is_some()
                    {
                        if let Some(form_data) =
                            action_obj["fill_form_with_user_input_credentials"]["data"].as_array()
                        {
                            println!("Filling form with user input credentials.");
                            let mut form_data_vec = Vec::new();
                            for item in form_data {
                                if let Some(selector) = item.as_str() {
                                    form_data_vec.push(selector.to_string());
                                }
                            }
                            fill_form_with_user_input_credentials(&driver, &form_data_vec).await?;
                        }
                        Ok("CONTINUE".to_string())
                    } else if action_obj.get("stop").is_some() {
                        println!("Stop condition reached.");
                        match action_obj["stop"]["final_answer"].as_str() {
                            Some(answer) => Ok(format!("FINAL_ANSWER: {}", answer)),
                            None => Ok("TASK_COMPLETE".to_string()),
                        }
                    } else {
                        println!("No recognized action found in next_action JSON");
                        Ok("CONTINUE".to_string())
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing next_action JSON: {}", e);
                    Ok("ERROR_PARSING_ACTION".to_string())
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing JSON response: {}", e);
            Ok("ERROR_PARSING_JSON".to_string())
        }
    }
}

use crate::utils::generate_ai_response;
use thirtyfour::prelude::*;
pub async fn go_to_url(driver: &WebDriver, url: &str) -> WebDriverResult<()> {
    print!("Navigating to URL: {}", url);
    driver.new_tab().await?;
    driver.goto(url).await?;
    Ok(())
}

pub async fn extract_content(driver: &WebDriver) -> WebDriverResult<String> {
    print!("Extracting content from the current page.");
    let content = driver.find(By::Tag("body")).await?.text().await?;
    Ok(content)
}

pub async fn click_element(driver: &WebDriver, selector: &str) -> WebDriverResult<()> {
    print!("Clicking element with selector: {}", selector);
    let element = driver.find(By::Css(selector)).await?;
    element.click().await?;
    Ok(())
}

pub async fn fill_form(driver: &WebDriver, form_data: &[(String, String)]) -> WebDriverResult<()> {
    print!("Filling form with provided data.");
    for (selector, value) in form_data {
        let element = driver.find(By::Css(selector)).await?;
        element.send_keys(value).await?;
    }
    Ok(())
}

pub async fn extract_information(
    driver: &WebDriver,
    _current_state: String,
) -> WebDriverResult<String> {
    print!("Analyzing the current page.");

    let content = driver.find(By::Tag("body")).await?.text().await?;

    let prompt = format!(
        "Analyze the current page and extract relevant information given the current task and the body of the content: {}, {}. I want you to give the answer in the following format: ```json {{
        \"final_goal_reached\": true or false,
        \"final_answer\": \"The final answer to the question\"
        }}```",
        _current_state,
        content.trim()
    );

    let response = generate_ai_response(&prompt, "").await;
    // Handle the AI response

    match response {
        Ok(ai_response) => {
            // Assuming the AI response is a JSON string
            let cleaned_response = ai_response
                .replace("```json", "")
                .replace("```", "")
                .replace("\n", "");
            println!("AI Response: {}", cleaned_response);
            return Ok(cleaned_response);
        }
        Err(e) => {
            eprintln!("Error generating AI response: {}", e);
            return Ok("Error: Failed to generate AI response".to_string());
        }
    }
}

pub async fn search_query(driver: &WebDriver, _search_term: String) -> WebDriverResult<()> {
    print!("Searching for: {}", _search_term);
    let encoded_term = urlencoding::encode(&_search_term);
    let search_url = format!("https://duckduckgo.com/?q={}", encoded_term);
    driver.goto(&search_url).await?;
    Ok(())
}

pub async fn go_back(driver: &WebDriver) -> WebDriverResult<()> {
    print!("Going back to the previous page.");
    driver.back().await?;
    Ok(())
}

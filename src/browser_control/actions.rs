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

// Open a new tab with a specific URL
pub async fn open_new_tab(driver: &WebDriver, url: &str) -> WebDriverResult<()> {
    print!("Opening new tab: {}", url);
    driver.new_tab().await?;
    driver.goto(url).await?;
    Ok(())
}

// Close the current tab
pub async fn close_current_tab(driver: &WebDriver) -> WebDriverResult<()> {
    print!("Closing current tab.");
    driver.close_window().await?;
    Ok(())
}

// Accept cookies by clicking a selector (commonly used for cookie banners)
pub async fn accept_cookies(driver: &WebDriver, selector: &str) -> WebDriverResult<()> {
    print!("Accepting cookies with selector: {}", selector);
    let element = driver.find(By::Css(selector)).await?;
    element.click().await?;
    Ok(())
}

// Go back to the previous page
pub async fn go_back(driver: &WebDriver) -> WebDriverResult<()> {
    print!("Going back to previous page.");
    driver.back().await?;
    Ok(())
}

// Refresh the current page
pub async fn refresh_page(driver: &WebDriver) -> WebDriverResult<()> {
    print!("Refreshing the current page.");
    driver.refresh().await?;
    Ok(())
}

// Solve captcha placeholder (actual implementation would require integration with a captcha solving service)
pub async fn solve_captcha(_driver: &WebDriver) -> WebDriverResult<()> {
    print!("Solving captcha (not implemented).");
    Ok(())
}

// Close popup/modal by clicking a selector
pub async fn close_popup(driver: &WebDriver, selector: &str) -> WebDriverResult<()> {
    print!("Closing popup with selector: {}", selector);
    let element = driver.find(By::Css(selector)).await?;
    element.click().await?;
    Ok(())
}

// Scroll to a specific element by CSS selector
pub async fn scroll_to_element(driver: &WebDriver, selector: &str) -> WebDriverResult<()> {
    print!("Scrolling to element with selector: {}", selector);
    let element = driver.find(By::Css(selector)).await?;
    driver
        .execute(
            "arguments[0].scrollIntoView({behavior: 'smooth', block: 'center'});",
            vec![element.to_json()?],
        )
        .await?;
    Ok(())
}

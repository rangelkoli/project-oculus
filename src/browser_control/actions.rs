use crate::utils::generate_ai_response;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
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

    // Generate AI summary of the page content
    let prompt = format!(
        "Summarize the following webpage content and identify the most important text on the page: {}",
        content.trim()
    );

    match generate_ai_response(&prompt, "").await {
        Ok(summary) => {
            println!("Page Summary: {}", summary);
            Ok(format!("{}\n\nSummary: {}", content, summary))
        }
        Err(e) => {
            eprintln!("Error generating summary: {}", e);
            Ok(content)
        }
    }
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

pub async fn fill_form_with_data(
    driver: &WebDriver,
    form_data: &[(String, String)],
) -> WebDriverResult<()> {
    print!("Filling form with provided data.");
    for (selector, value) in form_data {
        let element = driver.find(By::Css(selector)).await?;
        element.send_keys(value).await?;
    }

    Ok(())
}

pub async fn fill_form_with_user_input_credentials(
    driver: &WebDriver,
    input_cred_selector: &[(String)],
) -> WebDriverResult<()> {
    print!("Filling form with user-provided data.");

    for selector in input_cred_selector {
        print!("Enter value for {}: ", selector);
        io::stdout().flush().unwrap(); // Ensure the prompt is displayed immediately
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let value = input.trim();

        let element = driver.find(By::Css(selector)).await?;
        element.send_keys(value).await?;
    }
    Ok(())
}

pub async fn create_document(
    _driver: &WebDriver,
    filename: &str,
    content: &str,
    format_type: &str,
) -> WebDriverResult<String> {
    println!(
        "Creating document: {} with format: {}",
        filename, format_type
    );

    // Ensure the documents directory exists
    let documents_dir = "documents";
    if !Path::new(documents_dir).exists() {
        if let Err(e) = fs::create_dir_all(documents_dir) {
            eprintln!("Failed to create documents directory: {}", e);
            return Ok(format!(
                "Error: Failed to create documents directory: {}",
                e
            ));
        }
    }

    let file_path = match format_type.to_lowercase().as_str() {
        "markdown" | "md" => format!("{}/{}.md", documents_dir, filename),
        "text" | "txt" => format!("{}/{}.txt", documents_dir, filename),
        "json" => format!("{}/{}.json", documents_dir, filename),
        "html" => format!("{}/{}.html", documents_dir, filename),
        _ => format!("{}/{}.txt", documents_dir, filename), // Default to txt
    };

    // Write content to file
    if let Err(e) = fs::write(&file_path, content) {
        eprintln!("Failed to write file {}: {}", file_path, e);
        return Ok(format!("Error: Failed to write file {}: {}", file_path, e));
    }

    println!("Document saved successfully: {}", file_path);
    Ok(format!("Document saved: {}", file_path))
}

pub async fn generate_and_save_document(
    _driver: &WebDriver,
    task_description: &str,
    filename: &str,
    format_type: &str,
) -> WebDriverResult<String> {
    println!("Generating document for task: {}", task_description);

    let prompt = match format_type.to_lowercase().as_str() {
        "markdown" | "md" => format!(
            "Create a well-structured markdown document for the following task: {}. 
            Include appropriate headings, formatting, and organization. 
            Make it professional and comprehensive.",
            task_description
        ),
        "text" | "txt" => format!(
            "Create a well-structured text document for the following task: {}. 
            Make it clear, organized, and professional.",
            task_description
        ),
        "json" => format!(
            "Create a JSON document containing structured data for the following task: {}. 
            Ensure valid JSON format with appropriate structure.",
            task_description
        ),
        "html" => format!(
            "Create a complete HTML document for the following task: {}. 
            Include proper HTML structure, styling, and semantic markup.",
            task_description
        ),
        _ => format!(
            "Create a document for the following task: {}",
            task_description
        ),
    };

    match generate_ai_response(&prompt, "").await {
        Ok(generated_content) => {
            // Clean up the content if it contains markdown code blocks
            let cleaned_content =
                if format_type.to_lowercase() == "markdown" || format_type.to_lowercase() == "md" {
                    generated_content
                        .replace("```markdown", "")
                        .replace("```", "")
                        .trim()
                        .to_string()
                } else if format_type.to_lowercase() == "html" {
                    generated_content
                        .replace("```html", "")
                        .replace("```", "")
                        .trim()
                        .to_string()
                } else if format_type.to_lowercase() == "json" {
                    generated_content
                        .replace("```json", "")
                        .replace("```", "")
                        .trim()
                        .to_string()
                } else {
                    generated_content
                };

            create_document(_driver, filename, &cleaned_content, format_type).await
        }
        Err(e) => {
            eprintln!("Error generating document content: {}", e);
            Ok(format!("Error: Failed to generate document: {}", e))
        }
    }
}

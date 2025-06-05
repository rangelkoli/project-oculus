use reqwest;
use serde_json::{Value, json};
use std::env;
use std::io::{self, Write};

// General function to prompt user, get input, and return it
pub fn get_user_input(prompt: &str) -> String {
    // Print the prompt without a newline
    print!("{}", prompt);
    // Flush stdout to ensure the prompt is displayed immediately
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line from user");

    // Trim whitespace (including the newline character) and return
    input.trim().to_string()
}

pub async fn generate_ai_response(
    prompt: &str,
    system_instructions: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key =
        env::var("GEMINI_API_KEY").map_err(|_| "GEMINI_API_KEY environment variable not set")?;

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    let payload = json!({
        "system_instruction": {
            "parts": [
                {
                    "text": system_instructions
                }
            ]
        },
        "contents": [
            {
                "parts": [
                    {
                        "text": prompt
                    }
                ]
            }
        ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }

    let response_json: Value = response.json().await?;

    // Extract the generated text from the response
    let generated_text = response_json
        .get("candidates")
        .and_then(|candidates| candidates.get(0))
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.get(0))
        .and_then(|part| part.get("text"))
        .and_then(|text| text.as_str())
        .unwrap_or("No response generated");

    Ok(generated_text.to_string())
}

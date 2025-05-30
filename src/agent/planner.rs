use gemini_rust::Gemini;
use tokio;

#[tokio::main]
pub async fn planner_agent() -> Result<String, Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")?;
    let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string());
    let client = Gemini::with_model(&api_key, model);

    // Create a Google Search tool
    // let google_search_tool = Tool::google_search();

    let response = client
        .generate_content()
        .with_user_message("hi")
        .execute()
        .await?;

    println!("Response: {}", response.text());

    Ok(response.text())
}

use crate::prompts::PLANNER_PROMPT;
use project_oculus::utils::generate_ai_response;

pub async fn planner_agent() -> Result<String, Box<dyn std::error::Error>> {
    let user_task = project_oculus::utils::get_user_input("Enter the task you want to perform: ");
    println!("You entered: {}", user_task);

    let response = generate_ai_response(&user_task, PLANNER_PROMPT).await;
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

    Ok(string_response)
}

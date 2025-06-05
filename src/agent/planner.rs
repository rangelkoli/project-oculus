use project_oculus::utils::generate_ai_response;

pub async fn planner_agent() -> Result<String, Box<dyn std::error::Error>> {
    let user_task = project_oculus::utils::get_user_input("Enter the task you want to perform: ");
    println!("User task for planner: {}", user_task);

    // The input to the planner_agent is now just the user_task.
    // The PLANNER_PROMPT guides the LLM to structure its response, including the user_task in its output JSON.

    let prompt_input = gen_prompt(&user_task);

    let response = generate_ai_response(&prompt_input, "").await;
    let mut string_response: String = String::new();

    match response {
        Ok(ai_response) => {
            println!("Planner AI Raw Response: {}", ai_response);
            string_response = ai_response;
        }
        Err(e) => {
            // Log the error and return it, so the orchestrator can handle it.
            eprintln!("Error generating planner AI response: {}", e);
            return Err(e.into());
        }
    }

    // Basic cleanup of the response. More robust JSON parsing will be needed.
    // The LLM should ideally return clean JSON, but this is a fallback.
    string_response = string_response
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();

    println!("Cleaned Planner AI Response: {}", string_response);

    // At this stage, the planner_agent returns the JSON string.
    // The orchestrator will be responsible for parsing this JSON and extracting the plan details.
    Ok(string_response)
}

fn gen_prompt(user_task: &str) -> String {
    format!(
        r#"
        Generate a high-level, step-by-step plan for the orchestrator AI to accomplish the user's request using its browser access and other available tools.

Context:

User Request: {} (This will be filled with the specific task from the user)
Orchestrator Capabilities:
Access and interact with web browsers (navigate, click, type, scrape data, etc.).
Access to general knowledge.
Ability to execute code (specify languages if applicable, e.g., Python for data manipulation).
(Add any other specific tools or capabilities your orchestrator agent possesses).
Goal: Create a plan that is logical, efficient, and breaks down the user's request into manageable steps for the orchestrator. Each step should be an actionable instruction.
Instructions for the Planner Agent:

Understand the User's Core Goal: Analyze the {{USER_REQUEST}} to identify the fundamental objective. What does the user ultimately want to achieve?
Identify Key Information Needed: Determine what information is required to fulfill the request. Consider if this information needs to be sourced from the web.
Outline Major Steps: Break down the task into a sequence of high-level actions. For each step, consider:
What is the immediate goal of this step?
What actions will the orchestrator need to perform (e.g., "Navigate to a specific URL," "Search for information on a topic," "Extract specific data points from a webpage," "Summarize findings")?
If web interaction is needed, what kind of sites or search queries would be most effective? (You don't need to provide exact URLs unless they are explicitly given or obvious).
Maintain Simplicity and Clarity: Each step in the plan should be clear, concise, and unambiguous. Avoid overly complex or compound steps.
Consider Dependencies: Ensure the steps are in a logical order. If one step depends on the outcome of another, reflect this in the sequence.
Error Handling/Contingencies (Optional but Recommended): Briefly consider potential issues (e.g., website not available, information not found) and suggest very high-level alternative approaches if applicable. For example, "If initial search yields no results, try alternative search terms."
Define the Final Output/Deliverable: What should be the end result of executing the plan? (e.g., "A summary of X," "A list of Y," "Confirmation that Z action has been completed").
Output Format for the Plan:

Please provide the plan as a numbered list of actionable steps.

Example (for a hypothetical user request: "Find the current weather in London and the top 3 news headlines from BBC News"):

Determine current weather in London:
Search a reliable weather website (e.g., Google Weather, AccuWeather) for "weather in London."
Extract the current temperature, conditions (e.g., sunny, rainy), and wind speed.
Find top 3 news headlines from BBC News:
Navigate to the BBC News website.
Identify and extract the top 3 visible news headlines from the homepage.
Compile and present the information:
Combine the gathered weather information and news headlines into a concise summary.
Now, based on the User Request: {}, generate the plan."#,
        user_task, user_task
    )
}

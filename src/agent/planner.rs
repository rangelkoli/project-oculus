use project_oculus::utils::generate_ai_response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStep {
    pub id: String,                    // e.g., "web", "data", etc.
    pub parameters: serde_json::Value, // All parameters needed to create the agent
    pub run_in_parallel: bool,
    pub needs_context_from: Option<usize>, // Index of previous agent step to get context from
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlannerAgentPlan {
    pub steps: Vec<AgentStep>,
}

pub async fn planner_agent() -> Result<String, Box<dyn std::error::Error>> {
    let user_task = project_oculus::utils::get_user_input("Enter the task you want to perform: ");
    println!("User task for planner: {}", user_task);

    let mut string_response: String = String::new();
    // Generate the prompt for the planner AI.
    let instructions = generate_ai_response(
        &gen_prompt(&user_task),
        "You are a planning agent that creates detailed execution plans for accomplishing user tasks.",
    ).await;
    println!(
        "Instructions for planner AI: {}",
        instructions
            .as_deref()
            .unwrap_or("No instructions provided")
    );

    match instructions {
        Ok(response) => {
            string_response = response;
        }
        Err(e) => {
            eprintln!("Error generating planner AI response: {}", e);
        }
    }

    string_response = string_response
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();

    println!("Cleaned Planner AI Response: {}", string_response);

    // Try to parse the plan for validation (optional, can be used by orchestrator)
    let _plan: Result<PlannerAgentPlan, _> = serde_json::from_str(&string_response);
    if let Err(e) = &_plan {
        eprintln!(
            "Warning: Could not parse planner response as PlannerAgentPlan: {}",
            e
        );
    }

    Ok(string_response)
}

fn gen_prompt(user_task: &str) -> String {
    format!(
        r#"
Generate a high-level, step-by-step plan for the orchestrator AI to accomplish the user's request using its browser access and other available tools.

Context:

User Request: {user_task}
Orchestrator Capabilities:
Access and interact with web browsers (navigate, click, type, scrape data, etc.).
Access to general knowledge.
Ability to execute code (specify languages if applicable, e.g., Python for data manipulation).
(Add any other specific tools or capabilities your orchestrator agent possesses).
Goal: Create a plan that is logical, efficient, and breaks down the user's request into manageable steps for the orchestrator. Each step should be an actionable instruction.

Instructions for the Planner Agent:
- For each step, output a JSON object with the following structure:
  - id: String (unique identifier for the agent step, e.g., "ResponseAgent1")
  - parameters: Object containing:
    - goal: String (the specific goal for this agent step)
    - description: String (a brief description of what this agent does)
    - tools: String (tools or APIs this agent will use)
    - role: String (the role or type of agent, e.g., "WebSearchAgent")
    - backstory: String (background or context for the agent, can be brief)
    - context: String (any context or input needed from previous steps, or empty if none)
  - run_in_parallel: Boolean (true if this agent can run in parallel with others)
  - needs_context_from: Integer or null (index of previous step to get context from, or null)
- Output the plan as a JSON object: {{"steps": [{{...}}, ...]}}
- Do NOT output markdown or code block markers, only raw JSON.
- Example output:
{{
  "steps": [
    {{
      "id": "ResponseAgent1",
      "parameters": {{
        "goal": "Find the top 3 footballers 2024",
        "description": "Searches the web for the top 3 footballers in 2024.",
        "tools": "web_search",
        "role": "WebSearchAgent",
        "backstory": "An agent skilled at searching the web for sports rankings.",
        "context": ""
      }},
      "run_in_parallel": false,
      "needs_context_from": null
    }},
    ...
  ]
}}
Now, based on the User Request: {user_task}, generate the plan."#
    )
}

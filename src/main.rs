mod agent;
mod prompts;
mod utils;

use thirtyfour::common::print;

use crate::agent::planner::{AgentStep, PlannerAgentPlan, planner_agent};
use std::error::Error;
use thirtyfour::WebDriver;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Application starting...");

    let planner_response = planner_agent().await;
    match planner_response {
        Ok(response) => {
            println!("Planner AI Response: {}", response);
            // Debug: print the raw response before parsing
            println!("Raw planner response for debug: {:?}", response);
            // If the response is already a JSON object, use it directly
            let cleaned_response = response
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
                .to_string();

            println!("Cleaned planner response for debug: {:?}", cleaned_response);
            // Parse the plan
            let plan: PlannerAgentPlan = match serde_json::from_str(&cleaned_response) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!(
                        "Failed to parse planner response as plan: {}\nResponse was: {}",
                        e, cleaned_response
                    );
                    return Err(e.into());
                }
            };
            // Store outputs for context passing
            let mut agent_outputs: Vec<Option<String>> = vec![None; plan.steps.len()];
            for (i, step) in plan.steps.iter().enumerate() {
                // Try to extract agent parameters from top-level fields, fallback to parameters object
                let id = step.id.clone();
                let goal = step
                    .parameters
                    .get("goal")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let description = step
                    .parameters
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tools = step
                    .parameters
                    .get("tools")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let role = step
                    .parameters
                    .get("role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let backstory = step
                    .parameters
                    .get("backstory")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let agent_context = if let Some(ctx_idx) = step.needs_context_from {
                    agent_outputs
                        .get(ctx_idx)
                        .cloned()
                        .flatten()
                        .unwrap_or_default()
                } else if let Some(ctx) = step.parameters.get("context").and_then(|v| v.as_str()) {
                    ctx.to_string()
                } else {
                    String::new()
                };
                // Create WebDriver instance for each agent (or share if needed)
                let driver = WebDriver::new(
                    "http://localhost:4444",
                    thirtyfour::DesiredCapabilities::chrome(),
                )
                .await?;
                let mut agent = crate::agent::agent::AIAgent::new(
                    id,
                    goal,
                    description,
                    tools,
                    role,
                    backstory,
                    agent_context,
                    driver,
                );
                // Run the agent
                let result = agent.process().await;
                match result {
                    Ok(output) => {
                        println!("Agent step {} output: {}", i, output);
                        agent_outputs[i] = Some(output);
                    }
                    Err(e) => {
                        eprintln!("Agent step {} failed: {}", i, e);
                        agent_outputs[i] = None;
                    }
                }
            }
            println!("All agent steps finished. Outputs: {:?}", agent_outputs);
        }
        Err(e) => {
            eprintln!("Error generating planner AI response: {}", e);
        }
    }

    println!("Application finished.");
    Ok(())
}

mod agent;
mod prompts;
mod utils;

use crate::agent::orchestrator::orchestrator_agent;
use crate::agent::planner::planner_agent;
use std::error::Error;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Application starting...");

    let planner_response = planner_agent().await;
    match planner_response {
        Ok(response) => {
            println!("Planner AI Response: {}", response);
            // Here you can handle the planner's response, e.g., log it or pass it to the orchestrator.
            orchestrator_agent(response).await?;
        }
        Err(e) => {
            eprintln!("Error generating planner AI response: {}", e);
            // Depending on desired behavior, you might want to exit with an error code
            // std::process::exit(1);
        }
    }

    println!("Application finished.");
    Ok(())
}

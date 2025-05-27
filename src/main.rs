mod browser_control;
mod utils;
use browser_control::browser_use::new_browser;
use std::io::{self, Write};
use utils::get_user_input;
fn main() {
    // Initialize the Tokio runtime
    let mut website_url = get_user_input("Enter the website URL (without 'https://'): ");

    let search_term = get_user_input("Enter what you want to search for: ");
    match new_browser(
        website_url.trim(), // Pass the trimmed URL to the new_browser function
        search_term.trim(), // Pass the trimmed search term to the new_browser function
    ) {
        Ok(_) => println!("Test passed!"),
        Err(e) => eprintln!("Test failed: {}", e),
    }
}

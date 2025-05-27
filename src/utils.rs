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

use std::io::{self, Write};

fn main() {
    // Stage 1: print the prompt
    print!("$ ");
    io::stdout().flush().unwrap();

    // Stage 2: read the user's command
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // Strip the trailing newline that read_line includes
    let command = input.trim();

    // Print the error message in the required format
    println!("{}: command not found", command);
}
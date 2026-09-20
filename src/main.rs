use std::io::{self, Write};

fn main() {
    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Fresh buffer each iteration
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let command = input.trim();

        // Split into words. split_whitespace() collapses runs of spaces/tabs
        // and skips empty segments, so "echo   hello" -> ["echo", "hello"].
        let parts: Vec<&str> = command.split_whitespace().collect();

        match parts.first() {
            Some(&"exit") => break,
            Some(&"echo") => {
                // Join everything after the command name with a single space.
                println!("{}", parts[1..].join(" "));
            }
            _ => println!("{}: command not found", command),
        }
    }
}
use std::io::{self, Write};

const BUILTINS: &[&str] = &["echo", "exit", "type"];

fn main() {
    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Fresh buffer each iteration
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let command = input.trim();
        let parts: Vec<&str> = command.split_whitespace().collect();

        match parts.first() {
            Some(&"exit") => break,
            Some(&"echo") => println!("{}", parts[1..].join(" ")),
            Some(&"type") => {
                // type takes one argument: the command to inspect
                if let Some(&arg) = parts.get(1) {
                    if BUILTINS.contains(&arg) {
                        println!("{} is a shell builtin", arg);
                    } else {
                        println!("{}: not found", arg);
                    }
                }
                // If no argument was given, do nothing (prints a fresh prompt)
            }
            Some(_) => println!("{}: command not found", command),
            None => {} // empty input — just loop again
        }
    }
}
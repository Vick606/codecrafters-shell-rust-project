use std::io::{self, Write};

fn main() {
    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Fresh buffer each iteration — do not declare this outside the loop
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let command = input.trim();
        println!("{}: command not found", command);
    }
}
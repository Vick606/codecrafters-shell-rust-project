use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const BUILTINS: &[&str] = &["echo", "exit", "type"];

fn find_in_path(command: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(command);
        let Ok(meta) = fs::metadata(&candidate) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        if meta.permissions().mode() & 0o111 != 0 {
            return Some(candidate);
        }
    }
    None
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let command = input.trim();
        let parts: Vec<&str> = command.split_whitespace().collect();

        match parts.first() {
            Some(&"exit") => break,
            Some(&"echo") => println!("{}", parts[1..].join(" ")),
            Some(&"type") => {
                if let Some(&arg) = parts.get(1) {
                    if BUILTINS.contains(&arg) {
                        println!("{} is a shell builtin", arg);
                    } else if let Some(path) = find_in_path(arg) {
                        println!("{} is {}", arg, path.display());
                    } else {
                        println!("{}: not found", arg);
                    }
                }
            }
            Some(&cmd) => {
                if find_in_path(cmd).is_some() {
                    // Spawn with the bare name so argv[0] is "custom_exe", not the full path.
                    // status() inherits stdout/stderr, so the child's output goes to our terminal.
                    let _ = Command::new(cmd)
                        .args(&parts[1..])
                        .status();
                } else {
                    // Use the command name, not the full line, in the error message.
                    println!("{}: command not found", cmd);
                }
            }
            None => {}
        }
    }
}
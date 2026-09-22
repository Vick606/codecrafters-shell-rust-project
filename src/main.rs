use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const BUILTINS: &[&str] = &["echo", "exit", "type", "pwd", "cd"];

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

enum QuoteState {
    None,
    Single,
    Double,
}

/// Split a command line into arguments, honouring single and double quotes.
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut state = QuoteState::None;
    let mut has_token = false;

    for ch in input.chars() {
        match state {
            QuoteState::None => {
                if ch == '\'' {
                    state = QuoteState::Single;
                    has_token = true;
                } else if ch == '"' {
                    state = QuoteState::Double;
                    has_token = true;
                } else if ch.is_whitespace() {
                    if has_token {
                        tokens.push(std::mem::take(&mut current));
                        has_token = false;
                    }
                } else {
                    current.push(ch);
                    has_token = true;
                }
            }
            QuoteState::Single => {
                if ch == '\'' {
                    state = QuoteState::None;
                } else {
                    current.push(ch);
                }
            }
            QuoteState::Double => {
                if ch == '"' {
                    state = QuoteState::None;
                } else {
                    current.push(ch);
                }
            }
        }
    }

    if has_token {
        tokens.push(current);
    }

    tokens
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let owned_parts = tokenize(&input);
        let parts: Vec<&str> = owned_parts.iter().map(|s| s.as_str()).collect();

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
            Some(&"pwd") => {
                if let Ok(cwd) = env::current_dir() {
                    println!("{}", cwd.display());
                }
            }
            Some(&"cd") => {
                if let Some(&target) = parts.get(1) {
                    let expanded: PathBuf = if target == "~" {
                        match env::var_os("HOME") {
                            Some(home) => PathBuf::from(home),
                            None => continue,
                        }
                    } else {
                        PathBuf::from(target)
                    };

                    if env::set_current_dir(&expanded).is_err() {
                        println!("cd: {}: No such file or directory", target);
                    }
                }
            }
            Some(&cmd) => {
                if find_in_path(cmd).is_some() {
                    let _ = Command::new(cmd).args(&parts[1..]).status();
                } else {
                    println!("{}: command not found", cmd);
                }
            }
            None => {}
        }
    }
}
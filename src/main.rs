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
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const BUILTINS: &[&str] = &["echo", "exit", "type"];

/// Search PATH for an executable named `command`.
/// Returns the full path on success, `None` if nothing usable was found.
fn find_in_path(command: &str) -> Option<PathBuf> {
    // var_os, not var: PATH is a filesystem value, not necessarily UTF-8.
    let path_var = env::var_os("PATH")?;

    // split_paths uses ':' on Unix and ';' on Windows — no manual delimiter.
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(command);

        // Missing file, unreadable dir, dangling symlink — skip and continue.
        let Ok(meta) = fs::metadata(&candidate) else {
            continue;
        };

        // A directory with +x is not an executable.
        if !meta.is_file() {
            continue;
        }

        // Any execute bit (user/group/other) set => executable.
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
            Some(_) => println!("{}: command not found", command),
            None => {}
        }
    }
}
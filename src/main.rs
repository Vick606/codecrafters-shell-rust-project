use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut state = QuoteState::None;
    let mut has_token = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match state {
            QuoteState::None => {
                if ch == '\\' {
                    if let Some(next) = chars.next() {
                        current.push(next);
                        has_token = true;
                    }
                } else if ch == '\'' {
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
                if ch == '\\' {
                    match chars.peek() {
                        Some(&'"') | Some(&'\\') => {
                            current.push(chars.next().unwrap());
                        }
                        _ => current.push('\\'),
                    }
                } else if ch == '"' {
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

struct Redirects {
    stdout: Option<PathBuf>,
    stderr: Option<PathBuf>,
}

fn parse_redirects<'a>(parts: &[&'a str]) -> (Vec<&'a str>, Redirects) {
    let mut stdout = None;
    let mut stderr = None;
    let mut cmd_end = parts.len();
    let mut i = 0;

    while i < parts.len() {
        let p = parts[i];
        if p == ">" || p == "1>" {
            if cmd_end == parts.len() {
                cmd_end = i;
            }
            if let Some(t) = parts.get(i + 1) {
                stdout = Some(PathBuf::from(t));
            }
            i += 2;
        } else if p == "2>" {
            if cmd_end == parts.len() {
                cmd_end = i;
            }
            if let Some(t) = parts.get(i + 1) {
                stderr = Some(PathBuf::from(t));
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    let cmd: Vec<&str> = parts[..cmd_end].iter().copied().collect();
    (cmd, Redirects { stdout, stderr })
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let owned_parts = tokenize(&input);
        let parts: Vec<&str> = owned_parts.iter().map(|s| s.as_str()).collect();
        let (cmd_parts, redirects) = parse_redirects(&parts);

        // Redirections are set up before the command runs. For builtins, this
        // means the redirect file must be created even if the builtin produces
        // no output on that stream. External commands get the file via Stdio.
        let is_builtin = matches!(
            cmd_parts.first(),
            Some(&"exit") | Some(&"echo") | Some(&"type") | Some(&"pwd") | Some(&"cd")
        );
        if is_builtin {
            if let Some(ref path) = redirects.stderr {
                let _ = File::create(path);
            }
        }

        match cmd_parts.first() {
            Some(&"exit") => break,
            Some(&"echo") => {
                let output = cmd_parts[1..].join(" ");
                if let Some(ref path) = redirects.stdout {
                    if let Ok(mut f) = File::create(path) {
                        let _ = writeln!(f, "{}", output);
                    }
                } else {
                    println!("{}", output);
                }
            }
            Some(&"type") => {
                if let Some(&arg) = cmd_parts.get(1) {
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
                    if let Some(ref path) = redirects.stdout {
                        if let Ok(mut f) = File::create(path) {
                            let _ = writeln!(f, "{}", cwd.display());
                        }
                    } else {
                        println!("{}", cwd.display());
                    }
                }
            }
            Some(&"cd") => {
                if let Some(&target) = cmd_parts.get(1) {
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
                    let mut command = Command::new(cmd);
                    command.args(&cmd_parts[1..]);
                    if let Some(ref path) = redirects.stdout {
                        if let Ok(file) = File::create(path) {
                            command.stdout(Stdio::from(file));
                        }
                    }
                    if let Some(ref path) = redirects.stderr {
                        if let Ok(file) = File::create(path) {
                            command.stderr(Stdio::from(file));
                        }
                    }
                    let _ = command.status();
                } else {
                    println!("{}: command not found", cmd);
                }
            }
            None => {}
        }
    }
}
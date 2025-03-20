use clinput::{self, App};
use std::fs;
use std::io::{Read, Write as _, stdout};
use std::mem::take;
use std::process::Stdio;
use std::{
    fs::{OpenOptions, remove_file},
    process::Command,
};

fn code(lines: &Vec<String>) -> String {
    let mut code = String::new();
    for line in lines {
        code.push_str(line);
        code.push('\n');
    }
    format!(
        "
#![allow(unused)]

fn main() {{
let x = {{\n{code}\n}};
println!(\"{{x:?}}\");
}}
"
    )
}

fn println_raw(msg: &str) {
    println!("\r{msg}");
}

const SRC_PATH: &str = "file.rs";
const OUT_PATH: &str = "./file";
const LOG_PATH: &str = "logs.txt";

const RUST_C: &str = "rustc";

/// Removes the created files if they exist
fn clean(lines: &mut Vec<String>) {
    lines.clear();
    remove_file(SRC_PATH).unwrap_or_default();
    remove_file(OUT_PATH).unwrap_or_default();
    remove_file(LOG_PATH).unwrap_or_default();
}

fn interpret_code(code: String) -> Result<(), &'static str> {
    fs::write(SRC_PATH, code).map_err(|_| "Access denied.")?;
    let mut errors = String::new();

    Command::new(RUST_C)
        .args([SRC_PATH])
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "Failed to compile.")?
        .stderr
        .ok_or("Failed to fetch stderr.")?
        .read_to_string(&mut errors)
        .map_err(|_| "Failed to fetch errors.")?;

    let fixed_errors = errors.replace("\n", "\r\n");
    let trimmed_errors = fixed_errors.trim();
    if !trimmed_errors.is_empty() {
        println_raw(trimmed_errors);
        return Err("Compilation failed.");
    }

    if Command::new(OUT_PATH)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|_| "Failed to execute.")?
        .wait()
        .map_err(|_| "Failed to end execution.")?
        .success()
    {
        Ok(())
    } else {
        Err("Execution failed")
    }
}

fn main() {
    let mut log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_PATH)
        .unwrap();

    let mut lines = Vec::new();
    let mut unfinished_line: Option<String> = None;

    let mut app = App::new();
    app.action(|app| match app.line() {
        "clear" | "c" => {
            clean(&mut lines);
        }
        "exit" | "e" => {
            app.exit();
        }
        line => {
            stdout().flush().unwrap();
            let line = line.trim();
            let mut newline = match take(&mut unfinished_line) {
                Some(mut newline) => {
                    newline.reserve(line.len());
                    newline
                }
                None => {
                    let mut newline = String::with_capacity(1 + line.len());
                    newline.push(';');
                    newline
                }
            };
            newline.push_str(line);
            if line.ends_with('\\') {
                newline.pop();
                unfinished_line = Some(newline);
                return;
            }
            lines.push(newline);
            if let Err(msg) = interpret_code(code(&lines)) {
                lines.pop();
                println_raw(msg);
            }
            stdout().flush().unwrap();
        }
    });
    app.log(|info| writeln!(log, "{info}").unwrap());

    app.run();
    clean(&mut lines);
}

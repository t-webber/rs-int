use std::{
    fs::{remove_file, write},
    io::{Write as _, stdin, stdout},
    process::{Command, ExitStatus, Stdio},
};

const FILE_PATH: &str = "file.rs";
const OUTPUT_PATH: &str = "./file";
const RUST_C: &str = "rustc";

macro_rules! err_and_fail {
    ($code:expr, $msg:expr) => {{
        match $code {
            Err(_) => fail!($msg),
            Ok(val) => val,
        }
    }};
}

macro_rules! fail {
    ($msg:expr) => {{
        eprintln!($msg);
        continue;
    }};
}

macro_rules! err_unit {
    ($code:expr) => {{ $code.map_err(|_| ()) }};
}

fn flush() {
    stdout()
        .flush()
        .unwrap_or_else(|_| eprintln!("Failed to flush."));
}

fn process(cmd: &mut Command) -> Result<ExitStatus, ()> {
    err_unit!(cmd.spawn()).and_then(|mut ps| err_unit!(ps.wait()))
}

fn code(lines: &[Box<str>]) -> String {
    let mut code = String::new();
    for line in lines {
        code.push_str(line);
    }
    format!(
        r##"
#![allow(unused)]

fn main() {{
let x = {{{code}}};
println!("{{x:?}}");
}}
"##
    )
}

fn clean() {
    remove_file(FILE_PATH).expect("Access denied.");
    remove_file(OUTPUT_PATH).expect("Access denied.");
}

fn main() {
    let mut file = Vec::<Box<str>>::new();
    loop {
        let mut line = String::new();
        print!(">>> ");
        flush();
        err_and_fail!(stdin().read_line(&mut line), "Failed to read line.");
        match line.as_str().trim() {
            "clear" | "c" => {
                clean();
                continue;
            }
            "exit" | "e" => break,
            _ => file.push(line.into_boxed_str()),
        }
        err_and_fail!(write(FILE_PATH, code(&file)), "Access denied");
        if !err_and_fail!(
            process(Command::new(RUST_C).args([FILE_PATH])),
            "Failed to execute."
        )
        .success()
        {
            file.pop();
            fail!("Invalid code. Try again");
        }
        if !err_and_fail!(
            process(Command::new(OUTPUT_PATH).stdin(Stdio::piped())),
            "Failed to execute."
        )
        .success()
        {
            file.pop();
            fail!("Invalid code. Try again.");
        }
        flush();
    }
    clean();
}

//! The specular CLI: a thin caller of the library.
//!
//! Exit codes: 0 = conforms, 1 = does not conform, 2 = spec, verifier, or
//! runtime error. No bypass flags exist.

use std::path::Path;

use specular::{LintError, Report, lint, verify};

fn main() {
    std::process::exit(run());
}

const HELP: &str = include_str!("../HELP.txt");

fn run() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_error("usage: specular <lint|verify> <spec.json>; run `specular --help`");
        return 2;
    }
    if args
        .iter()
        .any(|a| a == "help" || a == "--help" || a == "-h")
    {
        println!("{HELP}");
        return 0;
    }
    if args.len() != 2 {
        print_error("usage: specular <lint|verify> <spec.json>; run `specular --help`");
        return 2;
    }
    if args.iter().any(|a| a.starts_with("--")) {
        print_error("flags are not supported; output is always JSON");
        return 2;
    }
    let command = &args[0];
    let spec_path = Path::new(&args[1]);
    match command.as_str() {
        "lint" => run_lint(spec_path),
        "verify" => run_verify(spec_path),
        other => {
            print_error(&format!("unknown command '{other}'; run `specular --help`"));
            2
        }
    }
}

fn run_lint(spec_path: &Path) -> i32 {
    match lint(spec_path) {
        Ok(_) => {
            println!("{}", serde_json::json!({"result": "pass"}));
            0
        }
        Err(error) => {
            print_lint_error(&error);
            2
        }
    }
}

fn print_lint_error(error: &LintError) {
    if let LintError::InvalidSpec(violations) = error {
        println!(
            "{}",
            serde_json::json!({"result": "fail", "violations": violations})
        );
    } else {
        print_error(&error.to_string());
    }
}

fn run_verify(spec_path: &Path) -> i32 {
    let spec = match lint(spec_path) {
        Ok(spec) => spec,
        Err(error) => {
            print_lint_error(&error);
            return 2;
        }
    };
    let root = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    match verify(&spec, &root, spec_path) {
        Ok(report) => {
            print_report(&report);
            i32::from(!report.conforms)
        }
        Err(error) => {
            print_error(&error.to_string());
            2
        }
    }
}

fn print_report(report: &Report) {
    match serde_json::to_string_pretty(report) {
        Ok(text) => println!("{text}"),
        Err(error) => print_error(&format!("report does not serialize: {error}")),
    }
}

fn print_error(message: &str) {
    println!(
        "{}",
        serde_json::json!({"result": "error", "message": message})
    );
}

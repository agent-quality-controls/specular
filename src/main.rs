//! The spec3 CLI: a thin caller of the library.
//!
//! Exit codes: 0 = conforms, 1 = does not conform, 2 = spec, verifier, or
//! runtime error. No bypass flags exist.

use std::path::Path;

use spec3::{LintError, Report, lint, verify};

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let (Some(command), Some(spec_path)) = (positional.first(), positional.get(1)) else {
        eprintln!("usage: spec3 <lint|verify> <spec.json> [--json]");
        return 2;
    };
    if args.iter().any(|a| a.starts_with("--") && a != "--json") {
        eprintln!("unknown flag; usage: spec3 <lint|verify> <spec.json> [--json]");
        return 2;
    }
    match command.as_str() {
        "lint" => run_lint(Path::new(spec_path), json),
        "verify" => run_verify(Path::new(spec_path), json),
        other => {
            eprintln!("unknown command '{other}'; usage: spec3 <lint|verify> <spec.json> [--json]");
            2
        }
    }
}

fn run_lint(spec_path: &Path, json: bool) -> i32 {
    match lint(spec_path) {
        Ok(_) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"result": "pass", "spec": spec_path.display().to_string()})
                );
            } else {
                println!("LINT PASS: {}", spec_path.display());
            }
            0
        }
        Err(error) => {
            print_lint_error(&error, json);
            2
        }
    }
}

fn print_lint_error(error: &LintError, json: bool) {
    if let LintError::InvalidSpec(violations) = error {
        if json {
            println!(
                "{}",
                serde_json::json!({"result": "fail", "violations": violations})
            );
        } else {
            for violation in violations {
                println!("LINT FAIL [{}] {}", violation.code, violation.message);
            }
        }
    } else if json {
        println!(
            "{}",
            serde_json::json!({"result": "error", "message": error.to_string()})
        );
    } else {
        eprintln!("error: {error}");
    }
}

fn run_verify(spec_path: &Path, json: bool) -> i32 {
    let spec = match lint(spec_path) {
        Ok(spec) => spec,
        Err(error) => {
            print_lint_error(&error, json);
            return 2;
        }
    };
    let root = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    match verify(&spec, &root, spec_path) {
        Ok(report) => {
            print_report(&report, json);
            i32::from(!report.conforms())
        }
        Err(error) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"result": "error", "message": error.to_string()})
                );
            } else {
                eprintln!("error: {error}");
            }
            2
        }
    }
}

fn print_report(report: &Report, json: bool) {
    if json {
        match serde_json::to_string_pretty(report) {
            Ok(text) => println!("{text}"),
            Err(error) => eprintln!("error: report does not serialize: {error}"),
        }
        return;
    }
    println!("spec sha256: {}", report.spec.sha256);
    for stamp in &report.verifier_files {
        println!("verifier {}: sha256 {}", stamp.path, stamp.sha256);
    }
    for diagnostic in &report.git {
        println!("git {}: {}", diagnostic.path, diagnostic.state);
    }
    for item in &report.evidence {
        let status = match item.status {
            spec3::Status::Pass => "pass",
            spec3::Status::Fail => "FAIL",
        };
        match &item.message {
            Some(message) => println!("{} ({}): {status} — {message}", item.id, item.source),
            None => println!("{} ({}): {status}", item.id, item.source),
        }
    }
    let (builtin, custom) = report.source_counts();
    println!(
        "evidence: {} items ({builtin} builtin, {custom} custom); conforms: {}",
        report.evidence.len(),
        report.conforms()
    );
}

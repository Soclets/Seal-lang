//! The one command for the whole build, lint and test run:
//!
//! ```text
//! cargo run -j 3 -p seal-check-all
//! ```
//!
//! It formats-checks, lints with warnings as errors, builds, runs the packages' own tests, then runs every test in
//! `tests/` as `tests/README.md` describes, and writes each one's status to `test-status.json`.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

/// IMPLEMENTATION.md, Building rule 13: Seal's building session uses 3 cores.
const JOBS: &str = "3";

fn step(name: &str, program: &str, arguments: &[&str]) -> bool {
    println!("== {name}");
    let status = Command::new(program).args(arguments).status();
    let passed = status.is_ok_and(|status| status.success());
    if !passed {
        println!("== {name}: FAILED");
    }
    passed
}

/// trace: D141. Runs one test folder from a compiled file: the folder copied to a scratch place, its script built
/// into a `.sealc` there, and run from that. A script that doesn't build is refused as loading its text is, so its
/// messages and status are what the test expects too.
fn run_compiled_test(seal: &Path, folder: &Path, scratch: &Path) -> Option<String> {
    let name = folder.file_name()?;
    let copy = scratch.join(name);
    let _ = fs::create_dir_all(&copy);
    for entry in fs::read_dir(folder).ok()?.filter_map(Result::ok) {
        let _ = fs::copy(entry.path(), copy.join(entry.file_name()));
    }
    let built = match Command::new(seal).args(["build", "program.seal"]).current_dir(&copy).output() {
        Ok(built) => built,
        Err(error) => return Some(format!("seal couldn't start: {error}")),
    };
    if built.status.success() {
        return run_test_as(seal, &copy, "program.sealc");
    }
    let expected_output = fs::read(copy.join("expected.out")).ok()?;
    let expected_status = String::from_utf8_lossy(&fs::read(copy.join("expected.status")).ok()?).trim().to_owned();
    let status = built.status.code().map_or_else(|| "killed".to_owned(), |code| code.to_string());
    if status != expected_status {
        return Some(format!("built with exit status {status}, expected {expected_status}"));
    }
    if built.stdout != expected_output {
        return Some("building gave other messages than expected.out".to_owned());
    }
    None
}

/// Runs one test folder, giving why it failed, or nothing when it passed.
fn run_test(seal: &Path, folder: &Path) -> Option<String> {
    run_test_as(seal, folder, "program.seal")
}

/// Runs one test folder's script, from its text or a compiled file, giving why it failed, or nothing.
fn run_test_as(seal: &Path, folder: &Path, script: &str) -> Option<String> {
    let read = |name: &str| fs::read(folder.join(name)).ok();
    let Some(expected_output) = read("expected.out") else {
        return Some("no expected.out".to_owned());
    };
    let Some(expected_status) = read("expected.status") else {
        return Some("no expected.status".to_owned());
    };
    let expected_status = String::from_utf8_lossy(&expected_status).trim().to_owned();
    let mut child = match Command::new(seal)
        .args(["run", script])
        .current_dir(folder)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return Some(format!("seal couldn't start: {error}")),
    };
    if let Some(mut stdin) = child.stdin.take() {
        // A script that stops before reading all its input closes the pipe; that isn't the test's failure.
        let _ = stdin.write_all(&read("input.txt").unwrap_or_default());
    }
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(error) => return Some(format!("seal couldn't finish: {error}")),
    };
    let status = output.status.code().map_or_else(|| "killed".to_owned(), |code| code.to_string());
    if status != expected_status {
        return Some(format!("exit status {status}, expected {expected_status}"));
    }
    if output.stdout != expected_output {
        return Some("output differs from expected.out".to_owned());
    }
    None
}

fn json_text(text: &str) -> String {
    let mut out = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn main() -> ExitCode {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    if std::env::set_current_dir(&root).is_err() {
        eprintln!("the workspace folder can't be entered");
        return ExitCode::FAILURE;
    }
    let mut all_passed = true;
    all_passed &= step("format", "cargo", &["fmt", "--all", "--check"]);
    all_passed &= step(
        "lint",
        "cargo",
        &["clippy", "-j", JOBS, "--workspace", "--all-targets", "--release", "--", "-D", "warnings"],
    );
    all_passed &= step("build", "cargo", &["build", "-j", JOBS, "--workspace", "--release"]);
    all_passed &=
        step("unit tests", "cargo", &["test", "-j", JOBS, "--workspace", "--release", "--", "--test-threads", JOBS]);

    // Building rule 6: the fuzzer runs with everything else. A fixed seed keeps this run the same every time; longer
    // runs with other seeds are made by hand, as `cargo run --release -p seal-fuzzer -- --seed N --seconds N`.
    all_passed &= step(
        "fuzzer",
        "cargo",
        &["run", "-q", "-j", JOBS, "--release", "-p", "seal-fuzzer", "--", "--seed", "1", "--count", "2000"],
    );

    // D19: Seal's results are compared with Ralph's, as a black box, on the tests and on fuzzed scripts.
    all_passed &= step(
        "comparison with Ralph",
        "cargo",
        &["run", "-q", "-j", JOBS, "--release", "-p", "seal-fuzzer", "--", "--ralph", "--seed", "1", "--count", "400"],
    );

    println!("== tests/");
    let seal = root.join("target/release/seal");
    let mut folders: Vec<PathBuf> = fs::read_dir("tests")
        .map(|entries| entries.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.is_dir()).collect())
        .unwrap_or_default();
    folders.sort();
    let mut status_lines = Vec::new();
    let mut passed = 0usize;
    for folder in &folders {
        let name = folder.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let outcome = run_test(&seal, folder);
        match &outcome {
            None => {
                passed += 1;
                println!("pass  {name}");
            }
            Some(reason) => {
                all_passed = false;
                println!("FAIL  {name}: {reason}");
            }
        }
        let mut line = format!(
            "    {}: {{ \"status\": {}",
            json_text(&name),
            json_text(if outcome.is_none() { "pass" } else { "fail" })
        );
        if let Some(reason) = &outcome {
            let _ = write!(line, ", \"why\": {}", json_text(reason));
        }
        line.push_str(" }");
        status_lines.push(line);
    }
    let failed = folders.len() - passed;
    println!("== tests/: {passed} passed, {failed} failed");

    // D141: every test again, each built into a `.sealc` and run from it, which must give exactly the same.
    println!("== tests/, from compiled files");
    let scratch = std::env::temp_dir().join(format!("seal-check-all-{}", std::process::id()));
    let mut compiled_passed = 0usize;
    for folder in &folders {
        let name = folder.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        match run_compiled_test(&seal, folder, &scratch) {
            None => compiled_passed += 1,
            Some(reason) => {
                all_passed = false;
                println!("FAIL  {name}, compiled: {reason}");
            }
        }
    }
    let _ = fs::remove_dir_all(&scratch);
    println!("== tests/, from compiled files: {compiled_passed} passed, {} failed", folders.len() - compiled_passed);
    let json = format!(
        "{{\n  \"passed\": {passed},\n  \"failed\": {failed},\n  \"tests\": {{\n{}\n  }}\n}}\n",
        status_lines.join(",\n")
    );
    if fs::write("test-status.json", json).is_err() {
        eprintln!("test-status.json can't be written");
        all_passed = false;
    }
    if all_passed {
        println!("== everything passed");
        ExitCode::SUCCESS
    } else {
        println!("== something FAILED");
        ExitCode::FAILURE
    }
}

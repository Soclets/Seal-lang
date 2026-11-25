//! The `seal` command: checks a script, and runs it with the test host (D47, tests/README.md).
//!
//! ```text
//! seal run FILE.seal      load the script with the test host's doors, then make the calls in calls.txt
//! seal check FILE.seal    check the script against the test host's doors, without running it
//! ```
//!
//! The test host's doors are a convention for tests, not part of Seal: `show_msg`, `show_msg.same_line`, `ask`,
//! `rand` and `wait`. `ask` reads standard input, `rand` reads `random.txt` and the calls come from `calls.txt`,
//! both beside the script.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::collections::VecDeque;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use seal::syntax::tree::{Entry, ExpressionKind, Item, StatementKind, UnaryOperator};
use seal::{Argument, Called, Doors, Error, Limits, Message, MessageKind, Pause, Script, Value};

/// tests/README.md: the test host gives every script a step budget of 1,000,000 and a memory cap of 64 MB.
const LIMITS: Limits = Limits { steps: 1_000_000, memory: 64 * 1024 * 1024 };

/// What the test host prints to, remembering whether its last line is still open.
#[derive(Default)]
struct Output {
    line_open: bool,
}

impl Output {
    fn show(&mut self, text: &str, end_line: bool) {
        let mut out = std::io::stdout().lock();
        // A closed pipe means nobody is reading; the script carries on regardless.
        let _ = out.write_all(text.as_bytes());
        if end_line {
            let _ = out.write_all(b"\n");
        }
        let _ = out.flush();
        self.line_open = !end_line;
    }

    /// Ends a line left open, so a message never starts mid-line (Unio D104).
    fn close_line(&mut self) {
        if self.line_open {
            self.show("", true);
        }
    }
}

fn with<T, R>(shared: &Mutex<T>, change: impl FnOnce(&mut T) -> R) -> Option<R> {
    shared.lock().ok().map(|mut guard| change(&mut guard))
}

/// The test host's doors, as tests/README.md describes them.
fn test_doors(output: &Arc<Mutex<Output>>, random: Vec<i64>) -> Result<Doors, Message> {
    let mut doors = Doors::new();
    let shown = Arc::clone(output);
    doors.add("show_msg", move |text: String| {
        with(&shown, |output| output.show(&text, true));
    })?;
    let shown = Arc::clone(output);
    doors.add("show_msg.same_line", move |text: String| {
        with(&shown, |output| output.show(&text, false));
    })?;
    let shown = Arc::clone(output);
    doors.add("ask", move |question: String| -> String {
        with(&shown, |output| output.show(&question, false));
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            // With no more input, the script ends there, quietly, with status 0.
            Ok(0) | Err(_) => std::process::exit(0),
            Ok(_) => {}
        }
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        line
    })?;
    let numbers = Mutex::new(VecDeque::from(random));
    doors
        .add("rand", move |low: i64, _high: i64| -> i64 {
            with(&numbers, VecDeque::pop_front).flatten().unwrap_or(low)
        })?
        .labels(["from", "to"])?;
    doors.add("wait", |_seconds: i64| Pause::<()>::new())?.labels(["seconds"])?;
    Ok(doors)
}

/// A plain value in a call written in `calls.txt`.
fn plain_value(expression: &seal::syntax::tree::Expression) -> Option<Value> {
    match &expression.kind {
        ExpressionKind::Whole(digits) => digits.parse().ok().map(Value::Whole),
        ExpressionKind::Float { text, .. } => text.parse().ok().map(Value::Float),
        ExpressionKind::Bool(flag) => Some(Value::Bool(*flag)),
        ExpressionKind::Text(text) => Some(Value::Text((*text).to_owned())),
        ExpressionKind::Unary { operator: UnaryOperator::Negate, operand } => match &operand.kind {
            ExpressionKind::Whole(digits) => format!("-{digits}").parse().ok().map(Value::Whole),
            ExpressionKind::Float { text, .. } => format!("-{text}").parse().ok().map(Value::Float),
            _ => None,
        },
        _ => None,
    }
}

fn procedure_name(expression: &seal::syntax::tree::Expression) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Name(name) => Some((*name).to_owned()),
        ExpressionKind::Member { receiver, name } => Some(format!("{}.{}", procedure_name(receiver)?, name.text)),
        _ => None,
    }
}

/// Reads one line of `calls.txt`: a Seal call with plain values, as `on_hit(10)` or `heal(1, by: 5)`.
fn read_call(line: &str) -> Option<(String, Vec<Argument>)> {
    let script = seal::syntax::parser::parse(line).ok()?;
    let [Item::Statement(statement)] = script.items.as_slice() else { return None };
    let StatementKind::Expression(expression) = &statement.kind else { return None };
    let ExpressionKind::Call { callee, arguments } = &expression.kind else { return None };
    let name = procedure_name(callee)?;
    let arguments = arguments
        .iter()
        .map(|Entry { label, value }| {
            Some(Argument { label: label.as_ref().map(|label| label.text.to_owned()), value: plain_value(value)? })
        })
        .collect::<Option<Vec<_>>>()?;
    Some((name, arguments))
}

/// The exit status for a message's kind, after Unio's (Unio D56, D82), until Seal's own are decided.
fn status_of(kind: MessageKind) -> u8 {
    match kind {
        MessageKind::Refused => 1,
        MessageKind::NotBuiltYet | MessageKind::NotDesignedYet => 3,
        MessageKind::CheckFailed | MessageKind::LimitReached => 4,
        MessageKind::Failed => 5,
        MessageKind::Internal => 7,
    }
}

fn report(error: &Error, output: &Mutex<Output>) -> ExitCode {
    with(output, Output::close_line);
    eprint!("{error}");
    ExitCode::from(match error {
        Error::Refused(messages) => {
            // A script with refusals exits with 1, even when other stops came too (Unio D148).
            messages.list.iter().map(|message| status_of(message.kind)).min().unwrap_or(1)
        }
        Error::Stopped(message) => status_of(message.kind),
        Error::Failed(_) => 5,
    })
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [command, path] = arguments.as_slice() else {
        eprintln!("usage: seal run FILE.seal, or seal check FILE.seal");
        return ExitCode::from(6);
    };
    if command != "run" && command != "check" {
        eprintln!("seal: `{command}` isn't a command\nusage: seal run FILE.seal, or seal check FILE.seal");
        return ExitCode::from(6);
    }
    let Ok(source) = std::fs::read_to_string(path) else {
        eprintln!("seal: `{path}` can't be read");
        return ExitCode::from(2);
    };
    let folder = Path::new(path).parent().unwrap_or(Path::new("."));
    let random: Vec<i64> = std::fs::read_to_string(folder.join("random.txt"))
        .map(|text| text.lines().filter_map(|line| line.trim().parse().ok()).collect())
        .unwrap_or_default();
    let output = Arc::new(Mutex::new(Output::default()));
    let doors = match test_doors(&output, random) {
        Ok(doors) => doors,
        Err(message) => {
            eprint!("{message}");
            return ExitCode::from(7);
        }
    };
    if command == "check" {
        return match Script::check(path, &source, &doors) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => report(&error, &output),
        };
    }
    let mut script = match Script::load(path, &source, &doors, LIMITS) {
        Ok(script) => script,
        Err(error) => return report(&error, &output),
    };
    let calls = std::fs::read_to_string(folder.join("calls.txt")).unwrap_or_default();
    for line in calls.lines().filter(|line| !line.trim().is_empty()) {
        let Some((name, arguments)) = read_call(line) else {
            with(&output, Output::close_line);
            eprintln!("seal: `{line}` in calls.txt isn't a call with plain values");
            return ExitCode::from(6);
        };
        let mut called = script.call_values(&name, &arguments);
        // The test host resumes a paused script at once.
        while let Ok(Called::Paused(task)) = called {
            called = script.resume_with_value(task, &Value::Nothing);
        }
        if let Err(error) = called {
            return report(&error, &output);
        }
    }
    ExitCode::SUCCESS
}

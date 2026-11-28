//! The `wax` command: checks a script, and runs it with the test host (D47, tests/README.md).
//!
//! ```text
//! wax run FILE.wax                     load the script with the test host's doors, then make the calls in calls.txt
//! wax check FILE.wax                   check the script against the test host's doors, without running it
//! wax build FILE.wax... [-o OUT.waxc] compile scripts for the test host's doors into one `.waxc` file (D141, D153)
//! wax run FILE.waxc [NAME]             load the script named NAME, or the file's only one, and run it the same way
//! wax keygen NAME                       make a key pair for signing: NAME.public for hosts, NAME.secret (D169)
//! ```
//!
//! `wax build` takes `--sign NAME.secret` to sign what it writes, and `wax run` takes `--key NAME.public` to load
//! only a file signed with the matching key, as a host that requires signatures does (D169, D175).
//!
//! `NAME.secret` signs scripts as the game: anyone who has it can. `wax keygen` writes it readable by its owner only;
//! keep it on the machine that builds the game's scripts, and never commit it to a repository or ship it. Hosts get
//! `NAME.public`.
//!
//! The test host's doors are a convention for tests, not part of Wax: `show_msg`, `show_msg.same_line`, `ask`,
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

use wax::syntax::tree::{Entry, ExpressionKind, Item, StatementKind, UnaryOperator};
use wax::{Argument, Called, Doors, Error, Limits, Message, MessageKind, Pause, Script, Value};

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
fn plain_value(expression: &wax::syntax::tree::Expression) -> Option<Value> {
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

fn procedure_name(expression: &wax::syntax::tree::Expression) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Name(name) => Some((*name).to_owned()),
        ExpressionKind::Member { receiver, name } => Some(format!("{}.{}", procedure_name(receiver)?, name.text)),
        _ => None,
    }
}

/// Reads one line of `calls.txt`: a Wax call with plain values, as `on_hit(10)` or `heal(1, by: 5)`.
fn read_call(line: &str) -> Option<(String, Vec<Argument>)> {
    let script = wax::syntax::parser::parse(line).ok()?;
    let [Item::Statement(statement)] = script.items.as_slice() else { return None };
    let StatementKind::Expression(expression) = &statement.kind else { return None };
    let (callee, arguments) = expression.kind.as_call()?;
    let name = procedure_name(callee)?;
    let arguments = arguments
        .iter()
        .map(|Entry { label, value }| {
            Some(Argument { label: label.as_ref().map(|label| label.text.to_owned()), value: plain_value(value)? })
        })
        .collect::<Option<Vec<_>>>()?;
    Some((name, arguments))
}

/// The exit status for a message's kind, after Unio's (Unio D56, D82), until Wax's own are decided.
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

const USAGE: &str = "usage: wax run FILE.wax, wax check FILE.wax, wax build FILE.wax... [-o OUT.waxc] [--sign KEY.secret], wax run FILE.waxc [NAME] [--key KEY.public], or wax keygen NAME";

/// Takes `flag` and the value after it out of the arguments, when they're there.
fn take_option(arguments: &mut Vec<String>, flag: &str) -> Result<Option<String>, ExitCode> {
    let Some(at) = arguments.iter().position(|argument| argument == flag) else { return Ok(None) };
    if at + 1 >= arguments.len() {
        eprintln!("wax: `{flag}` is followed by a file\n{USAGE}");
        return Err(ExitCode::from(6));
    }
    let value = arguments.remove(at + 1);
    arguments.remove(at);
    Ok(Some(value))
}

/// 32 bytes from the system's randomness, for a key or a signature (D175).
fn randomness() -> Result<[u8; 32], ExitCode> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|error| {
        eprintln!("wax: the system's randomness can't be read: {error}");
        ExitCode::from(2)
    })?;
    Ok(bytes)
}

/// Writes a key, a secret one readable and writable by its owner only (on Unix, permissions 600), made new so
/// another's file is never written over.
fn write_key(path: &str, bytes: &[u8], secret: bool) -> std::io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if secret {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(not(unix))]
    let _ = secret;
    options.open(path)?.write_all(bytes)
}

/// trace: D169, D175. `wax keygen NAME`: an ML-DSA-44 key pair, the public key for hosts and the secret one for the
/// build that signs, which stays on the developer's machine.
fn keygen_command(name: &str) -> ExitCode {
    let seed = match randomness() {
        Ok(seed) => seed,
        Err(code) => return code,
    };
    let (public, secret) = wax::host::signing::keys_from_seed(&seed);
    for (path, bytes, secret) in [(format!("{name}.public"), public, false), (format!("{name}.secret"), secret, true)] {
        if let Err(error) = write_key(&path, &bytes, secret) {
            eprintln!("wax: `{path}` can't be written: {error}");
            return ExitCode::from(2);
        }
    }
    // A secret key that leaks lets anyone sign scripts as the game: it stays on the machine that builds them.
    eprintln!(
        "wax: `{name}.secret` signs scripts as your game. Keep it on the machine that builds them, readable by you \
         alone, and never commit it or ship it; hosts get `{name}.public`."
    );
    ExitCode::SUCCESS
}

/// trace: D141, D153. `wax build`: one script to `FILE.waxc` beside it, or several, as a bundle, to `-o`.
fn build_command(paths: &[String]) -> ExitCode {
    let mut paths = paths.to_vec();
    let secret = match take_option(&mut paths, "--sign") {
        Ok(secret) => secret,
        Err(code) => return code,
    };
    let (scripts, out): (Vec<&String>, Option<&String>) = match paths.iter().position(|path| path == "-o") {
        Some(at) => (paths.iter().take(at).collect(), paths.get(at + 1)),
        None => (paths.iter().collect(), None),
    };
    let out = match (out, scripts.as_slice()) {
        (Some(out), _) => out.clone(),
        (None, [only]) => Path::new(only.as_str()).with_extension("waxc").display().to_string(),
        _ => {
            eprintln!("wax: several scripts build into one bundle, named with `-o OUT.waxc`\n{USAGE}");
            return ExitCode::from(6);
        }
    };
    let mut sources = Vec::with_capacity(scripts.len());
    for path in &scripts {
        let Ok(source) = std::fs::read_to_string(path.as_str()) else {
            eprintln!("wax: `{path}` can't be read");
            return ExitCode::from(2);
        };
        sources.push(source);
    }
    let output = Arc::new(Mutex::new(Output::default()));
    let doors = match test_doors(&output, Vec::new()) {
        Ok(doors) => doors,
        Err(message) => {
            eprint!("{message}");
            return ExitCode::from(7);
        }
    };
    let named: Vec<(&str, &str)> =
        scripts.iter().map(|path| path.as_str()).zip(sources.iter().map(String::as_str)).collect();
    let built = wax::build(&named, &doors).and_then(|bytes| match &secret {
        // trace: D169. Signed with the developer's secret key, when one is given.
        Some(secret) => {
            let key = std::fs::read(secret).unwrap_or_default();
            let random = randomness().unwrap_or_default();
            wax::host::signing::sign(&bytes, &key, &random)
                .map_err(|message| Error::Refused(wax::Messages::one(message)))
        }
        None => Ok(bytes),
    });
    match built {
        Ok(bytes) => match std::fs::write(&out, bytes) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("wax: `{out}` can't be written: {error}");
                ExitCode::from(2)
            }
        },
        Err(error) => report(&error, &output),
    }
}

/// Whether a file is a compiled script, by its extension (D153).
fn is_compiled(path: &str) -> bool {
    Path::new(path).extension().is_some_and(|extension| extension.eq_ignore_ascii_case("waxc"))
}

/// trace: D169. Loads only a file signed with the secret key matching the public key in `key`.
fn require_key(doors: &mut Doors, key: &str) -> Result<(), ExitCode> {
    let Ok(public) = std::fs::read(key) else {
        eprintln!("wax: `{key}` can't be read");
        return Err(ExitCode::from(2));
    };
    doors.require_signature(&public).map_err(|message| {
        eprint!("{}", message.text());
        ExitCode::from(7)
    })
}

fn main() -> ExitCode {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("build") && arguments.len() > 1 {
        return build_command(arguments.get(1..).unwrap_or(&[]));
    }
    if let [command, name] = arguments.as_slice()
        && command == "keygen"
    {
        return keygen_command(name);
    }
    let key = match take_option(&mut arguments, "--key") {
        Ok(key) => key,
        Err(code) => return code,
    };
    let (command, path, wanted) = match arguments.as_slice() {
        [command, path] => (command, path, None),
        [command, path, name] if command == "run" && is_compiled(path) => (command, path, Some(name)),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(6);
        }
    };
    if command != "run" && command != "check" {
        eprintln!("wax: `{command}` isn't a command\n{USAGE}");
        return ExitCode::from(6);
    }
    let compiled = is_compiled(path);
    let (source, bytes) = if compiled {
        let Ok(bytes) = std::fs::read(path) else {
            eprintln!("wax: `{path}` can't be read");
            return ExitCode::from(2);
        };
        (String::new(), bytes)
    } else {
        let Ok(source) = std::fs::read_to_string(path) else {
            eprintln!("wax: `{path}` can't be read");
            return ExitCode::from(2);
        };
        (source, Vec::new())
    };
    let folder = Path::new(path).parent().unwrap_or(Path::new("."));
    let random: Vec<i64> = std::fs::read_to_string(folder.join("random.txt"))
        .map(|text| text.lines().filter_map(|line| line.trim().parse().ok()).collect())
        .unwrap_or_default();
    let output = Arc::new(Mutex::new(Output::default()));
    let mut doors = match test_doors(&output, random) {
        Ok(doors) => doors,
        Err(message) => {
            eprint!("{message}");
            return ExitCode::from(7);
        }
    };
    if let Some(key) = key
        && let Err(code) = require_key(&mut doors, &key)
    {
        return code;
    }
    if command == "check" && !compiled {
        return match Script::check(path, &source, &doors) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => report(&error, &output),
        };
    }
    let loaded = if compiled {
        // A file with one script runs it; a bundle runs the one named.
        let name = match (wanted, wax::names(&bytes, &doors)) {
            (Some(name), _) => name.clone(),
            (None, Ok(names)) if names.len() == 1 => names.into_iter().next().unwrap_or_default(),
            (None, Ok(names)) => {
                eprintln!("wax: `{path}` holds {} scripts; say which: {}", names.len(), names.join(", "));
                return ExitCode::from(6);
            }
            (None, Err(error)) => return report(&error, &output),
        };
        Script::load_compiled(&bytes, &name, &doors, LIMITS)
    } else {
        Script::load(path, &source, &doors, LIMITS)
    };
    let mut script = match loaded {
        Ok(script) => script,
        Err(error) => return report(&error, &output),
    };
    let calls = std::fs::read_to_string(folder.join("calls.txt")).unwrap_or_default();
    for line in calls.lines().filter(|line| !line.trim().is_empty()) {
        let Some((name, arguments)) = read_call(line) else {
            with(&output, Output::close_line);
            eprintln!("wax: `{line}` in calls.txt isn't a call with plain values");
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

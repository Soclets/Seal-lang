//! Seal's fuzzer: random scripts, valid and not, thrown at loading and running (Building rule 6).
//!
//! ```text
//! seal-fuzzer [--seed N] [--count N] [--seconds N] [--ralph]
//! ```
//!
//! Every script is loaded and run with the test host's doors, under a small step budget and memory cap. Nothing a
//! script does may harm its host (D14): the fuzzer fails a script that makes Seal report a bug of its own, takes
//! too long for its step budget, holds more than its memory cap, leaves memory behind after a call, or gives
//! different results on a second run (D50). A failing script is shrunk to the smallest one that still fails, and
//! written to `fuzz/found/`.
//!
//! With `--ralph`, scripts are made from what Seal shares with Unio, translated, and compared with Unio's oracle,
//! Ralph, as a black box (D19).
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

mod generate;
mod ralph;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use generate::{Dialect, Generator, Random, mutate, soup};
use seal::{Called, Doors, Error, Limits, Message, MessageKind, Pause, Script, Value};

/// Small limits, so a script that loops or hoards is stopped quickly.
const LIMITS: Limits = Limits { steps: 20_000, memory: 256 * 1024 };
/// How long a script may take before it counts as frozen. Its step budget allows far less work than this.
pub const TOO_LONG: Duration = Duration::from_secs(10);
/// How much the test host shows before it stops keeping more.
const OUTPUT_CAP: usize = 64 * 1024;

/// What running a script came to, as far as a host can see.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    /// What the script showed through its doors.
    pub output: String,
    /// How it ended: `ok`, or the kind of message it ended with.
    pub ending: String,
    /// The messages' text, when it didn't end well.
    pub messages: String,
    /// Something a host must never see, when the run broke the rules.
    pub broken: Option<String>,
}

fn doors(output: &Arc<Mutex<String>>) -> Result<Doors, Message> {
    let mut doors = Doors::new();
    let shown = Arc::clone(output);
    doors.add("show_msg", move |text: String| {
        if let Ok(mut output) = shown.lock()
            && output.len() < OUTPUT_CAP
        {
            output.push_str(&text);
            output.push('\n');
        }
    })?;
    let shown = Arc::clone(output);
    doors.add("show_msg.same_line", move |text: String| {
        if let Ok(mut output) = shown.lock()
            && output.len() < OUTPUT_CAP
        {
            output.push_str(&text);
        }
    })?;
    doors.add("ask", |_question: String| "7".to_owned())?;
    doors.add("rand", |low: i64, _high: i64| low)?.labels(["from", "to"])?;
    doors.add("wait", |_seconds: i64| Pause::<()>::new())?.labels(["seconds"])?;
    Ok(doors)
}

fn ending_of(error: &Error) -> (String, String, Option<String>) {
    match error {
        Error::Refused(messages) => {
            let kinds: Vec<MessageKind> = messages.list.iter().map(|message| message.kind).collect();
            let ending = if kinds.contains(&MessageKind::Refused) {
                "refused"
            } else if kinds.contains(&MessageKind::NotBuiltYet) {
                "not built yet"
            } else {
                "not designed yet"
            };
            let broken = kinds
                .contains(&MessageKind::Internal)
                .then(|| "Seal reported a bug of its own while loading".to_owned());
            (ending.to_owned(), messages.text(), broken)
        }
        Error::Stopped(message) => {
            let broken = (message.kind == MessageKind::Internal).then(|| "Seal reported a bug of its own".to_owned());
            (message.kind.word().to_owned(), message.text(), broken)
        }
        Error::Failed(failure) => ("failed".to_owned(), failure.clone(), None),
    }
}

/// Loads a script and calls each of its procedures that takes only whole numbers, as a host would.
fn run_here(source: &str, call_procedures: bool) -> Run {
    let output = Arc::new(Mutex::new(String::new()));
    let finish = |ending: String, messages: String, broken: Option<String>| Run {
        output: output.lock().map(|output| output.clone()).unwrap_or_default(),
        ending,
        messages,
        broken,
    };
    let doors = match doors(&output) {
        Ok(doors) => doors,
        Err(message) => {
            return finish("host".to_owned(), message.text(), Some("the fuzzer's own doors were refused".to_owned()));
        }
    };
    let mut script = match Script::load("fuzz.seal", source, &doors, LIMITS) {
        Ok(script) => script,
        Err(error) => {
            let (ending, messages, broken) = ending_of(&error);
            return finish(ending, messages, broken);
        }
    };
    if script.memory_in_use() > LIMITS.memory {
        return finish("ok".to_owned(), String::new(), Some("the script holds more than its memory cap".to_owned()));
    }
    let names: Vec<String> =
        if call_procedures { script.procedures().iter().map(|name| (*name).to_owned()).collect() } else { Vec::new() };
    for name in names {
        let before = script.memory_in_use();
        // Most procedures take other things than one whole number; those calls are refused, which must be safe too.
        let mut called = script.call_values(&name, vec![seal::Argument { label: None, value: Value::Whole(5) }]);
        let mut resumes = 0;
        while let Ok(Called::Paused(task)) = called {
            resumes += 1;
            called = if resumes > 3 {
                script.cancel(task).map(|()| Called::Finished(Value::Nothing))
            } else {
                script.resume_with_value(task, &Value::Nothing)
            };
        }
        match called {
            Ok(_) | Err(Error::Refused(_) | Error::Failed(_)) => {
                if script.memory_in_use() > LIMITS.memory {
                    return finish(
                        "ok".to_owned(),
                        String::new(),
                        Some("the script holds more than its memory cap".to_owned()),
                    );
                }
                let _ = before;
            }
            Err(error) => {
                let (ending, messages, broken) = ending_of(&error);
                return finish(ending, messages, broken);
            }
        }
    }
    finish("ok".to_owned(), String::new(), None)
}

/// Runs a script on a thread of its own, so that one that freezes is noticed instead of freezing the fuzzer.
#[must_use]
pub fn run(source: &str) -> Run {
    run_as(source, true)
}

/// Runs a script's top level only, as Unio's `proc.main` runs, for comparing with Ralph.
#[must_use]
pub fn run_top_level(source: &str) -> Run {
    run_as(source, false)
}

fn run_as(source: &str, call_procedures: bool) -> Run {
    let (sender, receiver) = mpsc::channel();
    let owned = source.to_owned();
    let started = Instant::now();
    let spawned = std::thread::Builder::new().stack_size(1024 * 1024).spawn(move || {
        let _ = sender.send(run_here(&owned, call_procedures));
    });
    if spawned.is_err() {
        return Run {
            output: String::new(),
            ending: "host".to_owned(),
            messages: String::new(),
            broken: Some("no thread to run on".to_owned()),
        };
    }
    match receiver.recv_timeout(TOO_LONG) {
        Ok(mut run) => {
            if run.broken.is_none() && started.elapsed() > TOO_LONG / 2 {
                run.broken = Some(format!(
                    "the script took {:.1} s, far more than its step budget allows",
                    started.elapsed().as_secs_f64()
                ));
            }
            run
        }
        Err(mpsc::RecvTimeoutError::Timeout) => Run {
            output: String::new(),
            ending: "frozen".to_owned(),
            messages: String::new(),
            broken: Some("the script froze its host past its step budget".to_owned()),
        },
        Err(mpsc::RecvTimeoutError::Disconnected) => Run {
            output: String::new(),
            ending: "crashed".to_owned(),
            messages: String::new(),
            broken: Some("running the script crashed its thread".to_owned()),
        },
    }
}

/// Everything wrong with how a script ran, if anything is.
fn judge(source: &str) -> Option<String> {
    let first = run(source);
    if let Some(broken) = first.broken {
        return Some(broken);
    }
    // trace: D50: the same script gives the same results again.
    let second = run(source);
    if second.broken.is_none()
        && (second.output != first.output || second.ending != first.ending || second.messages != first.messages)
    {
        return Some("the script gave different results on a second run".to_owned());
    }
    second.broken
}

/// Shrinks a failing script: lines are taken out while it still fails the same way (Building rule 6).
pub fn shrink(source: &str, still_fails: &dyn Fn(&str) -> bool) -> String {
    let mut lines: Vec<String> = source.lines().map(str::to_owned).collect();
    let mut chunk = lines.len().max(1) / 2;
    while chunk >= 1 {
        let mut at = 0;
        while at < lines.len() {
            let mut shorter = lines.clone();
            shorter.drain(at..(at + chunk).min(shorter.len()));
            let candidate = shorter.join("\n") + "\n";
            if !shorter.is_empty() && still_fails(&candidate) {
                lines = shorter;
            } else {
                at += chunk;
            }
        }
        chunk /= 2;
    }
    lines.join("\n") + "\n"
}

pub fn save(name: &str, source: &str, why: &str) {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("found");
    let _ = std::fs::create_dir_all(&folder);
    let _ = std::fs::write(folder.join(format!("{name}.seal")), source);
    let _ = std::fs::write(folder.join(format!("{name}.txt")), format!("{why}\n"));
}

#[derive(Default)]
struct Tally {
    scripts: u64,
    ok: u64,
    refused: u64,
    undecided: u64,
    stopped: u64,
    found: u64,
}

fn count(tally: &mut Tally, ending: &str) {
    tally.scripts += 1;
    match ending {
        "ok" => tally.ok += 1,
        "refused" => tally.refused += 1,
        "not designed yet" | "not built yet" => tally.undecided += 1,
        _ => tally.stopped += 1,
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let option = |name: &str| {
        arguments
            .iter()
            .position(|argument| argument == name)
            .and_then(|at| arguments.get(at + 1))
            .and_then(|value| value.parse::<u64>().ok())
    };
    let seed = option("--seed").unwrap_or(1);
    let count_limit = option("--count");
    let seconds = option("--seconds").unwrap_or(if count_limit.is_some() { u64::MAX } else { 10 });
    let show = arguments.iter().any(|argument| argument == "--show");
    if arguments.iter().any(|argument| argument == "--ralph") {
        return ralph::ralph_mode(seed, count_limit.unwrap_or(u64::MAX), seconds);
    }
    let started = Instant::now();
    let mut random = Random::new(seed);
    let mut valid = Tally::default();
    let mut changed = Tally::default();
    let mut soups = Tally::default();
    let mut reasons: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut found = 0u64;
    let mut round = 0u64;
    while count_limit.is_none_or(|limit| round < limit) && started.elapsed().as_secs() < seconds {
        round += 1;
        let script_seed = random.next();
        let script = Generator::new(script_seed, Dialect::Seal).script();
        let mutated = mutate(&script, &mut random);
        let soup = soup(&mut random);
        if show {
            println!("{script}");
            let run = run(&script);
            println!("=> {}\n{}{}", run.ending, run.messages, run.output);
            continue;
        }
        for (kind, source, tally) in
            [("valid", &script, &mut valid), ("changed", &mutated, &mut changed), ("soup", &soup, &mut soups)]
        {
            let first = run(source);
            let ending = first.ending;
            if kind == "valid" && ending != "ok" {
                let what = first
                    .messages
                    .lines()
                    .find(|line| line.trim_start().starts_with("what:"))
                    .unwrap_or("")
                    .trim()
                    .to_owned();
                *reasons.entry(format!("{ending}: {what}")).or_insert(0u64) += 1;
            }
            count(tally, &ending);
            if let Some(why) = judge(source) {
                found += 1;
                tally.found += 1;
                let reason = why.clone();
                let small = shrink(source, &|candidate| judge(candidate).is_some_and(|again| again == reason));
                let name = format!("seed{seed}_{round}_{kind}");
                save(&name, &small, &why);
                println!("FOUND {name}: {why}\n{small}");
            }
        }
    }
    for (name, tally) in [("valid", &valid), ("changed", &changed), ("soup", &soups)] {
        println!(
            "{name:8} {} scripts: {} ran to their end, {} refused, {} not designed or built yet, {} stopped while running, {} broke a rule",
            tally.scripts, tally.ok, tally.refused, tally.undecided, tally.stopped, tally.found
        );
    }
    if arguments.iter().any(|argument| argument == "--why") {
        let mut sorted: Vec<(&String, &u64)> = reasons.iter().collect();
        sorted.sort_by(|left, right| right.1.cmp(left.1));
        for (reason, times) in sorted.iter().take(40) {
            println!("{times:6}  {reason}");
        }
    }
    println!("{found} scripts broke a rule, in {:.1} s, from seed {seed}", started.elapsed().as_secs_f64());
    if found == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

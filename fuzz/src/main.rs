//! Wax's fuzzer: random scripts, valid and not, thrown at loading and running (Building rule 6).
//!
//! ```text
//! wax-fuzzer [--seed N] [--count N] [--seconds N] [--ralph]
//! ```
//!
//! Every script is loaded and run with the test host's doors, under a small step budget and memory cap. Nothing a
//! script does may harm its host (D14): the fuzzer fails a script that makes Wax report a bug of its own, takes
//! too long for its step budget, holds more than its memory cap, leaves memory behind after a call, or gives
//! different results on a second run (D50). A failing script is shrunk to the smallest one that still fails, and
//! written to `fuzz/found/`.
//!
//! With `--ralph`, scripts are made from what Wax shares with Unio, translated, and compared with Unio's oracle,
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
use wax::{Called, Doors, Error, Limits, Message, MessageKind, Pause, Script, Value};

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
                .then(|| "Wax reported a bug of its own while loading".to_owned());
            (ending.to_owned(), messages.text(), broken)
        }
        Error::Stopped(message) => {
            let broken = (message.kind == MessageKind::Internal).then(|| "Wax reported a bug of its own".to_owned());
            (message.kind.word().to_owned(), message.text(), broken)
        }
        Error::Failed(failure) => ("failed".to_owned(), failure.clone(), None),
    }
}

/// How a script is loaded.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Loading {
    /// From its text, as a host usually does.
    Text,
    /// From its text, compiled with none of the compiler's shortcuts.
    Plainly,
    /// Built into a `.waxc` file and loaded from that (D141).
    Compiled,
    /// Built into a `.waxc` file that's then changed, with its checksum made to match: refused, or run without
    /// harming the host (D141, Building rule 1). The number picks the changes.
    Tampered(u64),
}

/// A `.waxc` file's bytes changed at a few places, then its checksum made to match, as someone changing it on
/// purpose would.
fn tamper(bytes: &mut [u8], seed: u64) {
    let mut random = Random::new(seed);
    let body = bytes.len().saturating_sub(8);
    // Past the header, which holds Wax's fingerprint and the doors' key, so the change reaches the script itself.
    let start = 32.min(body);
    for _ in 0..=random.below(3) {
        let at = start + random.below((body - start).max(1));
        if let Some(byte) = bytes.get_mut(at) {
            *byte = u8::try_from(random.below(256)).unwrap_or(0);
        }
    }
    let checksum = wax::compile::file::hash(bytes.get(..body).unwrap_or(&[]));
    if let Some(end) = bytes.get_mut(body..) {
        end.copy_from_slice(&checksum.to_le_bytes());
    }
}

/// trace: D141. A built script's file with a byte changed and its checksum left as it was is always refused.
fn damage_is_refused(source: &str, seed: u64) -> Option<String> {
    let output = Arc::new(Mutex::new(String::new()));
    let doors = doors(&output).ok()?;
    let mut bytes = wax::build(&[("fuzz.wax", source)], &doors).ok()?;
    let mut random = Random::new(seed);
    let length = bytes.len();
    if random.chance(1, 4) {
        // Cut short.
        bytes.truncate(random.below(length));
    } else {
        let at = random.below(length);
        if let Some(byte) = bytes.get_mut(at) {
            *byte ^= u8::try_from(1 + random.below(255)).unwrap_or(1);
        }
    }
    match Script::load_compiled(&bytes, "fuzz.wax", &doors, LIMITS) {
        Err(Error::Refused(_)) => None,
        Ok(_) => Some("a damaged `.waxc` file loaded".to_owned()),
        Err(_) => Some("a damaged `.waxc` file stopped instead of being refused".to_owned()),
    }
}

/// trace: D169. A signed file loads, the same as unsigned, where its key is required; changed anywhere, even with its
/// checksum remade, or signed with another key, it's always refused.
fn signed_change_is_refused(source: &str, seed: u64) -> Option<String> {
    static KEYS: std::sync::OnceLock<[(Vec<u8>, Vec<u8>); 2]> = std::sync::OnceLock::new();
    let [(public, secret), (_, other_secret)] = KEYS
        .get_or_init(|| [wax::host::signing::keys_from_seed(&[42; 32]), wax::host::signing::keys_from_seed(&[43; 32])]);
    let output = Arc::new(Mutex::new(String::new()));
    let plain_doors = doors(&output).ok()?;
    let built = wax::build(&[("fuzz.wax", source)], &plain_doors).ok()?;
    let mut doors = plain_doors.clone();
    doors.require_signature(public).ok()?;
    let randomness = [u8::try_from(seed % 256).unwrap_or(0); 32];
    let signed = wax::host::signing::sign(&built, secret, &randomness).ok()?;
    let unsigned_loads = Script::load_compiled(&built, "fuzz.wax", &plain_doors, LIMITS).is_ok();
    if Script::load_compiled(&signed, "fuzz.wax", &doors, LIMITS).is_ok() != unsigned_loads {
        return Some("a signed `.waxc` file loaded differently from the same file unsigned".to_owned());
    }
    let mut changed = built.clone();
    tamper(&mut changed, seed);
    let mut forged = changed.clone();
    forged.extend_from_slice(signed.get(built.len()..).unwrap_or(&[]));
    let refused =
        |bytes: &[u8]| matches!(Script::load_compiled(bytes, "fuzz.wax", &doors, LIMITS), Err(Error::Refused(_)));
    if changed != built && !refused(&forged) {
        return Some("a signed `.waxc` file changed after signing wasn't refused".to_owned());
    }
    let elsewhere = wax::host::signing::sign(&built, other_secret, &randomness).ok()?;
    if !refused(&elsewhere) || !refused(&built) {
        return Some("a `.waxc` file not signed with the host's key wasn't refused".to_owned());
    }
    None
}

/// Loads a script the way asked. Compiled plainly, it must give the same answers as with the compiler's shortcuts;
/// from a `.waxc` file, exactly the same as from its text.
fn load(source: &str, doors: &Doors, how: Loading) -> Result<Script, Error> {
    match how {
        Loading::Text => Script::load("fuzz.wax", source, doors, LIMITS),
        Loading::Plainly => Script::load_plainly("fuzz.wax", source, doors, LIMITS),
        Loading::Compiled | Loading::Tampered(_) => {
            let mut bytes = wax::build(&[("fuzz.wax", source)], doors)?;
            if let Loading::Tampered(seed) = how {
                tamper(&mut bytes, seed);
            }
            Script::load_compiled(&bytes, "fuzz.wax", doors, LIMITS)
        }
    }
}

/// Loads a script and calls each of its procedures that takes only whole numbers, as a host would.
fn run_here(source: &str, call_procedures: bool, strict_memory: bool, how: Loading) -> Run {
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
    let loaded = load(source, &doors, how);
    // A tampered file that loads may mean anything; all that holds is that the host comes to no harm: no crash, no
    // freeze, nothing past the memory cap. Its own bookkeeping, and Wax's messages about it, aren't judged.
    let tampered = matches!(how, Loading::Tampered(_));
    let mut script = match loaded {
        Ok(script) => script,
        Err(error) => {
            let (ending, messages, broken) = ending_of(&error);
            return finish(ending, messages, broken.filter(|_| !tampered));
        }
    };
    if script.memory_in_use() > LIMITS.memory {
        return finish("ok".to_owned(), String::new(), Some("the script holds more than its memory cap".to_owned()));
    }
    let audit = |script: &Script| if tampered { Ok(()) } else { script.audit() };
    if let Err(why) = audit(&script) {
        return finish(
            "ok".to_owned(),
            String::new(),
            Some(format!("after loading, the holder counts are off: {why}")),
        );
    }
    let names: Vec<String> =
        if call_procedures { script.procedures().iter().map(|name| (*name).to_owned()).collect() } else { Vec::new() };
    for name in names {
        let before = script.memory_in_use();
        // Most procedures take other things than one whole number; those calls are refused, which must be safe too.
        let mut called = script.call_values(&name, &[wax::Argument { label: None, value: Value::Whole(5) }]);
        if matches!(called, Err(wax::Error::Refused(_))) {
            // A procedure that takes nothing is called with nothing (D60).
            called = script.call_values(&name, &[]);
        }
        let mut resumes = 0;
        while let Ok(Called::Paused(task)) = called {
            if let Err(why) = audit(&script) {
                return finish(
                    "ok".to_owned(),
                    String::new(),
                    Some(format!("while `{name}` is paused, the holder counts are off: {why}")),
                );
            }
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
                if let Err(why) = audit(&script) {
                    return finish(
                        "ok".to_owned(),
                        String::new(),
                        Some(format!("after a call to `{name}`, the holder counts are off: {why}")),
                    );
                }
                // A generated script's procedures keep nothing once they're done, so a call that ran to its end,
                // failed or was cancelled must leave the script holding exactly what it held before (D15).
                let after = script.memory_in_use();
                if strict_memory && !tampered && after != before {
                    return finish(
                        "ok".to_owned(),
                        String::new(),
                        Some(format!(
                            "a call to `{name}` left the script holding {after} bytes, where it held {before} before"
                        )),
                    );
                }
            }
            Err(error) => {
                let (ending, messages, broken) = ending_of(&error);
                return finish(ending, messages, broken.filter(|_| !tampered));
            }
        }
    }
    finish("ok".to_owned(), String::new(), None)
}

/// Runs a script on a thread of its own, so that one that freezes is noticed instead of freezing the fuzzer.
#[must_use]
pub fn run(source: &str) -> Run {
    run_as(source, true, false, Loading::Text)
}

/// Runs a generated script, whose procedures must leave no memory behind.
fn run_strictly(source: &str) -> Run {
    run_as(source, true, true, Loading::Text)
}

/// Runs a script's top level only, as Unio's `proc.main` runs, for comparing with Ralph.
#[must_use]
pub fn run_top_level(source: &str) -> Run {
    run_as(source, false, false, Loading::Text)
}

fn run_as(source: &str, call_procedures: bool, strict_memory: bool, how: Loading) -> Run {
    let (sender, receiver) = mpsc::channel();
    let owned = source.to_owned();
    let started = Instant::now();
    let spawned = std::thread::Builder::new().stack_size(1024 * 1024).spawn(move || {
        let _ = sender.send(run_here(&owned, call_procedures, strict_memory, how));
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
fn judge(source: &str, generated: bool) -> Option<String> {
    let first = if generated { run_strictly(source) } else { run(source) };
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
    if second.broken.is_some() {
        return second.broken;
    }
    // Compiled plainly, with none of the compiler's shortcuts, it gives the same results too. A script that runs
    // into a limit is left out: the two ways count their steps and their memory a little differently.
    let plain = run_as(source, true, false, Loading::Plainly);
    let limited = |run: &Run| run.ending == "limit reached";
    if plain.broken.is_none()
        && !limited(&second)
        && !limited(&plain)
        && (plain.output != second.output || plain.ending != second.ending || plain.messages != second.messages)
    {
        return Some(format!(
            "compiled plainly, the script gave different results:\n--- with shortcuts ({})\n{}{}\n--- plainly ({})\n{}{}",
            second.ending, second.output, second.messages, plain.ending, plain.output, plain.messages
        ));
    }
    if plain.broken.is_some() {
        return plain.broken;
    }
    // trace: D141. Built into a `.waxc` file and loaded from that, it gives exactly the same, limits and all.
    let compiled = run_as(source, true, false, Loading::Compiled);
    if compiled.broken.is_none()
        && (compiled.output != second.output
            || compiled.ending != second.ending
            || compiled.messages != second.messages)
    {
        return Some(format!(
            "loaded from a `.waxc` file, the script gave different results:\n--- from its text ({})\n{}{}\n--- compiled ({})\n{}{}",
            second.ending, second.output, second.messages, compiled.ending, compiled.output, compiled.messages
        ));
    }
    if compiled.broken.is_some() {
        return compiled.broken;
    }
    let seed = wax::compile::file::hash(source.as_bytes());
    if let Some(damaged) = damage_is_refused(source, seed) {
        return Some(damaged);
    }
    // A signature costs far more than a load, so one script in four is signed.
    if seed.is_multiple_of(4)
        && let Some(signed) = signed_change_is_refused(source, seed)
    {
        return Some(signed);
    }
    run_as(source, true, false, Loading::Tampered(seed)).broken
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
    let _ = std::fs::write(folder.join(format!("{name}.wax")), source);
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
    // One script from a file, through every check a fuzzed one goes through.
    if let Some(path) = arguments.iter().position(|argument| argument == "--file").and_then(|at| arguments.get(at + 1))
    {
        let Ok(source) = std::fs::read_to_string(path) else {
            eprintln!("couldn't read {path}");
            return ExitCode::FAILURE;
        };
        let checked = run(&source);
        println!("=> {}\n{}{}", checked.ending, checked.messages, checked.output);
        return match checked.broken {
            Some(why) => {
                println!("BROKE A RULE: {why}");
                ExitCode::FAILURE
            }
            None => ExitCode::SUCCESS,
        };
    }
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
        let script = Generator::new(script_seed, Dialect::Wax).script();
        let mutated = mutate(&script, &mut random);
        let soup = soup(&mut random);
        if show {
            // Every script and how it went, so two builds' runs can be compared line by line.
            for (kind, source) in [("valid", &script), ("changed", &mutated), ("soup", &soup)] {
                println!("-- {kind}\n{source}");
                let run = run(source);
                println!("=> {}\n{}{}", run.ending, run.messages, run.output);
            }
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
            let generated = kind == "valid";
            if let Some(why) = judge(source, generated) {
                found += 1;
                tally.found += 1;
                let reason = why.clone();
                let kind_of = |why: &str| why.split(" left ").next().unwrap_or("").to_owned();
                let small = shrink(source, &|candidate| {
                    judge(candidate, generated).is_some_and(|again| kind_of(&again) == kind_of(&reason))
                });
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

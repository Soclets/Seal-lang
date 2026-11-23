//! Seal's benchmark host: runs the benchmarks of D57 by the rules of D58, and writes `bench/RESULTS.md`.
//!
//! ```text
//! seal-bench report [--runs N] [--variant LABEL=PATH]...   run everything and write bench/RESULTS.md
//! seal-bench seal FILE ARGUMENT WARMUPS RUNS                one benchmark on Seal, as the report runs it
//! seal-bench patrol COUNT ROUNDS                            the patrolling-scripts benchmark on Seal
//! seal-bench load FILE                                      load a script and stop
//! seal-bench lines COUNT                                    load a generated script of about COUNT lines
//! ```
//!
//! Every result is checked for correctness before it's timed (D58): a wrong answer is reported as wrong, never as a
//! time. Each program runs in a process of its own, so its peak memory is its own.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

mod report;

use std::fmt::Write as _;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use seal::{Argument, Called, Doors, Limits, Pause, Script, TaskId, Value};

/// Generous limits: the benchmarks measure speed, not the limits. Every check stays on (D17, D57).
const LIMITS: Limits = Limits { steps: u64::MAX, memory: 1 << 30 };

fn whole(value: i64) -> [Argument; 1] {
    [Argument { label: None, value: Value::Whole(value) }]
}

/// A result as every engine's driver prints it, so that results can be compared across engines.
fn shown(value: &Value) -> String {
    match value {
        Value::Whole(whole) => whole.to_string(),
        Value::Float(float) => float.to_string(),
        Value::Text(text) => {
            let ending: String = text.chars().rev().take(12).collect::<Vec<_>>().into_iter().rev().collect();
            format!("text {} {ending}", text.len())
        }
        other => format!("{other:?}"),
    }
}

/// Runs `run(argument)` of a script: once cold, then the warm-ups, then the timed runs.
fn seal(path: &str, argument: i64, warmups: usize, runs: usize) -> Result<String, String> {
    let source = std::fs::read_to_string(path).map_err(|_| format!("`{path}` can't be read"))?;
    let doors = Doors::new();
    let started = Instant::now();
    let mut script = Script::load(path, &source, &doors, LIMITS).map_err(|error| error.to_string())?;
    let load = started.elapsed().as_secs_f64();
    // The same text loaded again with the same doors: Seal keeps what it compiled, so this is the warm load (D58).
    let started = Instant::now();
    let again = Script::load(path, &source, &doors, LIMITS).map_err(|error| error.to_string())?;
    let load_warm = started.elapsed().as_secs_f64();
    drop(again);
    let call = |script: &mut Script| -> Result<(Value, f64), String> {
        let started = Instant::now();
        match script.call_values("run", &whole(argument)) {
            Ok(Called::Finished(value)) => Ok((value, started.elapsed().as_secs_f64())),
            Ok(Called::Paused(_)) => Err("the benchmark paused".to_owned()),
            Err(error) => Err(error.to_string()),
        }
    };
    let (mut result, first) = call(&mut script)?;
    for _ in 0..warmups {
        call(&mut script)?;
    }
    let mut times = Vec::new();
    for _ in 0..runs {
        let (value, time) = call(&mut script)?;
        result = value;
        times.push(format!("{time:.9}"));
    }
    let mut out = String::new();
    let _ = writeln!(out, "result {}", shown(&result));
    let _ = writeln!(out, "load {load:.9}");
    let _ = writeln!(out, "load_warm {load_warm:.9}");
    let _ = writeln!(out, "code {}", script.compiled_bytes());
    let _ = writeln!(out, "first {first:.9}");
    let _ = writeln!(out, "times {}", times.join(" "));
    let _ = writeln!(out, "memory {}", script.most_memory_used());
    Ok(out)
}

/// This process's resident memory now, in bytes, as the system reports it.
fn resident_bytes() -> u64 {
    std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()
        .and_then(|output| String::from_utf8_lossy(&output.stdout).trim().parse::<u64>().ok())
        .map_or(0, |kilobytes| kilobytes * 1024)
}

/// The patrolling-scripts benchmark (D57): `count` scripts loaded from the same text, each paused in its patrol,
/// then every one resumed `rounds` times.
fn patrol(path: &str, count: usize, rounds: usize) -> Result<String, String> {
    let source = std::fs::read_to_string(path).map_err(|_| format!("`{path}` can't be read"))?;
    let walks = Arc::new(AtomicU64::new(0));
    let mut doors = Doors::new();
    let counted = Arc::clone(&walks);
    doors
        .add("walk_to", move |_guard: i64, _x: i64| {
            counted.fetch_add(1, Ordering::Relaxed);
        })
        .and_then(|door| door.labels_after_first(["x"]))
        .map_err(|message| message.to_string())?;
    doors
        .add("wait", |_seconds: i64| Pause::<()>::new())
        .and_then(|door| door.labels(["seconds"]))
        .map_err(|message| message.to_string())?;
    let limits = Limits { steps: 1_000_000, memory: 1024 * 1024 };

    // Loading each script from a text of its own, so that nothing compiled is reused: a comment naming the guard
    // makes every text different. Timed apart, and those scripts dropped, before the run that's measured.
    let started = Instant::now();
    for guard in 0..count {
        let own_text = format!("// guard {guard}\n{source}");
        Script::load(path, &own_text, &doors, limits).map_err(|error| error.to_string())?;
    }
    let loaded_fresh = started.elapsed().as_secs_f64();

    let resident_before = resident_bytes();
    let started = Instant::now();
    let mut scripts: Vec<(Script, TaskId)> = Vec::with_capacity(count);
    for guard in 0..count {
        let mut script = Script::load(path, &source, &doors, limits).map_err(|error| error.to_string())?;
        match script.call_values("patrol", &whole(i64::try_from(guard).unwrap_or(0))) {
            Ok(Called::Paused(task)) => scripts.push((script, task)),
            Ok(Called::Finished(_)) => return Err("a patrol ended".to_owned()),
            Err(error) => return Err(error.to_string()),
        }
    }
    let loaded = started.elapsed().as_secs_f64();
    let resident_after = resident_bytes();
    let counted_memory: usize = scripts.iter().map(|(script, _)| script.memory_in_use()).sum();

    let started = Instant::now();
    for _ in 0..rounds {
        for (script, task) in &mut scripts {
            match script.resume_with_value(*task, &Value::Nothing) {
                Ok(Called::Paused(next)) => *task = next,
                Ok(Called::Finished(_)) => return Err("a patrol ended".to_owned()),
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    let resumed = started.elapsed().as_secs_f64();

    let started = Instant::now();
    for _ in 0..rounds {
        for (guard, (script, _)) in scripts.iter_mut().enumerate() {
            script.call_values("step", &whole(i64::try_from(guard).unwrap_or(0))).map_err(|error| error.to_string())?;
        }
    }
    let called = started.elapsed().as_secs_f64();

    let total = (count * rounds).max(1);
    #[allow(clippy::cast_precision_loss)] // counts of scripts and rounds, far below where a `f64` loses digits
    let (each, every) = (count.max(1) as f64, total as f64);
    let mut out = String::new();
    let _ = writeln!(out, "result {}", walks.load(Ordering::Relaxed));
    let _ = writeln!(out, "load_each {:.9}", loaded / each);
    let _ = writeln!(out, "load_fresh_each {:.9}", loaded_fresh / each);
    let _ = writeln!(out, "resume_each {:.9}", resumed / every);
    let _ = writeln!(out, "call_each {:.9}", called / every);
    let _ = writeln!(out, "memory_each {}", counted_memory / count.max(1));
    let _ = writeln!(out, "resident_each {}", resident_after.saturating_sub(resident_before) / (count.max(1) as u64));
    Ok(out)
}

/// A script of about `lines` lines, the same every time: procedures using maths, lists, loops, text and
/// conversions, for timing how long loading takes (D57's goal 4).
#[must_use]
pub fn big_script(lines: usize) -> String {
    const PER_PROCEDURE: usize = 21;
    let mut out = String::new();
    for index in 0..lines / PER_PROCEDURE {
        let _ = write!(
            out,
            "proc work_{index}[n: Whole, scale: Float] -> Whole
    mut total = n
    mut items = [n, n + 1, n + {index}]
    for each item at pos place in items
        if item rem.floor 2 == 0
            total = total + item * place
        else
            total = total - place
        end
    end
    mut level = scale
    mut count = 0
    while count < 3
        level = level * 1.5 + be.Float(count)
        count = count + 1
    end
    label = \"work\" & n & \":\" & total
    items.add(be.Whole.floor(level))
    return total + items.len + (be.Whole(label) ?? {index})
end

"
        );
    }
    out
}

/// Loads a generated script of about `lines` lines several times, giving the best and the mean time.
fn lines(count: usize) -> Result<String, String> {
    let source = big_script(count);
    let actual = source.lines().count();
    let mut times = Vec::new();
    for round in 0..10 {
        // A set of doors of its own each time, so that nothing is reused from the load before.
        let doors = Doors::new();
        let started = Instant::now();
        Script::load("big.seal", &source, &doors, LIMITS).map_err(|error| error.to_string())?;
        times.push(started.elapsed().as_secs_f64());
        let _ = round;
    }
    let best = times.iter().copied().fold(f64::INFINITY, f64::min);
    #[allow(clippy::cast_precision_loss)] // ten runs
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    Ok(format!("lines {actual}\nbytes {}\nbest {best:.6}\nmean {mean:.6}\n", source.len()))
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let text: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let number = |text: &str| text.parse::<usize>().unwrap_or(0);
    let outcome = match text.as_slice() {
        ["seal", path, argument, warmups, runs] => {
            seal(path, argument.parse().unwrap_or(0), number(warmups), number(runs))
        }
        ["patrol", path, count, rounds] => patrol(path, number(count), number(rounds)),
        ["load", path] => std::fs::read_to_string(path)
            .map_err(|_| format!("`{path}` can't be read"))
            .and_then(|source| {
                let started = Instant::now();
                Script::load(path, &source, &Doors::new(), LIMITS).map_err(|error| error.to_string())?;
                Ok(format!("load {:.9}\n", started.elapsed().as_secs_f64()))
            }),
        ["lines", count] => lines(number(count)),
        ["report", options @ ..] => report::report(options),
        _ => Err("usage: seal-bench report | seal FILE ARGUMENT WARMUPS RUNS | patrol FILE COUNT ROUNDS | load FILE | lines COUNT".to_owned()),
    };
    match outcome {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprint!("{why}");
            if !why.ends_with('\n') {
                eprintln!();
            }
            ExitCode::FAILURE
        }
    }
}

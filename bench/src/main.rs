//! Seal's benchmark host: runs the benchmark scripts and measures them (D57, D58).
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::process::ExitCode;
use std::time::Instant;

use seal::{Called, Doors, Limits, Script, Value, labelled};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [path, procedure, number] = arguments.as_slice() else {
        eprintln!("usage: seal-bench FILE.seal PROCEDURE NUMBER");
        return ExitCode::from(6);
    };
    let Ok(source) = std::fs::read_to_string(path) else {
        eprintln!("`{path}` can't be read");
        return ExitCode::from(2);
    };
    let _ = labelled::<i64>;
    let doors = Doors::new();
    let limits = Limits { steps: u64::MAX, memory: 1 << 30 };
    let started = Instant::now();
    let mut script = match Script::load(path, &source, &doors, limits) {
        Ok(script) => script,
        Err(error) => {
            eprint!("{error}");
            return ExitCode::from(1);
        }
    };
    let loaded = started.elapsed();
    let started = Instant::now();
    let result = script
        .call_values(procedure, vec![seal::Argument { label: None, value: Value::Whole(number.parse().unwrap_or(0)) }]);
    let ran = started.elapsed();
    match result {
        Ok(Called::Finished(value)) => println!("{value:?}"),
        Ok(Called::Paused(_)) => println!("paused"),
        Err(error) => eprint!("{error}"),
    }
    println!("load {:.6} s, run {:.6} s", loaded.as_secs_f64(), ran.as_secs_f64());
    ExitCode::SUCCESS
}

//! Comparing Wax with Unio's oracle, Ralph, as a black box (D19, Building rule 7): a script made of what the two
//! languages share is run on Wax's VM and, translated, on Ralph. A difference is a bug in one of them.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use crate::generate::{Dialect, Generator, Random};
use crate::{TOO_LONG, run_top_level, save, shrink};

/// Where Unio's oracle is on this machine. It's run, and never read.
const RALPH: &str = "/Users/ts/Plainly v2/oracle/_build/default/shell/ralph.exe";

fn ralph_path() -> String {
    std::env::var("RALPH").unwrap_or_else(|_| RALPH.to_owned())
}

/// What Ralph made of a Unio program: its output, its exit status, and what it said.
struct RalphRun {
    output: String,
    status: i32,
    messages: String,
}

impl RalphRun {
    fn unsupported(&self) -> bool {
        self.messages.contains("NotBuiltYet") || self.messages.contains("NotDesignedYet")
    }
}

/// Runs Ralph on a Unio program, alone in a folder of its own, since a Unio program is every file in its folder.
fn ralph(unio: &str, input: &str) -> Option<RalphRun> {
    let folder = std::env::temp_dir().join(format!("wax-ralph-{}", std::process::id()));
    std::fs::create_dir_all(&folder).ok()?;
    std::fs::write(folder.join("program.unio"), unio).ok()?;
    let mut child = Command::new(ralph_path())
        .arg("program.unio")
        .current_dir(&folder)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() > TOO_LONG => {
                let _ = child.kill();
                let _ = child.wait();
                return Some(RalphRun {
                    output: String::new(),
                    status: -1,
                    messages: "Ralph took too long".to_owned(),
                });
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(1)),
            Err(_) => return None,
        }
    }
    let output = child.wait_with_output().ok()?;
    Some(RalphRun {
        output: String::from_utf8_lossy(&output.stdout).into_owned(),
        status: output.status.code().unwrap_or(-1),
        messages: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// How one script compared between Wax and Ralph.
#[derive(Debug, PartialEq)]
enum Compared {
    Same,
    /// Not comparable: why.
    Skipped(&'static str),
    Different(String),
}

/// Runs a script on Wax and, translated, on Ralph, and compares what a player would see.
fn compare(source: &str) -> Compared {
    let wax_run = run_top_level(source);
    if wax_run.broken.is_some() {
        return Compared::Different("Wax broke a rule of its own".to_owned());
    }
    let wanted_status = match wax_run.ending.as_str() {
        "ok" => 0,
        "check failed" => 4,
        "refused" | "not designed yet" | "not built yet" => return Compared::Skipped("Wax doesn't load it"),
        _ => return Compared::Skipped("Wax stops it at a limit, which Unio doesn't have"),
    };
    let Ok(unio) = wax_to_unio::translate(source, &wax_to_unio::test_host_doors()) else {
        return Compared::Skipped("it uses something Unio doesn't have");
    };
    // The fuzzer's `ask` door always answers 7.
    let Some(ralph_run) = ralph(&unio, &"7\n".repeat(64)) else {
        return Compared::Skipped("Ralph couldn't be run");
    };
    if ralph_run.unsupported() {
        return Compared::Skipped("Ralph doesn't support it yet");
    }
    if ralph_run.status != wanted_status {
        return Compared::Different(format!(
            "Wax ends with `{}`, and Ralph with status {}\n--- Wax says\n{}--- Ralph says\n{}--- the Unio program\n{unio}",
            wax_run.ending, ralph_run.status, wax_run.messages, ralph_run.messages
        ));
    }
    if ralph_run.output != wax_run.output {
        return Compared::Different(format!(
            "the output differs\n--- Wax shows\n{}--- Ralph shows\n{}--- the Unio program\n{unio}",
            wax_run.output, ralph_run.output
        ));
    }
    Compared::Same
}

/// The kind of difference, without its details, so that shrinking keeps to one difference.
fn difference_kind(compared: &Compared) -> Option<String> {
    match compared {
        Compared::Different(why) => Some(why.lines().next().unwrap_or("").to_owned()),
        _ => None,
    }
}

/// Each test in `tests/` whose script has a Unio twin: Ralph must give the test's own expected output.
fn compare_tests() -> (u64, u64) {
    let (mut same, mut different) = (0, 0);
    let tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut folders: Vec<PathBuf> = std::fs::read_dir(&tests)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();
    folders.sort();
    for folder in folders {
        let Ok(source) = std::fs::read_to_string(folder.join("program.wax")) else { continue };
        let name = folder.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let unio = match wax_to_unio::translate(&source, &wax_to_unio::test_host_doors()) {
            Ok(unio) => unio,
            Err(wax_to_unio::Untranslatable::NotShared(what)) => {
                println!("test {name}: not compared, it uses {what}");
                continue;
            }
            Err(wax_to_unio::Untranslatable::Refused(_)) => {
                println!("test {name}: not compared, Wax refuses it");
                continue;
            }
        };
        let input = std::fs::read_to_string(folder.join("input.txt")).unwrap_or_default();
        let expected = std::fs::read_to_string(folder.join("expected.out")).unwrap_or_default();
        match ralph(&unio, &input) {
            Some(run) if run.unsupported() => println!("test {name}: not compared, Ralph doesn't support it yet"),
            Some(run) if run.status == 0 && run.output == expected => {
                same += 1;
                println!("test {name}: Ralph gives the expected output");
            }
            Some(run) => {
                different += 1;
                println!(
                    "test {name}: DIFFERENT, Ralph ends with status {} and shows\n{}{}",
                    run.status, run.output, run.messages
                );
            }
            None => println!("test {name}: Ralph couldn't be run"),
        }
    }
    (same, different)
}

/// Compares Wax with Ralph on the tests in `tests/`, and on random scripts made of what the two share.
pub fn ralph_mode(seed: u64, rounds: u64, seconds: u64) -> ExitCode {
    if !Path::new(&ralph_path()).exists() {
        println!("Ralph isn't on this machine (looked at `{}`), so the comparison is skipped", ralph_path());
        return ExitCode::SUCCESS;
    }
    let started = Instant::now();
    let mut random = Random::new(seed);
    let (mut same, mut different) = compare_tests();
    let mut skipped: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut round = 0;
    while round < rounds && started.elapsed().as_secs() < seconds {
        round += 1;
        let script = Generator::new(random.next(), Dialect::Ralph).script();
        match compare(&script) {
            Compared::Same => same += 1,
            Compared::Skipped(why) => *skipped.entry(why).or_insert(0) += 1,
            compared @ Compared::Different(_) => {
                different += 1;
                let kind = difference_kind(&compared);
                let small = shrink(&script, &|candidate| difference_kind(&compare(candidate)) == kind);
                let why = match compare(&small) {
                    Compared::Different(why) => why,
                    _ => kind.clone().unwrap_or_default(),
                };
                let name = format!("ralph_seed{seed}_{round}");
                save(&name, &small, &why);
                println!("DIFFERENT {name}: {why}\n--- the Wax script\n{small}");
            }
        }
    }
    println!(
        "compared with Ralph: {same} the same, {different} different, in {:.1} s, from seed {seed}",
        started.elapsed().as_secs_f64()
    );
    for (why, times) in skipped {
        println!("  {times} not compared: {why}");
    }
    if different == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

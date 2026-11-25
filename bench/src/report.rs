//! Runs every benchmark of D57 in every kind of D58, on Seal, Luau, `LuaJIT` and Node, and writes `bench/RESULTS.md`.
//!
//! Each program runs in a process of its own, under `/usr/bin/time -l`, so its peak memory is its own. A result is
//! checked before its time counts: every engine must give the same answer, and the Normal programs the answers
//! written here (D58).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

const LUAU: &str = "/Users/ts/.rokit/bin/luau";
const LUAJIT: &str = "/opt/homebrew/bin/luajit";
const NODE: &str = "/usr/local/bin/node";

/// One of D57's benchmarks, as run from a single script.
struct Benchmark {
    name: &'static str,
    title: &'static str,
    tests: &'static str,
    /// What `run` is given in the Normal and Ceil kinds, and the answer it must give.
    argument: i64,
    expected: &'static str,
    /// What the Purpose program isolates, and what its `run` is given.
    purpose: &'static str,
    purpose_argument: i64,
}

const BENCHMARKS: [Benchmark; 7] = [
    Benchmark {
        name: "fib",
        title: "`fib(32)`, recursive",
        tests: "calls",
        argument: 32,
        expected: "2178309",
        purpose: "calls alone: a procedure calling itself 1,000 deep, 20,000 times",
        purpose_argument: 1000,
    },
    Benchmark {
        name: "nbody",
        title: "n-body, 100,000 steps",
        tests: "float maths, records",
        argument: 100_000,
        expected: "-0.16907985939165887",
        purpose: "float maths on one record's fields, 5,000,000 times, with nothing called or allocated",
        purpose_argument: 5_000_000,
    },
    Benchmark {
        name: "spectral",
        title: "spectral-norm, size 200",
        tests: "loops, arrays, floats",
        argument: 200,
        expected: "1.2742236013532107",
        purpose: "a loop over a list of 1,000 floats, 20,000 times",
        purpose_argument: 20_000,
    },
    Benchmark {
        name: "fannkuch",
        title: "fannkuch-redux, 9",
        tests: "array shuffling, `Whole` maths",
        argument: 9,
        expected: "862930",
        purpose: "reversing a list of 16 whole numbers in place, 1,000,000 times",
        purpose_argument: 1_000_000,
    },
    Benchmark {
        name: "wordcount",
        title: "word count, 500,000 words",
        tests: "`Dict` and `Text`",
        argument: 500_000,
        expected: "5000000140",
        purpose: "looking up and counting 1,000 text keys in a `Dict`, 5,000 times",
        purpose_argument: 5000,
    },
    Benchmark {
        name: "textbuild",
        title: "text building, 20,000 joins",
        tests: "joining text many times",
        argument: 20_000,
        expected: "text 108890 19998,19999,",
        purpose: "one text growing by two characters, 30,000 times",
        purpose_argument: 30_000,
    },
    Benchmark {
        name: "trees",
        title: "binary-trees, depth 14",
        tests: "making and dropping heap values (joined by D72)",
        argument: 14,
        expected: "3222190",
        purpose: "making and dropping a tree 10 deep, 2,000 times, never reading it",
        purpose_argument: 2000,
    },
];

/// The new number types, timed on Seal alone (D115, D116): no other engine here has them. Each is a program, what
/// it does, what `run` is given, and the answer it must give, checked against Python's own `decimal` and integers.
const NUMBER_BENCHMARKS: [(&str, &str, i64, &str); 4] = [
    ("money.normal", "`Decimal`: 1,000,000 prices summed with tax, then split three ways", 1_000_000, "6786395"),
    ("money.float", "the same sums in `Float`s, as a yardstick", 1_000_000, "6786395"),
    ("factorial.normal", "`HugeWhole`: 3000 factorial, 9,131 digits", 3000, "9131"),
    ("interest.normal", "`HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits", 2000, "4048"),
];

/// The patrol benchmark's sizes: the Normal run, and the Purpose run, which isolates pausing in one script.
const PATROL: (usize, usize) = (1000, 100);
const PATROL_PURPOSE: (usize, usize) = (1, 1_000_000);

/// What an engine's patrol runs gave: every number each printed, by its name, then the answer and the peak memory.
type PatrolRuns = (BTreeMap<String, Vec<f64>>, String, u64);

/// What one process gave back.
#[derive(Clone, Default)]
struct Measured {
    result: String,
    times: Vec<f64>,
    first: Option<f64>,
    numbers: BTreeMap<String, f64>,
    /// The process's peak resident memory, in bytes.
    peak: Option<u64>,
    wall: f64,
}

#[allow(clippy::cast_precision_loss)] // counts of runs
fn mean(values: &[f64]) -> f64 {
    if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 }
}

#[allow(clippy::cast_precision_loss)] // counts of runs
fn deviation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let middle = mean(values);
    (values.iter().map(|value| (value - middle).powi(2)).sum::<f64>() / (values.len() - 1) as f64).sqrt()
}

fn least(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::INFINITY, f64::min)
}

/// Runs a command under `/usr/bin/time -l` and reads what it printed.
fn measure(program: &str, arguments: &[String]) -> Result<Measured, String> {
    let started = std::time::Instant::now();
    let output = Command::new("/usr/bin/time")
        .arg("-l")
        .arg(program)
        .args(arguments)
        .output()
        .map_err(|error| format!("`{program}` didn't start: {error}"))?;
    let wall = started.elapsed().as_secs_f64();
    let printed = String::from_utf8_lossy(&output.stdout);
    let errors = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(format!("`{program} {}` stopped: {}", arguments.join(" "), errors.trim()));
    }
    let mut measured = Measured { wall, ..Measured::default() };
    for line in printed.lines() {
        let Some((word, rest)) = line.split_once(' ') else { continue };
        match word {
            "result" => rest.clone_into(&mut measured.result),
            "times" => measured.times = rest.split(' ').filter_map(|time| time.parse().ok()).collect(),
            "first" => measured.first = rest.parse().ok(),
            _ => {
                if let Ok(number) = rest.parse::<f64>() {
                    measured.numbers.insert(word.to_owned(), number);
                }
            }
        }
    }
    measured.peak = errors
        .lines()
        .find(|line| line.contains("maximum resident set size"))
        .and_then(|line| line.split_whitespace().next())
        .and_then(|bytes| bytes.parse().ok());
    Ok(measured)
}

/// Whether two results are the same answer: the same text, or the same number however it's written.
fn same_answer(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(left), Ok(right)) => left.to_bits() == right.to_bits(),
        _ => false,
    }
}

/// A share of another time, as a percentage: with a decimal place when it's small.
fn share(part: f64) -> String {
    if part < 0.1 { format!("{:.1}%", part * 100.0) } else { format!("{:.0}%", part * 100.0) }
}

fn seconds(value: f64) -> String {
    if value >= 0.001 {
        format!("{:.2} ms", value * 1000.0)
    } else if value >= 0.000_001 {
        format!("{:.2} µs", value * 1_000_000.0)
    } else {
        format!("{:.0} ns", value * 1_000_000_000.0)
    }
}

#[allow(clippy::cast_precision_loss)] // sizes shown to one decimal place
fn bytes(value: u64) -> String {
    if value >= 1 << 30 {
        format!("{:.0} GB", value as f64 / f64::from(1 << 30))
    } else if value >= 1 << 20 {
        format!("{:.1} MB", value as f64 / f64::from(1 << 20))
    } else if value >= 1 << 10 {
        format!("{:.1} KB", value as f64 / f64::from(1 << 10))
    } else {
        format!("{value} B")
    }
}

/// The engines other than Seal: its name in the report, its program, its flags, and its scripts' extension.
struct Engine {
    name: &'static str,
    program: &'static str,
    flags: &'static [&'static str],
    extension: &'static str,
    warmups: usize,
}

// trace: D142. LuaJIT runs the Luau programs as they are, which are plain Lua; with `-joff` it's its interpreter alone.
const ENGINES: [Engine; 6] = [
    Engine { name: "Luau (interpreter)", program: LUAU, flags: &[], extension: "luau", warmups: 2 },
    Engine { name: "LuaJIT `-joff`", program: LUAJIT, flags: &["-joff"], extension: "luau", warmups: 2 },
    Engine { name: "LuaJIT (JIT)", program: LUAJIT, flags: &[], extension: "luau", warmups: 2 },
    Engine { name: "Node `--jitless`", program: NODE, flags: &["--jitless"], extension: "js", warmups: 2 },
    Engine { name: "Node (JIT), from cold", program: NODE, flags: &[], extension: "js", warmups: 0 },
    Engine { name: "Node (JIT), warmed up", program: NODE, flags: &[], extension: "js", warmups: 30 },
];

struct Report {
    root: PathBuf,
    scratch: PathBuf,
    runs: usize,
    seal: String,
    /// Other builds of Seal's benchmark host: the kind each stands for, and where it is.
    variants: Vec<(String, String)>,
    out: String,
    /// Each Normal row's mean time, by benchmark and engine, for the goals.
    normal: BTreeMap<(String, String), f64>,
    problems: Vec<String>,
}

impl Report {
    fn program(&self, name: &str, kind: &str, extension: &str) -> Option<PathBuf> {
        let path = self.root.join(format!("bench/programs/{name}.{kind}.{extension}"));
        path.exists().then_some(path)
    }

    /// A benchmark's program in a kind. A Ceil program that isn't written falls back on the Normal one.
    fn program_or_normal(&self, name: &str, kind: &str, extension: &str) -> Option<(PathBuf, bool)> {
        match self.program(name, kind, extension) {
            Some(path) => Some((path, false)),
            None => self.program(name, "normal", extension).map(|path| (path, true)),
        }
    }

    fn seal_row(&self, binary: &str, program: &Path, argument: i64) -> Result<Measured, String> {
        measure(
            binary,
            &[
                "seal".to_owned(),
                program.display().to_string(),
                argument.to_string(),
                "2".to_owned(),
                self.runs.to_string(),
            ],
        )
    }

    fn other_row(&self, engine: &Engine, program: &Path, argument: i64) -> Result<Measured, String> {
        let source = std::fs::read_to_string(program).map_err(|_| format!("`{}` can't be read", program.display()))?;
        let driver = std::fs::read_to_string(self.root.join(format!("bench/drivers/driver.{}", engine.extension)))
            .map_err(|_| "the driver can't be read".to_owned())?
            .replace("ARGUMENT", &argument.to_string())
            .replace("WARMUPS", &engine.warmups.to_string())
            .replace("RUNS", &self.runs.to_string());
        let file = self.scratch.join(format!("run.{}", engine.extension));
        std::fs::write(&file, format!("{source}\n{driver}")).map_err(|error| error.to_string())?;
        let mut arguments: Vec<String> = engine.flags.iter().map(|flag| (*flag).to_owned()).collect();
        arguments.push(file.display().to_string());
        measure(engine.program, &arguments)
    }

    /// How long a whole process takes to start, load a script and stop, running nothing: the cold load.
    fn cold_load(program: &str, flags: &[&str], script: &Path) -> Option<f64> {
        let mut walls = Vec::new();
        for _ in 0..5 {
            let mut arguments: Vec<String> = flags.iter().map(|flag| (*flag).to_owned()).collect();
            arguments.push(script.display().to_string());
            walls.push(measure(program, &arguments).ok()?.wall);
        }
        Some(least(&walls))
    }

    fn row(&mut self, engine: &str, kind: &str, measured: &Measured, size: u64, load: &str, note: &str) {
        let peak = measured.peak.map_or_else(|| "?".to_owned(), bytes);
        let first = measured.first.map_or_else(|| "?".to_owned(), seconds);
        let _ = writeln!(
            self.out,
            "| {engine} | {kind} | {} | {} | {} | {first} | {peak} | {} | {load} | {note} |",
            seconds(mean(&measured.times)),
            seconds(deviation(&measured.times)),
            seconds(least(&measured.times)),
            bytes(size),
        );
    }

    /// Runs one benchmark in one kind on every engine, checking the answers before writing the times.
    #[allow(clippy::too_many_lines)] // one benchmark's rows, written in the order they're read
    fn kind(&mut self, benchmark: &Benchmark, kind: &str) {
        let (program_kind, argument) = match kind {
            "Purpose" => ("purpose", benchmark.purpose_argument),
            "Normal" => ("normal", benchmark.argument),
            _ => ("ceil", benchmark.argument),
        };
        let mut rows: Vec<(String, String, Measured, u64, String, String)> = Vec::new();
        // Seal, in this kind's build.
        let binaries: Vec<(String, String)> = if kind.starts_with("Ceil") {
            let mut binaries = vec![("Ceil.source".to_owned(), self.seal.clone())];
            binaries.extend(self.variants.iter().filter(|(label, _)| label.starts_with("Ceil.")).cloned());
            binaries
        } else {
            vec![(kind.to_owned(), self.seal.clone())]
        };
        if let Some((program, stand_in)) = self.program_or_normal(benchmark.name, program_kind, "seal") {
            let size = std::fs::metadata(&program).map_or(0, |metadata| metadata.len());
            for (label, binary) in &binaries {
                match self.seal_row(binary, &program, argument) {
                    Ok(measured) => {
                        let warm = measured.numbers.get("load_warm").copied().unwrap_or(0.0);
                        let cold = measured.numbers.get("load").copied().unwrap_or(0.0);
                        let whole = Self::cold_load(binary, &["load"], &program)
                            .map_or_else(String::new, |wall| format!("; {} as a whole process", seconds(wall)));
                        let load = format!("{} cold, {} warm{whole}", seconds(cold), seconds(warm));
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // a count of bytes
                        let counted = measured.numbers.get("memory").map_or(0, |memory| *memory as u64);
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // a count of bytes
                        let code = measured.numbers.get("code").map_or(0, |code| *code as u64);
                        let mut note = format!("{} counted by Seal; {} compiled", bytes(counted), bytes(code));
                        if stand_in {
                            note.push_str("; the Normal program stands in");
                        }
                        rows.push(("Seal".to_owned(), label.clone(), measured, size, load, note));
                    }
                    Err(why) => self.problems.push(format!("{} {label} on Seal: {why}", benchmark.name)),
                }
            }
        }
        // The other engines run the Purpose, Normal and Ceil.source kinds: their builds are what's installed.
        for engine in &ENGINES {
            let Some((program, stand_in)) = self.program_or_normal(benchmark.name, program_kind, engine.extension)
            else {
                continue;
            };
            let size = std::fs::metadata(&program).map_or(0, |metadata| metadata.len());
            match self.other_row(engine, &program, argument) {
                Ok(measured) => {
                    let load = Self::cold_load(engine.program, engine.flags, &program)
                        .map_or_else(|| "?".to_owned(), |wall| format!("{} as a whole process", seconds(wall)));
                    let label = if kind.starts_with("Ceil") { "Ceil.source" } else { kind };
                    let note = if stand_in { "the Normal program stands in" } else { "" };
                    rows.push((engine.name.to_owned(), label.to_owned(), measured, size, load, note.to_owned()));
                }
                Err(why) => self.problems.push(format!("{} {kind} on {}: {why}", benchmark.name, engine.name)),
            }
        }
        // Correctness first (D58): a wrong answer is reported as wrong, and its time isn't written.
        let wanted = if kind == "Purpose" {
            rows.first().map(|row| row.2.result.clone()).unwrap_or_default()
        } else {
            benchmark.expected.to_owned()
        };
        for (engine, label, measured, size, load, note) in rows {
            if same_answer(&measured.result, &wanted) {
                if label == "Normal" {
                    self.normal.insert((benchmark.name.to_owned(), engine.clone()), mean(&measured.times));
                }
                self.row(&engine, &label, &measured, size, &load, &note);
            } else {
                let _ = writeln!(
                    self.out,
                    "| {engine} | {label} | **wrong answer**: `{}` where `{wanted}` is right | | | | | | | |",
                    measured.result
                );
                self.problems.push(format!("{} {label} on {engine} gave a wrong answer", benchmark.name));
            }
        }
    }

    fn benchmark(&mut self, benchmark: &Benchmark) {
        let _ = writeln!(self.out, "\n## {}\n", benchmark.title);
        let _ = writeln!(self.out, "Tests: {}. Purpose isolates {}.\n", benchmark.tests, benchmark.purpose);
        let _ = writeln!(
            self.out,
            "| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |"
        );
        let _ = writeln!(self.out, "|---|---|---|---|---|---|---|---|---|---|");
        for kind in ["Purpose", "Normal", "Ceil"] {
            self.kind(benchmark, kind);
        }
    }

    /// One engine's patrol run, repeated as whole processes: what each prints, gathered by its name.
    fn patrol_runs(&self, program: &str, arguments: &[String]) -> Result<PatrolRuns, String> {
        let mut gathered: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        let mut result = String::new();
        let mut peak = 0;
        for _ in 0..self.runs {
            let measured = measure(program, arguments)?;
            for (name, number) in &measured.numbers {
                gathered.entry(name.clone()).or_default().push(*number);
            }
            result = measured.result;
            peak = peak.max(measured.peak.unwrap_or(0));
        }
        Ok((gathered, result, peak))
    }

    #[allow(clippy::too_many_lines)] // the patrol benchmark's rows, written in the order they're read
    fn patrol(&mut self) {
        let _ = writeln!(self.out, "\n## 1,000 patrolling scripts\n");
        let _ = writeln!(
            self.out,
            "Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and \
             paused in its patrol; then every one is resumed {} times, and then a procedure of each is called {} \
             times. Purpose isolates pausing: one script, resumed {} times. Each row is {} whole processes. Loading is timed two \
             ways: every script from a text of its own, so that nothing compiled is reused, and every script from the \
             same text, where Seal reuses what it compiled, and the other engines compile once and make each script from \
             that.\n",
            PATROL.1, PATROL.1, PATROL_PURPOSE.1, self.runs
        );
        let _ = writeln!(
            self.out,
            "| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |"
        );
        let _ = writeln!(self.out, "|---|---|---|---|---|---|---|---|---|");
        let script = self.root.join("bench/programs/patrol.normal.seal");
        for (kind, (count, rounds)) in [("Purpose", PATROL_PURPOSE), ("Normal", PATROL), ("Ceil", PATROL)] {
            let wanted = (count + 2 * count * rounds).to_string();
            let mut binaries: Vec<(String, String)> = Vec::new();
            if kind == "Ceil" {
                binaries.push(("Ceil.source".to_owned(), self.seal.clone()));
                binaries.extend(self.variants.iter().filter(|(label, _)| label.starts_with("Ceil.")).cloned());
            } else {
                binaries.push((kind.to_owned(), self.seal.clone()));
            }
            let stand_in = if kind == "Ceil" { "the Normal program stands in" } else { "" };
            let mut rows = Vec::new();
            for (label, binary) in binaries {
                let arguments =
                    ["patrol".to_owned(), script.display().to_string(), count.to_string(), rounds.to_string()];
                match self.patrol_runs(&binary, &arguments) {
                    Ok(run) => rows.push(("Seal".to_owned(), label, run, "counted by Seal".to_owned())),
                    Err(why) => self.problems.push(format!("patrol {label} on Seal: {why}")),
                }
            }
            for engine in ENGINES.iter().filter(|engine| !engine.name.contains("warmed")) {
                let driver =
                    std::fs::read_to_string(self.root.join(format!("bench/drivers/patrol.{}", engine.extension)))
                        .unwrap_or_default()
                        .replace("COUNT", &count.to_string())
                        .replace("ROUNDS", &rounds.to_string());
                let file = self.scratch.join(format!("patrol.{}", engine.extension));
                if std::fs::write(&file, driver).is_err() {
                    continue;
                }
                let mut arguments: Vec<String> = engine.flags.iter().map(|flag| (*flag).to_owned()).collect();
                if engine.extension == "js" {
                    arguments.push("--expose-gc".to_owned());
                }
                arguments.push(file.display().to_string());
                let label = if kind == "Ceil" { "Ceil.source" } else { kind };
                match self.patrol_runs(engine.program, &arguments) {
                    Ok(run) => rows.push((
                        engine.name.replace(", from cold", ""),
                        label.to_owned(),
                        run,
                        "the engine's own heap".to_owned(),
                    )),
                    Err(why) => self.problems.push(format!("patrol {kind} on {}: {why}", engine.name)),
                }
            }
            for (engine, label, (numbers, result, peak), memory_note) in rows {
                if !same_answer(&result, &wanted) {
                    let _ = writeln!(
                        self.out,
                        "| {engine} | {label} | **wrong answer**: `{result}` where `{wanted}` is right | | | | | | |"
                    );
                    self.problems.push(format!("patrol {label} on {engine} gave a wrong answer"));
                    continue;
                }
                let cell = |name: &str| {
                    numbers.get(name).map_or_else(
                        || "?".to_owned(),
                        |values| {
                            format!(
                                "{} (±{}, least {})",
                                seconds(mean(values)),
                                seconds(deviation(values)),
                                seconds(least(values))
                            )
                        },
                    )
                };
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // a count of bytes
                let memory = numbers.get("memory_each").map_or(0, |values| mean(values) as u64);
                // Seal times loading from a text of its own apart, and reuses what it compiled for the same text. The
                // others compile each time unless told otherwise, and time the compiled-once way apart.
                let (own_text, same_text) =
                    if engine == "Seal" { ("load_fresh_each", "load_each") } else { ("load_each", "load_shared_each") };
                if label == "Normal" {
                    for (name, kept_as) in [
                        (own_text, "load_own_text"),
                        (same_text, "load_same_text"),
                        ("resume_each", "resume_each"),
                        ("call_each", "call_each"),
                    ] {
                        if let Some(values) = numbers.get(name) {
                            self.normal.insert((format!("patrol.{kept_as}"), engine.clone()), mean(values));
                        }
                    }
                    #[allow(clippy::cast_precision_loss)] // a count of bytes
                    self.normal.insert(("patrol.memory_each".to_owned(), engine.clone()), memory as f64);
                }
                let _ = writeln!(
                    self.out,
                    "| {engine} | {label} | {} | {} | {} | {} | {} | {} | {memory_note}{}{stand_in} |",
                    cell(own_text),
                    cell(same_text),
                    cell("resume_each"),
                    cell("call_each"),
                    bytes(memory),
                    bytes(peak),
                    if stand_in.is_empty() { "" } else { "; " },
                );
            }
        }
    }

    /// What Seal's checks cost (D58): the Normal programs on builds with a check left out, for measuring only.
    fn checks(&mut self) {
        let builds: Vec<(String, String)> =
            self.variants.iter().filter(|(label, _)| label.starts_with("Checks.")).cloned().collect();
        if builds.is_empty() {
            return;
        }
        let _ = writeln!(self.out, "\n## What Seal's checks cost (Purpose)\n");
        let _ = writeln!(
            self.out,
            "D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a \
             build of the VM with one kind of check left out. Those builds exist only for this table: a script never \
             runs without its checks (D17).\n"
        );
        let _ = writeln!(
            self.out,
            "| Benchmark | With every check | {} |",
            builds.iter().map(|(label, _)| label.replace("Checks.", "Without ")).collect::<Vec<_>>().join(" | ")
        );
        let _ = writeln!(self.out, "|---|---|{}", "---|".repeat(builds.len()));
        for benchmark in &BENCHMARKS {
            let Some(program) = self.program(benchmark.name, "normal", "seal") else { continue };
            let Ok(usual) = self.seal_row(&self.seal, &program, benchmark.argument) else { continue };
            let base = mean(&usual.times);
            let mut cells = Vec::new();
            for (_, binary) in &builds {
                cells.push(match self.seal_row(binary, &program, benchmark.argument) {
                    Ok(measured) if base > 0.0 => {
                        let time = mean(&measured.times);
                        format!("{} ({:+.1}%)", seconds(time), (time / base - 1.0) * 100.0)
                    }
                    _ => "?".to_owned(),
                });
            }
            let _ = writeln!(self.out, "| {} | {} | {} |", benchmark.title, seconds(base), cells.join(" | "));
        }
    }

    /// trace: D57, D115, D116. The new number types, timed, with their answers checked.
    fn numbers(&mut self) {
        let _ = writeln!(self.out, "\n## The new number types (D115, D116), Normal\n");
        let _ = writeln!(
            self.out,
            "Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what \
             exactness costs.\n"
        );
        let _ = writeln!(self.out, "| Program | Mean | Deviation | Least | Peak memory |");
        let _ = writeln!(self.out, "|---|---|---|---|---|");
        for (name, title, argument, expected) in NUMBER_BENCHMARKS {
            let program = self.root.join(format!("bench/programs/{name}.seal"));
            match self.seal_row(&self.seal, &program, argument) {
                Ok(measured) if same_answer(&measured.result, expected) => {
                    let peak = measured.peak.map_or_else(|| "?".to_owned(), bytes);
                    let _ = writeln!(
                        self.out,
                        "| {title} | {} | {} | {} | {peak} |",
                        seconds(mean(&measured.times)),
                        seconds(deviation(&measured.times)),
                        seconds(least(&measured.times))
                    );
                }
                Ok(measured) => self.problems.push(format!("`{name}` gave {}, not {expected}", measured.result)),
                Err(why) => self.problems.push(format!("`{name}`: {why}")),
            }
        }
    }

    fn goals(&self) -> String {
        let mut out = String::new();
        let time = |benchmark: &str, engine: &str| self.normal.get(&(benchmark.to_owned(), engine.to_owned())).copied();
        let _ = writeln!(out, "## D57's goals, judged Normal against Normal (D58)\n");
        let _ = writeln!(out, "| Goal | Benchmark | Seal | Against | Verdict |");
        let _ = writeln!(out, "|---|---|---|---|---|");
        let verdict = |seal: Option<f64>, other: Option<f64>, limit: f64| match (seal, other) {
            (Some(seal), Some(other)) if seal < other * limit => {
                format!("**met**: Seal takes {} of its time", share(seal / other))
            }
            (Some(seal), Some(other)) => format!("**not met**: Seal takes {} of its time", share(seal / other)),
            _ => "not measured".to_owned(),
        };
        let shown = |value: Option<f64>| value.map_or_else(|| "?".to_owned(), seconds);
        // The patrolling scripts as one run: every script loaded from the same text, then resumed. Each engine is
        // taken at its better way of loading the same text, compiled once.
        #[allow(clippy::cast_precision_loss)] // counts of scripts and rounds
        let whole_run = |engine: &str| {
            let load = time("patrol.load_same_text", engine)?;
            let resume = time("patrol.resume_each", engine)?;
            Some(load * PATROL.0 as f64 + resume * (PATROL.0 * PATROL.1) as f64)
        };
        // trace: D57, D142. Goal 1 has two bars, Luau's interpreter and LuaJIT's.
        for (bar, engine) in [("Luau's interpreter", "Luau (interpreter)"), ("LuaJIT's interpreter", "LuaJIT `-joff`")]
        {
            for benchmark in &BENCHMARKS {
                let (seal, other) = (time(benchmark.name, "Seal"), time(benchmark.name, engine));
                let _ = writeln!(
                    out,
                    "| 1. Faster than {bar} | {} | {} | {} | {} |",
                    benchmark.title,
                    shown(seal),
                    shown(other),
                    verdict(seal, other, 1.0)
                );
            }
            let (seal, other) = (whole_run("Seal"), whole_run(engine));
            let _ = writeln!(
                out,
                "| 1. Faster than {bar} | patrolling scripts, the whole run: loading {} from one text, resuming each {} times | {} | {} | {} |",
                PATROL.0,
                PATROL.1,
                shown(seal),
                shown(other),
                verdict(seal, other, 1.0)
            );
            for (part, what) in [
                ("resume_each", "resuming each"),
                ("load_same_text", "loading each from the same text"),
                ("load_own_text", "loading each from a text of its own"),
            ] {
                let name = format!("patrol.{part}");
                let (seal, other) = (time(&name, "Seal"), time(&name, engine));
                // trace: D150. Loading from a text of its own is judged against Luau alone; LuaJIT's is for reference.
                let judged = if part == "load_own_text" && engine.starts_with("LuaJIT") {
                    "for reference, no target (D150)".to_owned()
                } else {
                    verdict(seal, other, 1.0)
                };
                let _ = writeln!(
                    out,
                    "| 1. Faster than {bar} | patrolling scripts, {what} | {} | {} | {judged} |",
                    shown(seal),
                    shown(other),
                );
            }
        }
        for (name, limit, goal) in [
            ("fib", 1.0, "2. Faster than V8 `--jitless`"),
            ("nbody", 1.0, "2. Faster than V8 `--jitless`"),
            ("wordcount", 2.0, "2. Within twice V8 `--jitless`"),
            ("textbuild", 2.0, "2. Within twice V8 `--jitless`"),
        ] {
            let title = BENCHMARKS.iter().find(|benchmark| benchmark.name == name).map_or(name, |found| found.title);
            let (seal, node) = (time(name, "Seal"), time(name, "Node `--jitless`"));
            let _ = writeln!(
                out,
                "| {goal} | {title} | {} | {} | {} |",
                shown(seal),
                shown(node),
                verdict(seal, node, limit)
            );
        }
        let (resume, call) = (time("patrol.resume_each", "Seal"), time("patrol.call_each", "Seal"));
        let _ = writeln!(
            out,
            "| 6. Pausing costs about as much as a call | patrolling scripts | resume {} | host call {} | {} |",
            shown(resume),
            shown(call),
            match (resume, call) {
                (Some(resume), Some(call)) if resume <= call * 1.5 => "**met**".to_owned(),
                (Some(_), Some(_)) => "**not met**".to_owned(),
                _ => "not measured".to_owned(),
            }
        );
        if let Some(memory) = time("patrol.memory_each", "Seal") {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // a count of bytes
            let each = bytes(memory as u64);
            let _ = writeln!(
                out,
                "| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | {each} each, counted | | {} |",
                if memory < 16.0 * 1024.0 { "**met**" } else { "**not met**" }
            );
        }
        out
    }
}

/// Luau's version: its command prints none of its own, but says where the installed program lives, and that
/// folder is named after the version.
fn luau_version() -> String {
    version(LUAU, "--version")
        .split(['/', ' '])
        .find(|part| {
            part.split('.').count() == 3 && part.chars().all(|character| character.is_ascii_digit() || character == '.')
        })
        .map_or_else(|| "?".to_owned(), str::to_owned)
}

fn version(program: &str, flag: &str) -> String {
    Command::new(program)
        .arg(flag)
        .output()
        .ok()
        .map(|output| format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))
        .and_then(|text| text.lines().next().map(str::to_owned))
        .unwrap_or_else(|| "?".to_owned())
}

fn system(name: &str) -> String {
    Command::new("sysctl")
        .args(["-n", name])
        .output()
        .ok()
        .map_or_else(|| "?".to_owned(), |output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Runs everything and writes `bench/RESULTS.md`, giving a short account of how it went.
#[allow(clippy::too_many_lines)] // the report, written top to bottom in the order it's read
pub fn report(options: &[&str]) -> Result<String, String> {
    let mut runs = 10usize;
    let mut variants = Vec::new();
    let mut only: Option<String> = None;
    let mut options = options.iter();
    while let Some(option) = options.next() {
        match *option {
            "--runs" => runs = options.next().and_then(|runs| runs.parse().ok()).unwrap_or(10).max(10),
            "--variant" => {
                let Some((label, path)) = options.next().and_then(|variant| variant.split_once('=')) else {
                    return Err("`--variant` takes LABEL=PATH".to_owned());
                };
                variants.push((label.to_owned(), path.to_owned()));
            }
            "--only" => only = options.next().map(|name| (*name).to_owned()),
            other => return Err(format!("`report` doesn't know `{other}`")),
        }
    }
    let root = std::env::current_dir().map_err(|error| error.to_string())?;
    if !root.join("bench/programs").exists() {
        return Err("run `seal-bench report` from the repository's folder".to_owned());
    }
    let seal = std::env::current_exe().map_err(|error| error.to_string())?.display().to_string();
    let scratch = std::env::temp_dir().join(format!("seal-bench-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).map_err(|error| error.to_string())?;
    let mut report = Report {
        root,
        scratch: scratch.clone(),
        runs,
        seal,
        variants,
        out: String::new(),
        normal: BTreeMap::new(),
        problems: Vec::new(),
    };
    let busy_before = system("vm.loadavg");
    for benchmark in &BENCHMARKS {
        if only.as_deref().is_none_or(|only| only == benchmark.name) {
            report.benchmark(benchmark);
        }
    }
    if only.as_deref().is_none_or(|only| only == "patrol") {
        report.patrol();
    }
    if only.is_none() {
        report.checks();
        report.numbers();
    }
    // D57's fourth goal: a 10,000-line script checked and compiled in under 100 ms.
    let lines = measure(&report.seal, &["lines".to_owned(), "10000".to_owned()]).ok();
    let busy_after = system("vm.loadavg");
    let _ = std::fs::remove_dir_all(&scratch);

    let mut text = String::new();
    let _ = writeln!(text, "# Seal's benchmark results\n");
    let _ = writeln!(
        text,
        "Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every \
         benchmark in every kind, every number saying its kind, and every answer checked before its time counts.\n"
    );
    let _ = writeln!(text, "## Setup\n");
    let _ = writeln!(
        text,
        "- **Machine:** {}, {} cores, {}.",
        system("machdep.cpu.brand_string"),
        system("hw.ncpu"),
        bytes(system("hw.memsize").parse().unwrap_or(0))
    );
    let _ =
        writeln!(text, "- **System:** macOS {} ({}).", version("sw_vers", "-productVersion"), system("kern.osrelease"));
    let _ = writeln!(
        text,
        "- **Seal:** this repository, built by `{}`, release profile (optimised, link-time optimisation, one code unit).",
        version("rustc", "--version")
    );
    let _ = writeln!(
        text,
        "- **Luau:** `{}`, its interpreter, with native code generation off (the default of the `luau` command).",
        luau_version()
    );
    let _ = writeln!(
        text,
        "- **LuaJIT:** `{}`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.",
        version(LUAJIT, "-v")
    );
    let _ = writeln!(
        text,
        "- **Node:** `{}`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).",
        version(NODE, "--version")
    );
    let _ = writeln!(
        text,
        "- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{busy_before}` before, `{busy_after}` after. Other sessions were working on this machine during the run."
    );
    let _ = writeln!(
        text,
        "- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then {} times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.",
        report.runs
    );
    let _ = writeln!(
        text,
        "- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine."
    );
    let _ = writeln!(
        text,
        "- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.\n"
    );
    text.push_str(&report.goals());
    if let Some(lines) = lines {
        let best = lines.numbers.get("best").copied().unwrap_or(0.0);
        let count = lines.numbers.get("lines").copied().unwrap_or(0.0);
        let _ = writeln!(
            text,
            "| 4. A 10,000-line script loads in under 100 ms | a generated script of {count:.0} lines | {} best of 10 | 100 ms | {} |",
            seconds(best),
            if best < 0.1 { "**met**" } else { "**not met**" }
        );
    }
    let _ = writeln!(
        text,
        "\nGoal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost."
    );
    text.push_str(&report.out);
    if !report.problems.is_empty() {
        let _ = writeln!(text, "\n## What went wrong while measuring\n");
        for problem in &report.problems {
            let _ = writeln!(text, "- {problem}");
        }
    }
    let path = report.root.join("bench/RESULTS.md");
    let written = if only.is_none() {
        std::fs::write(&path, &text).map_err(|error| error.to_string())?;
        format!("wrote {}", path.display())
    } else {
        text.clone()
    };
    Ok(format!("{written}\n{} problems\n", report.problems.len()))
}

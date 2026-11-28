//! Wax's whole numbers of any size checked against GMP, on random numbers from a fixed seed, at the sizes where
//! its faster methods take over and far past them (D115, Building rule 5). GMP is reached through a small C program,
//! `gmp_reference.c`, built here with the system's C compiler, so nothing of GMP's gets into Wax. Where there's no
//! C compiler or no GMP, the test says so and compares nothing.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use wax::big::Big;

/// Where Homebrew puts GMP on a Mac with Apple's processors.
const GMP: &str = "/opt/homebrew";

struct Random(u64);

impl Random {
    fn below(&mut self, limit: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % limit.max(1)
    }

    /// A number of about `count` decimal digits, now and then all nines, where carries run furthest.
    fn number(&mut self, count: u64) -> String {
        let nines = self.below(8) == 0;
        let mut text = String::with_capacity(usize::try_from(count).unwrap_or(0) + 1);
        if self.below(2) == 0 {
            text.push('-');
        }
        text.push(char::from(b'1' + u8::try_from(self.below(9)).unwrap_or(0)));
        for _ in 1..count {
            let digit = if nines { 9 } else { self.below(10) };
            text.push(char::from(b'0' + u8::try_from(digit).unwrap_or(0)));
        }
        text
    }
}

/// The reference program, built once into the target folder; nothing when it can't be built here.
fn reference() -> Option<PathBuf> {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/gmp_reference.c");
    let program = std::env::temp_dir().join(format!("wax-gmp-reference-{}", std::process::id()));
    let built = Command::new("cc")
        .arg("-O2")
        .arg(&source)
        .arg(format!("-I{GMP}/include"))
        .arg(format!("-L{GMP}/lib"))
        .arg("-lgmp")
        .arg("-o")
        .arg(&program)
        .status()
        .ok()?;
    built.success().then_some(program)
}

// trace: D115
#[test]
fn whole_numbers_agree_with_gmp() {
    let Some(program) = reference() else {
        eprintln!("GMP or a C compiler isn't here, so nothing was compared");
        return;
    };
    let mut random = Random(0x6a09_e667_f3bc_c908);
    let mut input = String::new();
    let mut cases = Vec::new();
    let mut wax = Vec::new();
    // In base 2^32 digits: about each switch-over, then well past them all.
    let sizes: [u64; 9] = [60, 64, 70, 700, 768, 800, 1500, 5000, 12_000];
    // Every size meets every operation, twice.
    for (round, &size) in sizes.iter().cycle().take(sizes.len() * 12).enumerate() {
        let digits = size * 963 / 100;
        let extra = random.below(digits / 4 + 1);
        let a = random.number(digits + extra);
        let b_digits = match round % 3 {
            0 => digits,
            1 => digits / 2 + 1,
            _ => 1 + random.below(digits),
        };
        let b = random.number(b_digits);
        let operation = ["multiply", "floor", "truncate", "root", "add", "power"][(round / sizes.len()) % 6];
        let (x, y) = (Big::parse(&a).unwrap_or_default(), Big::parse(&b).unwrap_or_default());
        let line = match operation {
            "multiply" => x.multiply(&y).to_decimal(),
            "add" => x.add(&y).to_decimal(),
            "floor" | "truncate" => {
                let divided = if operation == "floor" { x.divide_floor(&y) } else { x.divide_truncating(&y) };
                divided.map_or_else(
                    || "by_zero".to_owned(),
                    |(quotient, remainder)| format!("{} {}", quotient.to_decimal(), remainder.to_decimal()),
                )
            }
            "root" => x.abs().square_root_floor().to_decimal(),
            _ => {
                // A short base raised far, so the answer is long.
                let base = Big::parse(&a[..a.len().min(12)]).unwrap_or_default();
                let exponent = 200 + random.below(2000);
                let line = base.power(exponent).to_decimal();
                let _ = writeln!(input, "power {} {exponent}", &a[..a.len().min(12)]);
                cases.push(format!("power, round {round}"));
                wax.push(line);
                continue;
            }
        };
        let _ = writeln!(input, "{operation} {a} {b}");
        cases.push(format!("{operation} of {} and {} decimal digits, round {round}", a.len(), b.len()));
        wax.push(line);
    }
    let mut child = Command::new(&program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("the reference didn't run: {error}"));
    // The cases are written while the answers are read, so neither side waits on a full pipe.
    let writer = child.stdin.take().map(|mut stdin| {
        std::thread::spawn(move || {
            stdin.write_all(input.as_bytes()).unwrap_or_else(|error| panic!("the cases weren't written: {error}"));
        })
    });
    let output = child.wait_with_output().unwrap_or_else(|error| panic!("the reference didn't finish: {error}"));
    if let Some(writer) = writer {
        writer.join().unwrap_or_else(|_| panic!("writing the cases went wrong"));
    }
    let _ = std::fs::remove_file(&program);
    let text = String::from_utf8_lossy(&output.stdout);
    let theirs: Vec<&str> = text.lines().collect();
    assert_eq!(theirs.len(), wax.len(), "GMP gave {} answers for {} cases", theirs.len(), wax.len());
    for ((case, ours), theirs) in cases.iter().zip(&wax).zip(theirs) {
        assert!(ours == theirs, "{case}: Wax and GMP differ");
    }
}

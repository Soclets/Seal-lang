//! Seal's number types checked against an outside reference, Python, on random numbers from a fixed seed (D115,
//! D116, D124 to D127): whole numbers against Python's `int`, `Decimal` against Python's `decimal` set to
//! decimal128's rules (34 digits, its exponent range, a tie to even), and `HugeDecimal` against exact fractions.
//! Each case is worked out by Seal here and by Python in a process of its own, and every line must match.

use std::fmt::Write as _;
use std::io::Write as _;
use std::process::{Command, Stdio};

use seal::big::Big;
use seal::decimal::{Decimal, HugeDecimal, NumberError};

/// A fixed-seed source of random numbers, so a failing case can be run again.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, limit: u64) -> u64 {
        self.next() % limit.max(1)
    }

    /// A whole number's digits, now and then all nines or a one and zeros, where carries run furthest.
    fn digits(&mut self, count: usize) -> String {
        let shape = self.below(10);
        let mut text = String::with_capacity(count);
        for index in 0..count {
            let digit = match shape {
                0 => 9,
                1 if index == 0 => 1,
                1 => 0,
                _ => self.below(10),
            };
            text.push(char::from(b'0' + u8::try_from(digit).unwrap_or(0)));
        }
        let trimmed = text.trim_start_matches('0');
        if trimmed.is_empty() { "0".to_owned() } else { trimmed.to_owned() }
    }

    fn sign(&mut self) -> &'static str {
        if self.below(2) == 0 { "-" } else { "" }
    }
}

/// Runs a Python program, giving what it prints, or nothing when Python can't be run here.
fn python(program: &str) -> Option<String> {
    let mut child =
        Command::new("python3").arg("-I").arg("-").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().ok()?;
    child.stdin.take()?.write_all(program.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    assert!(output.status.success(), "Python stopped: {}", String::from_utf8_lossy(&output.stdout));
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Compares Seal's lines with Python's, naming the first case that differs.
fn agree(what: &str, cases: &[String], seal: &[String], program: &str) {
    let Some(reference) = python(program) else {
        eprintln!("{what}: Python isn't here, so nothing was compared");
        return;
    };
    let reference: Vec<&str> = reference.lines().collect();
    assert_eq!(reference.len(), seal.len(), "{what}: Python gave {} lines for {} cases", reference.len(), seal.len());
    for ((case, ours), theirs) in cases.iter().zip(seal).zip(reference) {
        assert_eq!(ours, theirs, "{what}: {case}");
    }
}

fn error_name(error: NumberError) -> &'static str {
    match error {
        NumberError::ByZero => "by_zero",
        NumberError::Inexact => "inexact",
        NumberError::TooBig => "too_big",
        NumberError::Negative => "negative",
    }
}

// trace: D115
#[test]
fn whole_numbers_agree_with_python() {
    let mut random = Random(0x0dd_b1a5_e5ba_d5ee);
    let mut cases = Vec::new();
    let mut seal = Vec::new();
    let mut program = String::from("import sys\nsys.set_int_max_str_digits(0)\ncases = [\n");
    for round in 0..1500 {
        // Mostly ordinary sizes, now and then thousands of digits, past the faster methods' switch-overs.
        let size = |random: &mut Random| match random.below(10) {
            0..=5 => usize::try_from(random.below(40) + 1).unwrap_or(1),
            6..=8 => usize::try_from(random.below(400) + 1).unwrap_or(1),
            _ => usize::try_from(random.below(12_000) + 1).unwrap_or(1),
        };
        let (left_size, right_size) = (size(&mut random), size(&mut random));
        let a = format!("{}{}", random.sign(), random.digits(left_size));
        let b = format!("{}{}", random.sign(), random.digits(right_size));
        let (x, y) = (Big::parse(&a).unwrap_or_default(), Big::parse(&b).unwrap_or_default());
        let operation = ["add", "subtract", "multiply", "floor", "truncate", "text", "power", "root"][round % 8];
        let line = match operation {
            "add" => x.add(&y).to_decimal(),
            "subtract" => x.subtract(&y).to_decimal(),
            "multiply" => x.multiply(&y).to_decimal(),
            "floor" | "truncate" => {
                let divided = if operation == "floor" { x.divide_floor(&y) } else { x.divide_truncating(&y) };
                match divided {
                    Some((quotient, remainder)) => format!("{} {}", quotient.to_decimal(), remainder.to_decimal()),
                    None => "by_zero".to_owned(),
                }
            }
            "text" => Big::parse(&x.to_decimal()).map_or_else(|| "unread".to_owned(), |back| back.to_decimal()),
            "power" => {
                let exponent = random.below(30);
                let written = &a[..a.len().min(30)];
                let base = Big::parse(written).unwrap_or_default();
                let _ = writeln!(program, "  ('power', '{written}', '{exponent}'),");
                cases.push(format!("{operation} {written}"));
                seal.push(base.power(exponent).to_decimal());
                continue;
            }
            _ => x.abs().square_root_floor().to_decimal(),
        };
        let _ = writeln!(program, "  ('{operation}', '{a}', '{b}'),");
        cases.push(format!("{operation} of numbers of {left_size} and {right_size} digits"));
        seal.push(line);
    }
    program.push_str(
        "]\nimport math\nfor op, a, b in cases:\n    a, b = int(a), int(b)\n    if op == 'add': print(a + b)\n    elif op == 'subtract': print(a - b)\n    elif op == 'multiply': print(a * b)\n    elif op in ('floor', 'truncate'):\n        if b == 0: print('by_zero'); continue\n        if op == 'floor': q, r = divmod(a, b)\n        else:\n            q = abs(a) // abs(b) * (1 if (a < 0) == (b < 0) else -1); r = a - q * b\n        print(q, r)\n    elif op == 'text': print(int(str(a)))\n    elif op == 'power': print(a ** b)\n    else: print(math.isqrt(abs(a)))\n",
    );
    agree("whole numbers", &cases, &seal, &program);
}

/// A random `Decimal`: up to 34 digits, its exponent mostly near zero, now and then near either end of its range.
fn random_decimal(random: &mut Random) -> Decimal {
    let size = usize::try_from(random.below(34) + 1).unwrap_or(1);
    let coefficient = random.digits(size).parse::<u128>().unwrap_or(0);
    let exponent = match random.below(10) {
        0 => -6176 + i32::try_from(random.below(40)).unwrap_or(0),
        1 => 6111 - i32::try_from(random.below(40)).unwrap_or(0),
        _ => i32::try_from(random.below(41)).unwrap_or(0) - 20,
    };
    Decimal { negative: random.below(2) == 0, coefficient, exponent }
}

fn decimal_line(result: Result<Decimal, NumberError>) -> String {
    match result {
        Ok(value) if value.coefficient == 0 => format!("0 {}", value.exponent),
        Ok(value) => format!("{}{} {}", if value.negative { "-" } else { "" }, value.coefficient, value.exponent),
        Err(error) => error_name(error).to_owned(),
    }
}

fn python_decimal(value: &Decimal) -> String {
    format!("D('{}{}E{}')", if value.negative { "-" } else { "" }, value.coefficient, value.exponent)
}

// trace: D116, D124, D125, D127
#[test]
fn decimals_agree_with_python_decimal128() {
    let mut random = Random(0x1234_fedc_ba98_7654);
    let mut cases = Vec::new();
    let mut seal = Vec::new();
    let mut program = String::from(
        "import decimal\nfrom decimal import Decimal as D\nc = decimal.Context(prec=34, Emax=6144, Emin=-6143, rounding=decimal.ROUND_HALF_EVEN, clamp=1, traps=[decimal.Overflow, decimal.DivisionByZero, decimal.InvalidOperation])\ndecimal.setcontext(c)\ndef show(v):\n    s, digits, e = v.as_tuple()\n    n = int(''.join(map(str, digits)))\n    return f\"0 {e}\" if n == 0 else f\"{'-' if s else ''}{n} {e}\"\ncases = [\n",
    );
    for round in 0..3000 {
        let (a, b) = (random_decimal(&mut random), random_decimal(&mut random));
        let operation = ["add", "subtract", "multiply", "divide", "root"][round % 5];
        let line = match operation {
            "add" => decimal_line(a.add(&b)),
            "subtract" => decimal_line(a.subtract(&b)),
            "multiply" => decimal_line(a.multiply(&b)),
            "divide" => decimal_line(a.divide(&b)),
            _ => decimal_line(a.abs().square_root()),
        };
        let _ = writeln!(program, "  ('{operation}', {}, {}),", python_decimal(&a), python_decimal(&b));
        cases.push(format!("{operation} {} {}", python_decimal(&a), python_decimal(&b)));
        seal.push(line);
    }
    program.push_str(
        "]\nfor op, a, b in cases:\n    try:\n        if op == 'add': r = c.add(a, b)\n        elif op == 'subtract': r = c.subtract(a, b)\n        elif op == 'multiply': r = c.multiply(a, b)\n        elif op == 'divide': r = c.divide(a, b)\n        else: r = c.sqrt(abs(a))\n        print(show(r))\n    except decimal.Overflow: print('too_big')\n    except decimal.DivisionByZero: print('by_zero')\n    except decimal.InvalidOperation: print('by_zero' if op == 'divide' else 'invalid')\n",
    );
    agree("decimals", &cases, &seal, &program);
}

// trace: D116, D127
#[test]
fn huge_decimals_agree_with_exact_fractions() {
    let mut random = Random(0x7777_1234_abcd_0001);
    let mut cases = Vec::new();
    let mut seal = Vec::new();
    let mut program = String::from("from fractions import Fraction as F\ncases = [\n");
    let make = |random: &mut Random| {
        let size = usize::try_from(random.below(80) + 1).unwrap_or(1);
        let text = format!("{}{}", random.sign(), random.digits(size));
        let exponent = i64::try_from(random.below(61)).unwrap_or(0) - 30;
        (HugeDecimal::new(Big::parse(&text).unwrap_or_default(), exponent), format!("F({text}) * F(10) ** {exponent}"))
    };
    for round in 0..2000 {
        let ((a, a_text), (b, b_text)) = (make(&mut random), make(&mut random));
        let operation = ["add", "subtract", "multiply", "divide"][round % 4];
        let result = match operation {
            "add" => Ok(a.add(&b)),
            "subtract" => Ok(a.subtract(&b)),
            "multiply" => Ok(a.multiply(&b)),
            _ => a.divide(&b),
        };
        // An exact value, as a fraction in lowest terms.
        let line = match result {
            Ok(value) => {
                let (numerator, denominator) = if value.exponent >= 0 {
                    (
                        value.coefficient.multiply(&Big::from_u64(10).power(value.exponent.unsigned_abs())),
                        Big::from_u64(1),
                    )
                } else {
                    (value.coefficient.clone(), Big::from_u64(10).power(value.exponent.unsigned_abs()))
                };
                format!("fraction {} {}", numerator.to_decimal(), denominator.to_decimal())
            }
            Err(error) => error_name(error).to_owned(),
        };
        let _ = writeln!(program, "  ('{operation}', {a_text}, {b_text}),");
        cases.push(format!("{operation} {a_text} {b_text}"));
        seal.push(line);
    }
    program.push_str(
        "]\ndef exact(f):\n    d = f.denominator\n    for p in (2, 5):\n        while d % p == 0: d //= p\n    return d == 1\nfor op, a, b in cases:\n    if op == 'add': r = a + b\n    elif op == 'subtract': r = a - b\n    elif op == 'multiply': r = a * b\n    else:\n        if b == 0: print('by_zero'); continue\n        r = a / b\n        if not exact(r): print('inexact'); continue\n    print('fraction', r.numerator, r.denominator)\n",
    );
    // Seal's fraction isn't in lowest terms; Python's is, so Seal's is reduced the same way before comparing.
    let seal: Vec<String> = seal.into_iter().map(|line| lowest_terms(&line)).collect();
    agree("huge decimals", &cases, &seal, &program);
}

/// A "fraction n d" line in lowest terms.
fn lowest_terms(line: &str) -> String {
    let mut parts = line.split(' ');
    let (Some("fraction"), Some(numerator), Some(denominator)) = (parts.next(), parts.next(), parts.next()) else {
        return line.to_owned();
    };
    let (mut numerator, mut denominator) =
        (Big::parse(numerator).unwrap_or_default(), Big::parse(denominator).unwrap_or_default());
    let (two, five) = (Big::from_u64(2), Big::from_u64(5));
    for factor in [&two, &five] {
        while !denominator.compare(&Big::from_u64(1)).is_eq() {
            let (Some((n, n_rest)), Some((d, d_rest))) =
                (numerator.divide_truncating(factor), denominator.divide_truncating(factor))
            else {
                break;
            };
            if !n_rest.is_zero() || !d_rest.is_zero() {
                break;
            }
            (numerator, denominator) = (n, d);
        }
    }
    format!("fraction {} {}", numerator.to_decimal(), denominator.to_decimal())
}

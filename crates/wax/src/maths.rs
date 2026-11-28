//! Correctly rounded maths on `Float`s: `**`, the angle built-ins and `lerp` (D50, D108, D109, D114).
//!
//! Each function works its answer out as a big fixed-point number with a bound on its error, and gives the `Float`
//! that the whole interval rounds to; when the interval straddles two, it works again at twice the precision. This
//! is Ziv's strategy, which is how CORE-MATH and MPFR round correctly, written here with [`Big`] and no `unsafe`, so
//! the answer is the same on every machine (D50).
// Splitting wide numbers keeps exactly the bits wanted, which is what these casts are for; and the maths reads best in
// the formulas' own letters, x, y, m, e and k, as the comments write them. Comparing `Float`s exactly is what finding
// exact answers needs, and the `Float`s made from whole numbers here only judge sizes, never answers.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]

use std::cmp::Ordering;
use std::sync::Mutex;

use crate::big::Big;

/// The precision the first try works at, in bits after the point.
const FIRST_PRECISION: u64 = 128;
/// The precision past which the answer must be exact or a tie, which every function deals with before it gets here.
const LAST_PRECISION: u64 = 1 << 14;

/// A `Float` as an exact whole number times a power of two: `(m, e)` for `m × 2^e`, with `m` odd or zero.
fn decompose(value: f64) -> (Big, i64) {
    if value == 0.0 {
        return (Big::zero(), 0);
    }
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i64;
    let fraction = bits & ((1 << 52) - 1);
    let (mut mantissa, mut power) =
        if exponent == 0 { (fraction, -1074) } else { (fraction | (1 << 52), exponent - 1075) };
    let zeros = mantissa.trailing_zeros();
    mantissa >>= zeros;
    power += i64::from(zeros);
    let big = Big::from_u64(mantissa);
    (if value < 0.0 { big.negated() } else { big }, power)
}

/// The `Float` nearest to `m × 2^e`, ties to even, infinite past the largest (IEEE 754's rounding).
#[must_use]
pub fn round_dyadic(m: &Big, e: i64) -> f64 {
    if m.is_zero() {
        return 0.0;
    }
    let negative = m.is_negative();
    let magnitude = m.abs();
    let bits = magnitude.bits() as i64;
    // The exponent of the leading bit, and the place of the last bit a `Float` keeps.
    let lead = e + bits - 1;
    let last = (lead - 52).max(-1074);
    let quotient = if last > e {
        let shift = (last - e) as u64;
        let kept = magnitude.shift_right(shift);
        let half = shift >= 1 && magnitude.shift_right(shift - 1).is_odd();
        let below_half = shift >= 1 && magnitude.has_bits_below(shift - 1);
        let up = half && (below_half || kept.is_odd());
        if up { kept.add(&Big::from_u64(1)) } else { kept }
    } else {
        magnitude.shift_left((e - last) as u64)
    };
    let Some(mut quotient) = quotient.to_u128_magnitude() else {
        return if negative { f64::NEG_INFINITY } else { f64::INFINITY };
    };
    let mut last = last;
    if quotient == 1 << 53 {
        quotient = 1 << 52;
        last += 1;
    }
    let value = if quotient >= 1 << 52 {
        let biased = last + 1075;
        if biased >= 0x7ff {
            f64::INFINITY
        } else {
            f64::from_bits(((biased as u64) << 52) | ((quotient as u64) & ((1 << 52) - 1)))
        }
    } else {
        // A subnormal: `last` is -1074 here.
        f64::from_bits(quotient as u64)
    };
    if negative { -value } else { value }
}

/// An answer worked out to some precision: `approx × 2^exponent`, wrong by at most `error × 2^exponent`.
struct Approximation {
    approx: Big,
    exponent: i64,
    error: u64,
}

/// The `Float` every value in the approximation's interval rounds to, when there's one.
fn certain(approximation: &Approximation) -> Option<f64> {
    let error = Big::from_u64(approximation.error);
    let low = round_dyadic(&approximation.approx.subtract(&error), approximation.exponent);
    let high = round_dyadic(&approximation.approx.add(&error), approximation.exponent);
    (low.to_bits() == high.to_bits()).then_some(low)
}

/// Ziv's strategy: works the answer out at higher and higher precision until it's certain which `Float` it rounds to.
fn correctly_rounded(work: impl Fn(u64) -> Approximation) -> f64 {
    let mut precision = FIRST_PRECISION;
    loop {
        let approximation = work(precision);
        if let Some(answer) = certain(&approximation) {
            return answer;
        }
        if precision >= LAST_PRECISION {
            // Every function deals with its exact answers and ties first, so this is never reached; the nearest
            // `Float` to the best approximation is still the right one in every case but a tie.
            return round_dyadic(&approximation.approx, approximation.exponent);
        }
        precision *= 2;
    }
}

/// `value × 2^precision` as a whole number, truncated: a fixed-point number with `precision` bits after the point.
fn fixed(m: &Big, e: i64, precision: u64) -> Big {
    let shift = e + precision as i64;
    if shift >= 0 { m.shift_left(shift as u64) } else { m.shift_right((-shift) as u64) }
}

/// `a × b` for two fixed-point numbers.
fn multiply(a: &Big, b: &Big, precision: u64) -> Big {
    a.multiply(b).shift_right(precision)
}

/// `a ÷ b` for two fixed-point numbers.
fn divide(a: &Big, b: &Big, precision: u64) -> Big {
    a.shift_left(precision).divide_truncating(b).map(|(quotient, _)| quotient).unwrap_or_default()
}

/// The square root of a fixed-point number that isn't negative.
fn square_root(a: &Big, precision: u64) -> Big {
    a.shift_left(precision).square_root_floor()
}

/// `atan(1/k)` as a fixed-point number, for Machin's formula: Σ (-1)^n / ((2n+1) k^(2n+1)), each term rounded down.
fn arctan_of_inverse(k: u64, precision: u64) -> Big {
    let k = Big::from_u64(k);
    let k_squared = k.multiply(&k);
    let mut power = Big::from_u64(1).shift_left(precision).divide_truncating(&k).map(|(q, _)| q).unwrap_or_default();
    let mut sum = Big::zero();
    let mut n = 0u64;
    while !power.is_zero() {
        let term = power.divide_truncating(&Big::from_u64(2 * n + 1)).map(|(q, _)| q).unwrap_or_default();
        sum = if n.is_multiple_of(2) { sum.add(&term) } else { sum.subtract(&term) };
        power = power.divide_truncating(&k_squared).map(|(q, _)| q).unwrap_or_default();
        n += 1;
    }
    sum
}

/// π, kept at the most precision asked for so far.
static PI: Mutex<Option<(u64, Big)>> = Mutex::new(None);
/// ln 2, likewise.
static LN2: Mutex<Option<(u64, Big)>> = Mutex::new(None);

/// A constant at `precision` bits, from what's kept or worked out anew at more precision. It's wrong by less than 2
/// units of its last place.
fn constant(kept: &Mutex<Option<(u64, Big)>>, precision: u64, work: impl Fn(u64) -> Big) -> Big {
    let mut slot = kept.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((held, value)) = slot.as_ref()
        && *held >= precision
    {
        return value.shift_right(held - precision);
    }
    // Worked out with room to spare: each term's rounding costs one unit at the working precision.
    let working = precision.max(256) + 64;
    let value = work(working);
    let answer = value.shift_right(working - precision);
    *slot = Some((working, value));
    answer
}

/// π as a fixed-point number: Machin's formula, 16 atan(1/5) − 4 atan(1/239).
fn pi(precision: u64) -> Big {
    constant(&PI, precision, |working| {
        arctan_of_inverse(5, working).shift_left(4).subtract(&arctan_of_inverse(239, working).shift_left(2))
    })
}

/// ln 2 as a fixed-point number: 2 atanh(1/3) = 2 Σ 1 / ((2n+1) 3^(2n+1)).
fn ln2(precision: u64) -> Big {
    constant(&LN2, precision, |working| {
        let nine = Big::from_u64(9);
        let mut power = Big::from_u64(1)
            .shift_left(working)
            .divide_truncating(&Big::from_u64(3))
            .map(|(q, _)| q)
            .unwrap_or_default();
        let mut sum = Big::zero();
        let mut n = 0u64;
        while !power.is_zero() {
            sum = sum.add(&power.divide_truncating(&Big::from_u64(2 * n + 1)).map(|(q, _)| q).unwrap_or_default());
            power = power.divide_truncating(&nine).map(|(q, _)| q).unwrap_or_default();
            n += 1;
        }
        sum.shift_left(1)
    })
}

/// sin and cos of a fixed-point `r` with |r| at most about π/4, by their Taylor series, each wrong by at most
/// about as many units of the last place as there are terms.
fn sine_and_cosine(r: &Big, precision: u64) -> (Big, Big, u64) {
    let one = Big::from_u64(1).shift_left(precision);
    let r_squared = multiply(r, r, precision);
    let mut sine = r.clone();
    let mut cosine = one.clone();
    let mut sine_term = r.clone();
    let mut cosine_term = one;
    let mut n = 1u64;
    let mut terms = 0u64;
    while !sine_term.is_zero() || !cosine_term.is_zero() {
        // term(n) = term(n-1) × r² / ((2n)(2n+1)) for sin, and / ((2n-1)(2n)) for cos, alternating.
        cosine_term = multiply(&cosine_term, &r_squared, precision)
            .divide_truncating(&Big::from_u64((2 * n - 1) * (2 * n)))
            .map(|(q, _)| q)
            .unwrap_or_default()
            .negated();
        sine_term = multiply(&sine_term, &r_squared, precision)
            .divide_truncating(&Big::from_u64((2 * n) * (2 * n + 1)))
            .map(|(q, _)| q)
            .unwrap_or_default()
            .negated();
        sine = sine.add(&sine_term);
        cosine = cosine.add(&cosine_term);
        n += 1;
        terms += 1;
    }
    (sine, cosine, 3 * terms + 4)
}

/// `x` less the nearest multiple of π/2, as a fixed-point number, and which multiple it was, modulo 4.
fn reduce_by_half_pi(x: f64, precision: u64) -> (Big, u64) {
    let (m, e) = decompose(x);
    // π/2 needs as many more bits as `x` has before its point, so `k × π/2` is still exact to `precision`.
    let lead = (e + m.bits() as i64).max(0) as u64;
    let working = precision + lead + 8;
    let half_pi = pi(working).shift_right(1);
    let x_fixed = fixed(&m, e, working);
    let (k, _) = x_fixed.add(&half_pi.shift_right(1)).divide_floor(&half_pi).unwrap_or_default();
    let r = x_fixed.subtract(&k.multiply(&half_pi)).shift_right(working - precision);
    let quadrant = k.divide_floor(&Big::from_u64(4)).map(|(_, rest)| rest).unwrap_or_default().to_i64().unwrap_or(0);
    (r, quadrant as u64)
}

/// The precision to work at so that a result about `2^lead` keeps `precision` bits of its own.
fn relative(precision: u64, lead: i64) -> u64 {
    precision + (-lead).max(0) as u64
}

fn sine_cosine_at(x: f64, precision: u64, cosine_wanted: bool) -> Approximation {
    let (_, e) = decompose(x);
    let lead = e + decompose(x).0.bits() as i64;
    let working = relative(precision, lead.min(0)) + 16;
    let (r, quadrant) = reduce_by_half_pi(x, working);
    let (sine, cosine, error) = sine_and_cosine(&r, working);
    let pick = (quadrant + u64::from(cosine_wanted)) % 4;
    let approx = match pick {
        0 => sine,
        1 => cosine,
        2 => sine.negated(),
        _ => cosine.negated(),
    };
    Approximation { approx, exponent: -(working as i64), error: error + 4 }
}

/// sin, correctly rounded (D109).
#[must_use]
pub fn sin(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return if x.is_finite() { x } else { f64::NAN };
    }
    correctly_rounded(|precision| sine_cosine_at(x, precision, false))
}

/// cos, correctly rounded (D109).
#[must_use]
pub fn cos(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x == 0.0 {
        return 1.0;
    }
    correctly_rounded(|precision| sine_cosine_at(x, precision, true))
}

/// tan, correctly rounded (D109). It's never infinite, since no `Float` is an odd multiple of π/2.
#[must_use]
pub fn tan(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return if x.is_finite() { x } else { f64::NAN };
    }
    correctly_rounded(|precision| {
        let lead = decompose(x).1 + decompose(x).0.bits() as i64;
        let working = relative(precision, lead.min(0)) + 32;
        let (r, quadrant) = reduce_by_half_pi(x, working);
        let (sine, cosine, error) = sine_and_cosine(&r, working);
        // tan(r + kπ/2) is tan r for even k and −1/tan r = −cos r / sin r for odd k.
        let (top, bottom) = if quadrant % 2 == 0 { (sine, cosine) } else { (cosine.negated(), sine) };
        let quotient = divide(&top, &bottom, working);
        // Dividing by a small number magnifies the error; it's bounded by the bottom's size.
        let bottom_bits = bottom.abs().bits() as i64;
        let magnification = (working as i64 - bottom_bits + 2).clamp(0, 62) as u32;
        Approximation {
            approx: quotient,
            exponent: -(working as i64),
            error: (error + 4).saturating_mul(4u64.saturating_mul(1 << magnification)).saturating_add(4),
        }
    })
}

/// atan of a fixed-point `z`, any size: for |z| > 1 it's ±π/2 − atan(1/z); otherwise the argument is halved with
/// atan(z) = 2 atan(z / (1 + √(1 + z²))) until it's small, then summed as Σ (-1)^n z^(2n+1) / (2n+1).
fn arctan(z: &Big, precision: u64) -> (Big, u64) {
    let one = Big::from_u64(1).shift_left(precision);
    if z.abs().compare(&one) == Ordering::Greater {
        let inverse = divide(&one, z, precision);
        let (inner, error) = arctan(&inverse, precision);
        let half_pi = pi(precision).shift_right(1);
        let side = if z.is_negative() { half_pi.negated() } else { half_pi };
        return (side.subtract(&inner), error + 4);
    }
    let mut z = z.clone();
    let mut halvings = 0u32;
    let small = Big::from_u64(1).shift_left(precision.saturating_sub(3));
    while z.abs().compare(&small) == Ordering::Greater && halvings < 8 {
        let root = square_root(&one.add(&multiply(&z, &z, precision)), precision);
        z = divide(&z, &one.add(&root), precision);
        halvings += 1;
    }
    let z_squared = multiply(&z, &z, precision);
    let mut power = z.clone();
    let mut sum = Big::zero();
    let mut n = 0u64;
    while !power.is_zero() {
        let term = power.divide_truncating(&Big::from_u64(2 * n + 1)).map(|(q, _)| q).unwrap_or_default();
        sum = if n.is_multiple_of(2) { sum.add(&term) } else { sum.subtract(&term) };
        power = multiply(&power, &z_squared, precision);
        n += 1;
    }
    (sum.shift_left(u64::from(halvings)), (2 * n + 8) << halvings)
}

/// atan2(y, x): the angle of the point (x, y), correctly rounded (D109). The point (0, 0) has no direction and gives 0,
/// and a negative zero counts as zero, where IEEE 754 would give ±π (D133).
#[must_use]
pub fn atan2(y: f64, x: f64) -> f64 {
    // trace: D133
    let (y, x) = (if y == 0.0 { 0.0 } else { y }, if x == 0.0 { 0.0 } else { x });
    atan2_of(y, x).unwrap_or(0.0)
}

fn atan2_of(y: f64, x: f64) -> Option<f64> {
    if y == 0.0 && x == 0.0 {
        return None;
    }
    if y == 0.0 && x > 0.0 {
        return Some(y);
    }
    Some(correctly_rounded(|precision| {
        let (ym, ye) = decompose(y);
        let (xm, xe) = decompose(x);
        let working = precision + 64 + (ye - xe).unsigned_abs().min(2200);
        let angle = if x == 0.0 {
            let half_pi = pi(working).shift_right(1);
            (if y < 0.0 { half_pi.negated() } else { half_pi }, 4)
        } else {
            // y / x exactly as a fixed-point number: (ym / xm) × 2^(ye − xe).
            let ratio =
                fixed(&ym, ye - xe, working + xm.bits()).divide_truncating(&xm).map(|(q, _)| q).unwrap_or_default();
            let ratio = fixed(&ratio, -(xm.bits() as i64), 0);
            let (inner, error) = arctan(&ratio, working);
            if x > 0.0 {
                (inner, error)
            } else {
                let pi_value = pi(working);
                (if y < 0.0 { inner.subtract(&pi_value) } else { inner.add(&pi_value) }, error + 4)
            }
        };
        Approximation { approx: angle.0, exponent: -(working as i64), error: angle.1 + 8 }
    }))
}

/// asin, correctly rounded (D109); nothing outside -1 to 1.
#[must_use]
pub fn asin(x: f64) -> Option<f64> {
    if !(-1.0..=1.0).contains(&x) {
        return None;
    }
    if x == 0.0 {
        return Some(x);
    }
    Some(correctly_rounded(|precision| {
        let lead = decompose(x).1 + decompose(x).0.bits() as i64;
        let working = relative(precision, lead.min(0)) + 64;
        let (m, e) = decompose(x);
        let value = fixed(&m, e, working);
        let one = Big::from_u64(1).shift_left(working);
        let rest = one.subtract(&multiply(&value, &value, working));
        if rest.is_zero() || rest.is_negative() {
            let half_pi = pi(working).shift_right(1);
            return Approximation {
                approx: if x < 0.0 { half_pi.negated() } else { half_pi },
                exponent: -(working as i64),
                error: 4,
            };
        }
        let root = square_root(&rest, working);
        let (angle, error) = arctan(&divide(&value, &root, working), working);
        Approximation { approx: angle, exponent: -(working as i64), error: error.saturating_mul(4).saturating_add(64) }
    }))
}

/// acos, correctly rounded (D109); nothing outside -1 to 1.
#[must_use]
pub fn acos(x: f64) -> Option<f64> {
    if !(-1.0..=1.0).contains(&x) {
        return None;
    }
    if x == 1.0 {
        return Some(0.0);
    }
    Some(correctly_rounded(|precision| {
        let working = precision * 2 + 64;
        let (m, e) = decompose(x);
        let value = fixed(&m, e, working);
        let one = Big::from_u64(1).shift_left(working);
        let rest = one.subtract(&multiply(&value, &value, working));
        let root = square_root(&rest.abs(), working);
        // acos x = atan2(√(1 − x²), x), between 0 and π.
        let angle = if x == 0.0 {
            (pi(working).shift_right(1), 4)
        } else {
            let (inner, error) = arctan(&divide(&root, &value, working), working);
            if x > 0.0 { (inner, error) } else { (inner.add(&pi(working)), error + 4) }
        };
        Approximation {
            approx: angle.0,
            exponent: -(working as i64),
            error: angle.1.saturating_mul(4).saturating_add(64),
        }
    }))
}

/// ln of a positive `m × 2^e` as a fixed-point number: e ln 2 + ln(m'), m' scaled into [2/3, 4/3], by
/// 2 atanh((m' − 1)/(m' + 1)).
fn natural_log(m: &Big, e: i64, precision: u64) -> (Big, u64) {
    let bits = m.bits() as i64;
    // m × 2^e = m' × 2^k with m' = m / 2^(bits) in [1/2, 1), moved to [2/3, 4/3).
    let mut k = e + bits;
    let mut scaled = fixed(m, -bits, precision);
    let two_thirds =
        Big::from_u64(2).shift_left(precision).divide_truncating(&Big::from_u64(3)).map(|(q, _)| q).unwrap_or_default();
    if scaled.compare(&two_thirds) == Ordering::Less {
        scaled = scaled.shift_left(1);
        k -= 1;
    }
    let one = Big::from_u64(1).shift_left(precision);
    let z = divide(&scaled.subtract(&one), &scaled.add(&one), precision);
    let z_squared = multiply(&z, &z, precision);
    let mut power = z;
    let mut sum = Big::zero();
    let mut n = 0u64;
    while !power.is_zero() {
        sum = sum.add(&power.divide_truncating(&Big::from_u64(2 * n + 1)).map(|(q, _)| q).unwrap_or_default());
        power = multiply(&power, &z_squared, precision);
        n += 1;
    }
    let log = sum.shift_left(1).add(&ln2(precision).multiply(&Big::from_i64(k)));
    (log, 2 * n + 8 + 2 * k.unsigned_abs())
}

/// e^t for a fixed-point `t`: t = k ln 2 + r, and e^r by its Taylor series after halving r eight times and squaring
/// back. Gives the answer as `approx × 2^exponent`.
fn exponential(t: &Big, precision: u64) -> Approximation {
    let log2 = ln2(precision);
    let (k, _) = t.add(&log2.shift_right(1)).divide_floor(&log2).unwrap_or_default();
    let r = t.subtract(&k.multiply(&log2)).shift_right(8);
    let one = Big::from_u64(1).shift_left(precision);
    let mut sum = one.clone();
    let mut term = one;
    let mut n = 1u64;
    while !term.is_zero() {
        term = multiply(&term, &r, precision).divide_truncating(&Big::from_u64(n)).map(|(q, _)| q).unwrap_or_default();
        sum = sum.add(&term);
        n += 1;
    }
    for _ in 0..8 {
        sum = multiply(&sum, &sum, precision);
    }
    let k = k.to_i64().unwrap_or(0);
    Approximation {
        approx: sum,
        exponent: k - precision as i64,
        error: (n + 4 + k.unsigned_abs().saturating_mul(4)).saturating_mul(1 << 10),
    }
}

/// `x ** y` exactly as `m × 2^e`, when it's a `Float` or a tie between two: `y` a whole number, or `x` a perfect
/// power for `y`'s fraction, as `4.0 ** 0.5` (D108).
fn exact_power(x: f64, y: f64) -> Option<(Big, i64)> {
    let (m, e) = decompose(x.abs());
    let (ym, ye) = decompose(y);
    if ye >= 0 {
        // A whole power: exact when it's positive, or when x is a power of two.
        let n = ym.to_i64()?.checked_mul(1i64.checked_shl(ye as u32)?)?;
        if n > 0 {
            if m.bits() as i64 * n > 4096 {
                return None;
            }
            return Some((m.power(n as u64), e.checked_mul(n)?));
        }
        return (m.compare(&Big::from_u64(1)) == Ordering::Equal).then(|| Some((m, e.checked_mul(n)?))).flatten();
    }
    // y = a / 2^k: exact when x's whole number is a perfect 2^k-th power and its exponent divides by 2^k.
    let k = (-ye) as u32;
    if k > 5 {
        return (m.compare(&Big::from_u64(1)) == Ordering::Equal && (e * ym.to_i64()?) % (1 << k) == 0)
            .then(|| Some((Big::from_u64(1), e * ym.to_i64()? / (1 << k))))
            .flatten();
    }
    if e % (1 << k) != 0 {
        return None;
    }
    let mut root = m;
    for _ in 0..k {
        let next = root.square_root_floor();
        if next.multiply(&next) != root {
            return None;
        }
        root = next;
    }
    let a = ym.to_i64()?;
    if a <= 0 && root.compare(&Big::from_u64(1)) != Ordering::Equal {
        return None;
    }
    if root.bits() as i64 * a.abs() > 4096 {
        return None;
    }
    Some((root.power(a.unsigned_abs()), (e >> k) * a))
}

/// `x ** y` on two `Float`s, correctly rounded, with IEEE 754's answers where x is 0, 1 or negative; not-a-number
/// or infinite where there's no real or finite answer, which the float check then stops on (D18, D108).
#[must_use]
pub fn power(x: f64, y: f64) -> f64 {
    if y == 0.0 || x == 1.0 {
        return 1.0;
    }
    if !x.is_finite() || !y.is_finite() {
        return f64::NAN;
    }
    let y_whole = y.fract() == 0.0;
    let odd = y_whole && y.abs() < 9_007_199_254_740_992.0 && (y as i64) % 2 != 0;
    if x == 0.0 {
        let magnitude = if y > 0.0 { 0.0 } else { f64::INFINITY };
        return if odd && x.is_sign_negative() { -magnitude } else { magnitude };
    }
    if x < 0.0 && !y_whole {
        return f64::NAN;
    }
    let sign = if x < 0.0 && odd { -1.0 } else { 1.0 };
    if let Some((m, e)) = exact_power(x, y) {
        return sign * round_dyadic(&m, e);
    }
    let (m, e) = decompose(x.abs());
    let (ym, ye) = decompose(y);
    // Past these, the answer is surely infinite or zero: |y × log2 x| beyond 1100.
    let log2_size = (e + m.bits() as i64 - 1) as f64;
    if (log2_size.abs() + 1.0) * y.abs() > 2200.0 && (log2_size * y > 1100.0 || log2_size * y < -1200.0) {
        return sign * if log2_size * y > 0.0 { f64::INFINITY } else { 0.0 };
    }
    sign * correctly_rounded(|precision| {
        let working = precision + 64;
        let (log, error) = natural_log(&m, e, working + 32);
        // t = y × ln x, exactly as y is m × 2^e.
        let t = fixed(&log.multiply(&ym), ye, 0).shift_right(32);
        let mut answer = exponential(&t, working);
        let y_size = ym.bits() as i64 + ye;
        answer.error =
            answer.error.saturating_add(error.saturating_mul(1 << y_size.clamp(0, 40)) >> 16).saturating_add(64);
        answer
    })
}

/// `lerp(a, to: b, by: t)`, a + t × (b − a), worked out exactly and rounded once (D114).
#[must_use]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    let (am, ae) = decompose(a);
    let (bm, be) = decompose(b);
    let (tm, te) = decompose(t);
    // Every value as a whole number over a common power of two.
    let low = ae.min(be).min(ae.min(be) + te);
    let a_whole = fixed(&am, ae - low, 0);
    let difference = fixed(&bm, be - ae.min(be), 0).subtract(&fixed(&am, ae - ae.min(be), 0));
    let product = difference.multiply(&tm);
    let product_exponent = ae.min(be) + te;
    let total = a_whole.add(&fixed(&product, product_exponent - low, 0));
    round_dyadic(&total, low)
}

/// The exact rational values of sin at multiples of 30°, by how many twelfths of a turn: `Some` where sin is
/// rational (Niven's theorem), as the `Float` it is.
const SINE_OF_TWELFTHS: [Option<f64>; 12] =
    [Some(0.0), Some(0.5), None, Some(1.0), None, Some(0.5), Some(0.0), Some(-0.5), None, Some(-1.0), None, Some(-0.5)];

/// sin, cos or tan (`which` 0, 1 or 2) of an angle of `x` units, `per_turn` to a full turn: 360 for `degs` and 1
/// for `turns` (D113). The angle is worked out as an exact fraction of a turn, so `sin(90 degs)` is exactly 1.0, and
/// the rest is correctly rounded (D50). `tan` of a quarter turn is infinite, which the float check stops on.
#[must_use]
pub fn angle_function(which: u8, x: f64, per_turn: u64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let (m, e) = decompose(x);
    // Twelfths of a turn, when the angle is a whole number of them: m × 2^e × 12 / per_turn. m is odd, so m × 12 ×
    // 2^e is whole only when e is at least -2.
    let twelve_m = m.multiply(&Big::from_u64(12));
    let whole =
        if e >= 0 { Some(fixed(&twelve_m, e, 0)) } else { (-e <= 2).then(|| twelve_m.shift_right((-e) as u64)) };
    let twelfths = whole.and_then(|whole| {
        let (quotient, remainder) = whole.divide_floor(&Big::from_u64(per_turn))?;
        remainder
            .is_zero()
            .then(|| quotient.divide_floor(&Big::from_u64(12)).map(|(_, rest)| rest.to_i64().unwrap_or(0)))?
    });
    if let Some(k) = twelfths {
        let sine = |k: i64| SINE_OF_TWELFTHS.get(k.rem_euclid(12) as usize).copied().flatten();
        match which {
            0 => {
                if let Some(value) = sine(k) {
                    return value;
                }
            }
            1 => {
                if let Some(value) = sine(k + 3) {
                    return value;
                }
            }
            _ => {
                let k = k.rem_euclid(12);
                if k % 3 == 0 {
                    // tan at multiples of 45°: 0, 1, infinite, -1 by eighths; at twelfths only the quarters are these.
                    return [0.0, f64::INFINITY, 0.0, f64::INFINITY][(k / 3) as usize];
                }
            }
        }
    }
    // tan at 45°, 135° and their like, which aren't twelfths.
    if which == 2 {
        let eighths = if e >= 0 {
            Some(fixed(&m.multiply(&Big::from_u64(8)), e, 0))
        } else {
            (-e <= 3).then(|| m.multiply(&Big::from_u64(8)).shift_right((-e) as u64))
        };
        if let Some(eighths) = eighths
            && let Some((quotient, remainder)) = eighths.divide_floor(&Big::from_u64(per_turn))
            && remainder.is_zero()
        {
            let k = quotient.divide_floor(&Big::from_u64(8)).map_or(0, |(_, rest)| rest.to_i64().unwrap_or(0));
            return [0.0, 1.0, f64::INFINITY, -1.0, 0.0, 1.0, f64::INFINITY, -1.0][k.rem_euclid(8) as usize];
        }
    }
    correctly_rounded(|precision| {
        let lead = e + m.bits() as i64 - 8;
        let working = relative(precision, lead.min(0)) + 32;
        // The angle's fraction of a turn, in fixed point, and what's left after whole quarter turns.
        let fraction = fixed(&m, e, working)
            .divide_floor(&Big::from_u64(per_turn))
            .map(|(quotient, _)| quotient)
            .unwrap_or_default();
        let one = Big::from_u64(1).shift_left(working);
        let (_, within) = fraction.divide_floor(&one).unwrap_or_default();
        let quarter = one.shift_right(2);
        let (quadrant, rest) = within.divide_floor(&quarter).unwrap_or_default();
        let quadrant = quadrant.to_i64().unwrap_or(0) as u64;
        // Centred on the nearest quarter turn, so the series stays short and tan's division is safe.
        let (quadrant, rest) = if rest.compare(&quarter.shift_right(1)) == Ordering::Greater {
            ((quadrant + 1) % 4, rest.subtract(&quarter))
        } else {
            (quadrant, rest)
        };
        let two_pi = pi(working).shift_left(1);
        let r = multiply(&rest, &two_pi, working);
        let (sine, cosine, error) = sine_and_cosine(&r, working);
        let approx = match which {
            0 => [sine.clone(), cosine.clone(), sine.negated(), cosine.negated()][quadrant as usize % 4].clone(),
            1 => [cosine.clone(), sine.negated(), cosine.negated(), sine.clone()][quadrant as usize % 4].clone(),
            _ => {
                let (top, bottom) = if quadrant % 2 == 0 { (sine, cosine) } else { (cosine.negated(), sine) };
                let bottom_bits = bottom.abs().bits() as i64;
                let magnification = (working as i64 - bottom_bits + 2).clamp(0, 62) as u32;
                return Approximation {
                    approx: divide(&top, &bottom, working),
                    exponent: -(working as i64),
                    error: (error + 8).saturating_mul(4u64.saturating_mul(1 << magnification)).saturating_add(8),
                };
            }
        };
        Approximation { approx, exponent: -(working as i64), error: error + 8 }
    })
}

/// `x × numerator / denominator × π^pi` for a whole `x`, correctly rounded: a unit conversion (D120).
#[must_use]
pub fn scale_whole(x: i64, numerator: u128, denominator: u128, pi: i8) -> f64 {
    scale(&Big::from_i64(x), 0, numerator, denominator, pi)
}

/// `x × numerator / denominator × π^pi` for a `Float` `x`, correctly rounded: a unit conversion (D120).
#[must_use]
pub fn scale_float(x: f64, numerator: u128, denominator: u128, pi: i8) -> f64 {
    if x == 0.0 {
        return x;
    }
    let (m, e) = decompose(x);
    scale(&m, e, numerator, denominator, pi)
}

/// `m × 2^e × numerator / denominator × π^power`: exact and rounded once without π, and with it, worked out by
/// Ziv's strategy, as π's powers are never a tie.
fn scale(m: &Big, e: i64, numerator: u128, denominator: u128, power: i8) -> f64 {
    let product = m.multiply(&Big::from_u128(numerator));
    if power == 0 {
        return divide_rounded(&product, e, denominator);
    }
    if product.is_zero() {
        return 0.0;
    }
    let negative = product.is_negative();
    let magnitude = if negative { product.negated() } else { product };
    let times = u64::from(power.unsigned_abs());
    let denominator = Big::from_u128(denominator);
    let answer = correctly_rounded(|precision| {
        let room = precision + denominator.bits();
        let mut approx = magnitude.shift_left(room);
        let pi_fixed = pi(precision + 8);
        for _ in 0..times {
            approx = if power > 0 {
                approx.multiply(&pi_fixed).shift_right(precision + 8)
            } else {
                approx.shift_left(precision + 8).divide_truncating(&pi_fixed).map(|(q, _)| q).unwrap_or_default()
            };
        }
        let approx = approx.divide_truncating(&denominator).map(|(q, _)| q).unwrap_or_default();
        // Each step is wrong by under 2^-(precision + 6) of the value and one unit; kept to `precision` bits, the
        // whole error is a few units.
        let shift = approx.bits().saturating_sub(precision);
        Approximation {
            approx: approx.shift_right(shift),
            exponent: e - room as i64 + shift as i64,
            error: 2 * times + 4,
        }
    });
    if negative { -answer } else { answer }
}

/// `m × 2^e / denominator`, rounded once: the quotient is worked out with 140 more bits than a `Float` keeps, and a
/// remainder makes the last bit sticky, so rounding it is rounding the exact value.
fn divide_rounded(m: &Big, e: i64, denominator: u128) -> f64 {
    if denominator == 1 {
        return round_dyadic(m, e);
    }
    let extra = 140 + Big::from_u128(denominator).bits();
    let scaled = m.shift_left(extra);
    let Some((quotient, remainder)) = scaled.divide_truncating(&Big::from_u128(denominator)) else { return f64::NAN };
    // A remainder past the kept bits only ever nudges a value that isn't a tie, so one sticky bit is enough.
    let sticky = if remainder.is_zero() {
        quotient
    } else if quotient.is_negative() {
        quotient.shift_left(1).subtract(&Big::from_u64(1))
    } else {
        quotient.shift_left(1).add(&Big::from_u64(1))
    };
    let shift = if remainder.is_zero() { extra as i64 } else { extra as i64 + 1 };
    round_dyadic(&sticky, e - shift)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ulps(a: f64, b: f64) -> u64 {
        a.to_bits().abs_diff(b.to_bits())
    }

    #[test]
    fn rounding_a_dyadic_number() {
        assert_eq!(round_dyadic(&Big::from_u64(3), -1), 1.5);
        assert_eq!(round_dyadic(&Big::from_u64(1), -1074), f64::from_bits(1));
        assert_eq!(round_dyadic(&Big::from_u64(1), -1075), 0.0);
        assert_eq!(round_dyadic(&Big::from_u64(3), -1076), f64::from_bits(1));
        assert_eq!(round_dyadic(&Big::from_u64(1), 1024), f64::INFINITY);
        // 2^53 + 1 ties to even, 2^53.
        assert_eq!(round_dyadic(&Big::from_u64((1 << 53) + 1), 0), 9_007_199_254_740_992.0);
        assert_eq!(round_dyadic(&Big::from_u64((1 << 53) + 3), 0), 9_007_199_254_740_996.0);
    }

    #[test]
    fn known_answers() {
        assert_eq!(sin(1.0), 0.841_470_984_807_896_5);
        assert_eq!(cos(1.0), 0.540_302_305_868_139_8);
        assert_eq!(tan(1.0), 1.557_407_724_654_902_3);
        assert_eq!(atan2(1.0, 1.0), std::f64::consts::FRAC_PI_4);
        assert_eq!(atan2(1.0, 0.0), std::f64::consts::FRAC_PI_2);
        assert_eq!(atan2(0.0, -1.0), std::f64::consts::PI);
        assert_eq!(atan2(-0.0, -1.0), std::f64::consts::PI);
        assert_eq!(atan2(0.0, 0.0), 0.0);
        assert_eq!(atan2(-0.0, -0.0), 0.0);
        assert_eq!(asin(1.0), Some(std::f64::consts::FRAC_PI_2));
        assert_eq!(acos(-1.0), Some(std::f64::consts::PI));
        assert_eq!(acos(1.0), Some(0.0));
        assert_eq!(asin(1.5), None);
        assert_eq!(power(2.0, 0.5), std::f64::consts::SQRT_2);
        assert_eq!(power(4.0, 0.5), 2.0);
        assert_eq!(power(2.0, 10.0), 1024.0);
        assert_eq!(power(2.0, -2.0), 0.25);
        assert_eq!(power(-2.0, 3.0), -8.0);
        assert!(power(-2.0, 0.5).is_nan());
        assert_eq!(power(10.0, -2.0), 0.01);
        assert_eq!(power(2.0, 2000.0), f64::INFINITY);
        assert_eq!(lerp(0.0, 10.0, 0.25), 2.5);
        assert_eq!(lerp(1.0, 3.0, 1.0), 3.0);
        assert_eq!(lerp(0.1, 0.2, 0.0), 0.1);
        assert_eq!(angle_function(0, 90.0, 360), 1.0);
        assert_eq!(angle_function(0, 30.0, 360), 0.5);
        assert_eq!(angle_function(1, 60.0, 360), 0.5);
        assert_eq!(angle_function(1, 90.0, 360), 0.0);
        assert_eq!(angle_function(2, 45.0, 360), 1.0);
        assert_eq!(angle_function(2, 90.0, 360), f64::INFINITY);
        assert_eq!(angle_function(0, 0.25, 1), 1.0);
        assert_eq!(angle_function(0, 1.0, 360), (1.0f64).to_radians().sin());
        assert_eq!(scale_whole(2500, 1, 1000, 0), 2.5);
        assert_eq!(scale_float(1.0, 1, 3, 0), 1.0 / 3.0);
        // A radian in degrees, and 90 degrees in radians, through an exact π.
        assert_eq!(scale_float(1.0, 180, 1, -1), 57.295_779_513_082_32);
        assert_eq!(scale_whole(90, 1, 180, 1), std::f64::consts::FRAC_PI_2);
        assert_eq!(scale_float(-0.5, 2, 1, 1), -std::f64::consts::PI);
    }

    #[test]
    fn close_to_the_platform_everywhere() {
        // The platform's own functions are within an ulp of the right answer nearly everywhere, so the correctly
        // rounded answers are rarely more than one away.
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        for _ in 0..300 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let x = (seed >> 11) as f64 / (1u64 << 53) as f64 * 20.0 - 10.0;
            assert!(ulps(sin(x), x.sin()) <= 1, "sin({x})");
            assert!(ulps(cos(x), x.cos()) <= 1, "cos({x})");
            // The platform's own tan is off by 2 here and there, as at -7.59421142430311, checked by hand.
            assert!(ulps(tan(x), x.tan()) <= 2, "tan({x})");
            let y = x / 10.0;
            assert!(ulps(asin(y).unwrap_or(0.0), y.asin()) <= 1, "asin({y})");
            assert!(ulps(acos(y).unwrap_or(0.0), y.acos()) <= 1, "acos({y})");
            assert!(ulps(atan2(x, y + 0.5), x.atan2(y + 0.5)) <= 1, "atan2({x}, {y})");
            let base = x.abs() + 0.01;
            assert!(ulps(power(base, y * 3.0), base.powf(y * 3.0)) <= 1, "{base} ** {}", y * 3.0);
        }
        assert!(ulps(sin(1.0e22), 1.0e22f64.sin()) <= 1);
    }
}

#[cfg(test)]
mod dump {
    /// Prints inputs and answers for checking by hand against an independent calculation.
    #[test]
    #[ignore = "run by hand: cargo test -p wax-lang --lib dump -- --ignored --nocapture"]
    fn dump_answers() {
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        for _ in 0..100 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let x = (seed >> 11) as f64 / (1u64 << 53) as f64 * 40.0 - 20.0;
            let y = x / 21.0;
            let b = x.abs() / 3.0 + 0.1;
            println!("sin {:e} {:e}", x, super::sin(x));
            println!("cos {:e} {:e}", x, super::cos(x));
            println!("tan {:e} {:e}", x, super::tan(x));
            println!("asin {:e} {:e}", y, super::asin(y).unwrap_or(0.0));
            println!("atan2 {:e} {:e} {:e}", x, y, super::atan2(x, y));
            println!("pow {:e} {:e} {:e}", b, y * 5.0, super::power(b, y * 5.0));
            println!("sindeg {:e} {:e}", x * 37.0, super::angle_function(0, x * 37.0, 360));
            println!("costurn {:e} {:e}", y, super::angle_function(1, y, 1));
            println!("tandeg {:e} {:e}", x * 9.0, super::angle_function(2, x * 9.0, 360));
        }
    }
}

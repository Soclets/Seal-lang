//! `Decimal`, IEEE 754's decimal128 (D116, D124, D125, D127), and `HugeDecimal`, an exact decimal of any size (D116,
//! D127). Each is a whole number times a power of ten, so `0.1` is exactly a tenth.
//!
//! A `Decimal` keeps at most 34 digits, with its power of ten from -6176 to 6111, and rounds past them to the
//! nearest, a tie to the even one. Each operation keeps the power of ten IEEE 754 prefers for it, which is how
//! `1.50 + 1.50` comes out as `3.00` (D125). Everything is worked out exactly first and rounded once, so every answer
//! is the correctly rounded one. A `HugeDecimal` never rounds: what has no exact answer fails instead.
// The counts of digits and powers of ten here are far inside every type they're cast between.
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)]

use std::cmp::Ordering;

use crate::big::Big;
use crate::check::tree::Rounding;
use crate::maths::round_dyadic;

/// The most digits a `Decimal` keeps.
pub const DIGITS: u64 = 34;
/// The smallest power of ten a `Decimal`'s last digit can stand for.
pub const EXPONENT_MIN: i64 = -6176;
/// The largest.
pub const EXPONENT_MAX: i64 = 6111;
/// Past this many digits, a power is worked out by narrowing it down instead of exactly.
const EXACT_POWER_DIGITS: u64 = 4000;

/// Why a decimal operation has no answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberError {
    /// Dividing by zero.
    ByZero,
    /// A `HugeDecimal` or `HugeWhole` operation with no exact answer, as `1 / 3`.
    Inexact,
    /// Too big for the type.
    TooBig,
    /// A negative number's square root.
    Negative,
}

const POWERS_OF_TEN: [u128; 39] = {
    let mut powers = [1u128; 39];
    let mut index = 1;
    while index < 39 {
        powers[index] = powers[index - 1] * 10;
        index += 1;
    }
    powers
};

/// 10^power.
fn ten_to(power: u64) -> Big {
    match POWERS_OF_TEN.get(power as usize) {
        Some(&small) => Big::from_u128(small),
        None => Big::from_u64(10).power(power),
    }
}

/// How many digits a whole number's size has; 0 for zero.
#[must_use]
pub fn digit_count(value: &Big) -> u64 {
    if let Some(small) = value.to_u128_magnitude() {
        return POWERS_OF_TEN.iter().position(|&power| power > small).unwrap_or(39) as u64;
    }
    let magnitude = value.abs();
    // log10(2) is about 0.30103, so the count is one of two next to this; the powers of ten settle which.
    let mut count = (magnitude.bits() * 30_103 / 100_000).max(1);
    while magnitude.compare(&ten_to(count)) != Ordering::Less {
        count += 1;
    }
    while count > 1 && magnitude.compare(&ten_to(count - 1)) == Ordering::Less {
        count -= 1;
    }
    count
}

/// An exact decimal of any size: `coefficient × 10^exponent`, the coefficient carrying the sign (D116).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HugeDecimal {
    pub coefficient: Big,
    pub exponent: i64,
}

impl HugeDecimal {
    #[must_use]
    pub fn new(coefficient: Big, exponent: i64) -> HugeDecimal {
        HugeDecimal { coefficient, exponent }
    }

    #[must_use]
    pub fn from_whole(value: i64) -> HugeDecimal {
        HugeDecimal::new(Big::from_i64(value), 0)
    }

    #[must_use]
    pub fn from_big(value: Big) -> HugeDecimal {
        HugeDecimal::new(value, 0)
    }

    /// Reads a number written out: digits with a `.` between digits when it has a fraction, and `_` between
    /// digits, as a script writes a number, with no sign.
    #[must_use]
    pub fn read(text: &str) -> Option<HugeDecimal> {
        let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
        let clean = |part: &str| -> Option<String> {
            let digits: String = part.chars().filter(|&character| character != '_').collect();
            (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())).then_some(digits)
        };
        let whole = clean(whole)?;
        let fraction = if text.contains('.') { clean(fraction)? } else { String::new() };
        let coefficient = Big::parse(&format!("{whole}{fraction}"))?;
        Some(HugeDecimal::new(coefficient, -(fraction.len() as i64)))
    }

    /// Reads a number from text, as `be.Decimal` and `be.HugeDecimal` do (D126): spaces and tabs around it are
    /// ignored, a `+` or `-` in front is fine, and the number is digits with a `.` between digits when it has a
    /// fraction, as `be.Whole` reads a whole number (Unio D95, D143).
    #[must_use]
    pub fn from_text(text: &str) -> Option<HugeDecimal> {
        let trimmed = text.trim_matches(|character| character == ' ' || character == '\t');
        let (negative, rest) = match trimmed.as_bytes().first() {
            Some(b'-') => (true, trimmed.get(1..)?),
            Some(b'+') => (false, trimmed.get(1..)?),
            _ => (false, trimmed),
        };
        if rest.contains('_') {
            return None;
        }
        let value = HugeDecimal::read(rest)?;
        Some(if negative { value.negated() } else { value })
    }

    /// A `Float`'s exact value.
    #[must_use]
    pub fn from_float(value: f64) -> HugeDecimal {
        if value == 0.0 || !value.is_finite() {
            return HugeDecimal::default();
        }
        let bits = value.to_bits();
        let exponent_bits = ((bits >> 52) & 0x7ff) as i64;
        let fraction = bits & ((1 << 52) - 1);
        let (mantissa, power) =
            if exponent_bits == 0 { (fraction, -1074) } else { (fraction | (1 << 52), exponent_bits - 1075) };
        // An odd mantissa, so the decimal has no trailing zeros.
        let zeros = mantissa.trailing_zeros();
        let (mantissa, power) = (Big::from_u64(mantissa >> zeros), power + i64::from(zeros));
        let exact = if power >= 0 {
            HugeDecimal::new(mantissa.shift_left(power as u64), 0)
        } else {
            // m / 2^k = m × 5^k / 10^k.
            let k = power.unsigned_abs();
            HugeDecimal::new(mantissa.multiply(&Big::from_u64(5).power(k)), -(k as i64))
        };
        if value < 0.0 { exact.negated() } else { exact }
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.coefficient.is_zero()
    }

    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.coefficient.is_negative()
    }

    #[must_use]
    pub fn negated(&self) -> HugeDecimal {
        HugeDecimal::new(self.coefficient.negated(), self.exponent)
    }

    #[must_use]
    pub fn abs(&self) -> HugeDecimal {
        HugeDecimal::new(self.coefficient.abs(), self.exponent)
    }

    /// How many digits it holds: its size, for counting memory and work.
    #[must_use]
    pub fn digits(&self) -> u64 {
        digit_count(&self.coefficient)
    }

    /// The same amount with trailing zeros taken off its coefficient, while its exponent stays at most `towards`.
    #[must_use]
    pub fn trimmed(mut self, towards: i64) -> HugeDecimal {
        if self.coefficient.is_zero() {
            self.exponent = self.exponent.max(towards);
            return self;
        }
        let ten = Big::from_u64(10);
        while self.exponent < towards {
            match self.coefficient.divide_truncating(&ten) {
                Some((quotient, remainder)) if remainder.is_zero() => {
                    self.coefficient = quotient;
                    self.exponent += 1;
                }
                _ => break,
            }
        }
        self
    }

    /// Both coefficients at the smaller of the two exponents.
    fn aligned(&self, other: &HugeDecimal) -> (Big, Big, i64) {
        let exponent = self.exponent.min(other.exponent);
        let left = self.coefficient.multiply(&ten_to((self.exponent - exponent) as u64));
        let right = other.coefficient.multiply(&ten_to((other.exponent - exponent) as u64));
        (left, right, exponent)
    }

    #[must_use]
    pub fn add(&self, other: &HugeDecimal) -> HugeDecimal {
        let (left, right, exponent) = self.aligned(other);
        HugeDecimal::new(left.add(&right), exponent)
    }

    #[must_use]
    pub fn subtract(&self, other: &HugeDecimal) -> HugeDecimal {
        self.add(&other.negated())
    }

    #[must_use]
    pub fn multiply(&self, other: &HugeDecimal) -> HugeDecimal {
        HugeDecimal::new(self.coefficient.multiply(&other.coefficient), self.exponent + other.exponent)
    }

    /// Compares amounts, so `3.00` and `3.0` are the same (D125).
    #[must_use]
    pub fn compare(&self, other: &HugeDecimal) -> Ordering {
        let (left_negative, right_negative) = (self.is_negative(), other.is_negative());
        if left_negative != right_negative {
            return if left_negative { Ordering::Less } else { Ordering::Greater };
        }
        match (self.is_zero(), other.is_zero()) {
            (true, true) => return Ordering::Equal,
            (true, false) => return if right_negative { Ordering::Greater } else { Ordering::Less },
            (false, true) => return if left_negative { Ordering::Less } else { Ordering::Greater },
            (false, false) => {}
        }
        // Where the leading digits stand settles it unless they stand at the same place.
        let left_top = self.exponent + self.digits() as i64;
        let right_top = other.exponent + other.digits() as i64;
        if left_top != right_top {
            let bigger = if left_top > right_top { Ordering::Greater } else { Ordering::Less };
            return if left_negative { bigger.reverse() } else { bigger };
        }
        let (left, right, _) = self.aligned(other);
        left.compare(&right)
    }

    /// The exact quotient, with its exponent as near `self.exponent - other.exponent` as it can be (D116, Unio D70).
    ///
    /// # Errors
    ///
    /// `ByZero`, or `Inexact` when the quotient isn't a decimal with an end.
    pub fn divide(&self, other: &HugeDecimal) -> Result<HugeDecimal, NumberError> {
        if other.is_zero() {
            return Err(NumberError::ByZero);
        }
        let preferred = self.exponent - other.exponent;
        if self.is_zero() {
            return Ok(HugeDecimal::new(Big::zero(), preferred));
        }
        // The divisor's twos and fives can be made up with tens; whatever else is in it must divide the dividend.
        let (mut rest, mut twos, mut fives) = (other.coefficient.abs(), 0u64, 0u64);
        while !rest.is_zero() && !rest.is_odd() {
            rest = rest.shift_right(1);
            twos += 1;
        }
        let five = Big::from_u64(5);
        while let Some((quotient, remainder)) = rest.divide_truncating(&five) {
            if !remainder.is_zero() {
                break;
            }
            rest = quotient;
            fives += 1;
        }
        let Some((quotient, remainder)) = self.coefficient.divide_truncating(&rest) else {
            return Err(NumberError::ByZero);
        };
        if !remainder.is_zero() {
            return Err(NumberError::Inexact);
        }
        let tens = twos.max(fives);
        let mut quotient = quotient.shift_left(tens - twos).multiply(&five.power(tens - fives));
        if other.is_negative() {
            quotient = quotient.negated();
        }
        Ok(HugeDecimal::new(quotient, preferred - tens as i64).trimmed(preferred))
    }

    /// `div` and `rem` (D127): the whole number of times `other` goes into this, rounded down or towards zero, and
    /// what's left over, exactly.
    ///
    /// # Errors
    ///
    /// `ByZero`.
    pub fn divide_whole(&self, other: &HugeDecimal, floor: bool) -> Result<(Big, HugeDecimal), NumberError> {
        let (left, right, exponent) = self.aligned(other);
        let divided = if floor { left.divide_floor(&right) } else { left.divide_truncating(&right) };
        let (quotient, remainder) = divided.ok_or(NumberError::ByZero)?;
        Ok((quotient, HugeDecimal::new(remainder, exponent)))
    }

    /// Raised to a whole power, exactly; a negative power is one over it, which must have an end (D127).
    ///
    /// # Errors
    ///
    /// `TooBig` past `most_digits` digits; `ByZero` for zero to a negative power; `Inexact`.
    pub fn power(&self, power: i64, most_digits: u64) -> Result<HugeDecimal, NumberError> {
        let times = power.unsigned_abs();
        if self.digits().saturating_mul(times) > most_digits {
            return Err(NumberError::TooBig);
        }
        let exponent = self.exponent.checked_mul(times as i64).ok_or(NumberError::TooBig)?;
        let raised = HugeDecimal::new(self.coefficient.power(times), exponent);
        if power >= 0 { Ok(raised) } else { HugeDecimal::from_whole(1).divide(&raised) }
    }

    /// The exact square root, with its exponent as near half this one's as it can be (D127).
    ///
    /// # Errors
    ///
    /// `Negative`, or `Inexact` when the root has no end.
    pub fn square_root(&self) -> Result<HugeDecimal, NumberError> {
        if self.is_negative() {
            return Err(NumberError::Negative);
        }
        let ideal = self.exponent.div_euclid(2);
        let (coefficient, exponent) = if self.exponent.rem_euclid(2) == 1 {
            (self.coefficient.multiply(&Big::from_u64(10)), self.exponent - 1)
        } else {
            (self.coefficient.clone(), self.exponent)
        };
        let root = coefficient.square_root_floor();
        if root.multiply(&root) != coefficient {
            return Err(NumberError::Inexact);
        }
        Ok(HugeDecimal::new(root, exponent / 2).trimmed(ideal))
    }

    /// Rounded to a whole number (D126, Unio D178).
    #[must_use]
    pub fn to_whole(&self, rounding: Rounding) -> Big {
        if self.exponent >= 0 {
            return self.coefficient.multiply(&ten_to(self.exponent as u64));
        }
        let divisor = ten_to(self.exponent.unsigned_abs());
        let Some((floor, left)) = self.coefficient.divide_floor(&divisor) else { return Big::zero() };
        if left.is_zero() {
            return floor;
        }
        let one = Big::from_u64(1);
        let twice = left.shift_left(1);
        let half = twice.compare(&divisor);
        let up = match rounding {
            Rounding::Floor => false,
            Rounding::Ceil => true,
            Rounding::Trunc => self.is_negative(),
            Rounding::Nearest => half == Ordering::Greater || (half == Ordering::Equal && !self.is_negative()),
            Rounding::NearestEven => half == Ordering::Greater || (half == Ordering::Equal && floor.is_odd()),
        };
        if up { floor.add(&one) } else { floor }
    }

    /// The nearest `Float`, a tie to even; infinite when it's past a `Float`'s range.
    #[must_use]
    pub fn to_float(&self) -> f64 {
        if self.is_zero() {
            return 0.0;
        }
        let magnitude = self.coefficient.abs();
        let value = if self.exponent >= 0 {
            // Past this, it's far beyond a `Float` anyway.
            if self.exponent > 400 {
                return if self.is_negative() { f64::NEG_INFINITY } else { f64::INFINITY };
            }
            round_dyadic(&magnitude.multiply(&ten_to(self.exponent as u64)), 0)
        } else {
            if self.exponent < -400 - self.digits() as i64 {
                return if self.is_negative() { -0.0 } else { 0.0 };
            }
            let divisor = ten_to(self.exponent.unsigned_abs());
            let shift = (divisor.bits() + 70).saturating_sub(magnitude.bits());
            let Some((quotient, remainder)) = magnitude.shift_left(shift).divide_truncating(&divisor) else {
                return f64::NAN;
            };
            // One more bit, set when anything was left over, so the rounding sees past the kept bits.
            let sticky = quotient.shift_left(1).add(&Big::from_u64(u64::from(!remainder.is_zero())));
            round_dyadic(&sticky, -(shift as i64) - 1)
        };
        if self.is_negative() { -value } else { value }
    }

    /// As shown: its digits, with as many after the point as its exponent says, and no `~` (D124, D125).
    #[must_use]
    pub fn to_text(&self) -> String {
        show(self.is_negative(), &self.coefficient.abs().to_decimal(), self.exponent)
    }

    /// Shown to `places` digits after its point: rounded to the nearest, a tie away from zero, with no minus on
    /// zero (D94, D125).
    #[must_use]
    pub fn with_places(&self, places: u8) -> String {
        let target = -i64::from(places);
        let magnitude = self.coefficient.abs();
        let digits = if self.exponent >= target {
            magnitude.multiply(&ten_to((self.exponent - target) as u64))
        } else {
            let divisor = ten_to((target - self.exponent) as u64);
            let Some((quotient, remainder)) = magnitude.divide_truncating(&divisor) else { return String::new() };
            if remainder.shift_left(1).compare(&divisor) == Ordering::Less {
                quotient
            } else {
                quotient.add(&Big::from_u64(1))
            }
        };
        show(self.is_negative() && !digits.is_zero(), &digits.to_decimal(), target)
    }
}

/// A decimal written out in full: `digits × 10^exponent`, with no `e`.
fn show(negative: bool, digits: &str, exponent: i64) -> String {
    let zero = digits.bytes().all(|byte| byte == b'0');
    let mut text = String::with_capacity(digits.len() + 3);
    if negative && !zero {
        text.push('-');
    }
    if exponent >= 0 {
        text.push_str(digits);
        if !zero {
            text.extend(std::iter::repeat_n('0', exponent as usize));
        }
        return text;
    }
    let places = exponent.unsigned_abs() as usize;
    if digits.len() <= places {
        text.push_str("0.");
        text.extend(std::iter::repeat_n('0', places - digits.len()));
        text.push_str(digits);
    } else {
        let (whole, fraction) = digits.split_at(digits.len() - places);
        text.push_str(whole);
        text.push('.');
        text.push_str(fraction);
    }
    text
}

/// IEEE 754's decimal128 (D116): `coefficient × 10^exponent`, with at most 34 digits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Decimal {
    pub negative: bool,
    pub coefficient: u128,
    pub exponent: i32,
}

impl Decimal {
    #[must_use]
    pub fn from_whole(value: i64) -> Decimal {
        Decimal { negative: value < 0, coefficient: u128::from(value.unsigned_abs()), exponent: 0 }
    }

    /// Its exact value.
    #[must_use]
    pub fn exact(&self) -> HugeDecimal {
        let magnitude = Big::from_u128(self.coefficient);
        HugeDecimal::new(if self.negative { magnitude.negated() } else { magnitude }, i64::from(self.exponent))
    }

    /// An exact decimal rounded to a `Decimal`, to the nearest, a tie to even.
    ///
    /// # Errors
    ///
    /// `TooBig` past a `Decimal`'s largest.
    pub fn rounded(value: &HugeDecimal) -> Result<Decimal, NumberError> {
        round(value.is_negative(), value.coefficient.abs(), value.exponent, false)
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.coefficient == 0
    }

    #[must_use]
    pub fn negated(&self) -> Decimal {
        Decimal { negative: !self.negative && !self.is_zero(), ..*self }
    }

    #[must_use]
    pub fn abs(&self) -> Decimal {
        Decimal { negative: false, ..*self }
    }

    fn signed(&self) -> i128 {
        // A coefficient is below 10^34, far inside an `i128`.
        let magnitude = self.coefficient as i128;
        if self.negative { -magnitude } else { magnitude }
    }

    /// `+`, exactly and rounded once (D116).
    ///
    /// # Errors
    ///
    /// `TooBig`.
    pub fn add(&self, other: &Decimal) -> Result<Decimal, NumberError> {
        let (low, high) = if self.exponent <= other.exponent { (self, other) } else { (other, self) };
        let gap = (high.exponent - low.exponent) as usize;
        // The everyday case: both fit an `i128` at the smaller exponent.
        if let Some(&scale) = POWERS_OF_TEN.get(gap)
            && let Some(high_scaled) = high.signed().checked_mul(scale as i128)
            && let Some(sum) = high_scaled.checked_add(low.signed())
            && sum.unsigned_abs() < POWERS_OF_TEN[DIGITS as usize]
        {
            return Ok(Decimal { negative: sum < 0, coefficient: sum.unsigned_abs(), exponent: low.exponent });
        }
        Decimal::rounded(&self.exact().add(&other.exact()))
    }

    /// `-`.
    ///
    /// # Errors
    ///
    /// `TooBig`.
    pub fn subtract(&self, other: &Decimal) -> Result<Decimal, NumberError> {
        self.add(&other.negated())
    }

    /// `*`.
    ///
    /// # Errors
    ///
    /// `TooBig`.
    pub fn multiply(&self, other: &Decimal) -> Result<Decimal, NumberError> {
        let exponent = i64::from(self.exponent) + i64::from(other.exponent);
        let negative = self.negative != other.negative;
        if let Some(product) = self.coefficient.checked_mul(other.coefficient)
            && product < POWERS_OF_TEN[DIGITS as usize]
            && (EXPONENT_MIN..=EXPONENT_MAX).contains(&exponent)
        {
            return Ok(Decimal { negative: negative && product != 0, coefficient: product, exponent: exponent as i32 });
        }
        round(negative, Big::from_u128(self.coefficient).multiply(&Big::from_u128(other.coefficient)), exponent, false)
    }

    /// `/`, to the nearest 34 digits, or exactly with the exponent IEEE 754 prefers when the quotient has an end.
    ///
    /// # Errors
    ///
    /// `ByZero` or `TooBig`.
    pub fn divide(&self, other: &Decimal) -> Result<Decimal, NumberError> {
        divide(&self.exact(), &other.exact())
    }

    /// `div` and `rem` (D127): a whole-valued `Decimal` and what's left over.
    ///
    /// # Errors
    ///
    /// `ByZero`, or `TooBig` when the whole number of times has more than 34 digits.
    pub fn divide_whole(&self, other: &Decimal, floor: bool) -> Result<(Decimal, Decimal), NumberError> {
        let (quotient, remainder) = self.exact().divide_whole(&other.exact(), floor)?;
        if digit_count(&quotient) > DIGITS {
            return Err(NumberError::TooBig);
        }
        let quotient = Decimal::rounded(&HugeDecimal::from_big(quotient))?;
        Ok((quotient, Decimal::rounded(&remainder)?))
    }

    /// Compares amounts (D125).
    #[must_use]
    pub fn compare(&self, other: &Decimal) -> Ordering {
        if self.exponent == other.exponent {
            return self.signed().cmp(&other.signed());
        }
        self.exact().compare(&other.exact())
    }

    /// `sqrt`, correctly rounded, exactly with the exponent IEEE 754 prefers when the root has an end (D127).
    ///
    /// # Errors
    ///
    /// `Negative`.
    pub fn square_root(&self) -> Result<Decimal, NumberError> {
        if self.negative && !self.is_zero() {
            return Err(NumberError::Negative);
        }
        let exponent = i64::from(self.exponent);
        let ideal = exponent.div_euclid(2);
        if self.is_zero() {
            return Ok(Decimal { negative: false, coefficient: 0, exponent: ideal as i32 });
        }
        let (mut coefficient, mut even) = (Big::from_u128(self.coefficient), exponent);
        if even.rem_euclid(2) == 1 {
            coefficient = coefficient.multiply(&Big::from_u64(10));
            even -= 1;
        }
        // Enough digits that the root has at least 35, one past those kept.
        let extra = (70u64.saturating_sub(digit_count(&coefficient))).div_ceil(2);
        let scaled = coefficient.multiply(&ten_to(2 * extra));
        let root = scaled.square_root_floor();
        let root_exponent = even / 2 - extra as i64;
        if root.multiply(&root) == scaled {
            let exact = HugeDecimal::new(root, root_exponent).trimmed(ideal);
            return round(false, exact.coefficient, exact.exponent, false);
        }
        round(false, root, root_exponent, true)
    }

    /// Raised to a whole power, correctly rounded (D127).
    ///
    /// # Errors
    ///
    /// `ByZero` for zero to a negative power, or `TooBig`.
    pub fn power(&self, power: i64) -> Result<Decimal, NumberError> {
        if power == 0 {
            return Ok(Decimal { negative: false, coefficient: 1, exponent: 0 });
        }
        let times = power.unsigned_abs();
        if self.is_zero() {
            if power < 0 {
                return Err(NumberError::ByZero);
            }
            // As IEEE 754's `pow` gives it: a plain zero.
            return Ok(Decimal::default());
        }
        let negative = self.negative && times % 2 == 1;
        let size = digit_count(&Big::from_u128(self.coefficient)).saturating_mul(times);
        if size <= EXACT_POWER_DIGITS {
            let raised = HugeDecimal::new(
                Big::from_u128(self.coefficient).power(times),
                i64::from(self.exponent) * times as i64,
            );
            let raised = if negative { raised.negated() } else { raised };
            return if power > 0 { Decimal::rounded(&raised) } else { divide(&HugeDecimal::from_whole(1), &raised) };
        }
        // Too many digits to work out exactly: narrowed down between two bounds, closer each time, until both
        // round to the same `Decimal`, which is then the rounding of the value between them.
        let mut precision = 60u64;
        loop {
            let (low, high) = power_bounds(self.coefficient, i64::from(self.exponent), times, precision);
            let (low, high) = if power > 0 {
                (low, high)
            } else {
                (reciprocal(&high, precision, false), reciprocal(&low, precision, true))
            };
            let first = round(negative, low.coefficient, low.exponent, false);
            let second = round(negative, high.coefficient, high.exponent, false);
            if first == second {
                return first;
            }
            if precision > size + 100 {
                // Past the exact size the bounds are the value itself; this isn't reached.
                return first;
            }
            precision *= 2;
        }
    }

    /// `lerp(a, to: b, by: t)`, a + t × (b − a), worked out exactly and rounded once (D127, D114).
    ///
    /// # Errors
    ///
    /// `TooBig`.
    pub fn lerp(&self, to: &Decimal, by: &Decimal) -> Result<Decimal, NumberError> {
        let (a, b, t) = (self.exact(), to.exact(), by.exact());
        Decimal::rounded(&a.add(&t.multiply(&b.subtract(&a))))
    }

    /// The nearest `Float`.
    #[must_use]
    pub fn to_float(&self) -> f64 {
        self.exact().to_float()
    }

    /// The `Decimal` a `Float` shows: `be.Decimal.shown` (D126).
    ///
    /// # Errors
    ///
    /// `TooBig`, which no `Float` is.
    pub fn from_float_shown(value: f64) -> Result<Decimal, NumberError> {
        let shown = crate::number::float_to_text(value);
        let (negative, digits) = match shown.trim_start_matches('~').strip_prefix('-') {
            Some(rest) => (true, rest.to_owned()),
            None => (false, shown.trim_start_matches('~').to_owned()),
        };
        let read = HugeDecimal::read(&digits).unwrap_or_default();
        Decimal::rounded(&if negative { read.negated() } else { read })
    }

    /// The `Decimal` nearest a `Float`'s exact value: `be.Decimal.nearest` (D126).
    ///
    /// # Errors
    ///
    /// `TooBig`, which no `Float` is.
    pub fn from_float_nearest(value: f64) -> Result<Decimal, NumberError> {
        Decimal::rounded(&HugeDecimal::from_float(value))
    }

    /// As shown: exactly its digits, with the places it keeps, and no `~` (D124, D125).
    #[must_use]
    pub fn to_text(&self) -> String {
        show(self.negative, &self.coefficient.to_string(), i64::from(self.exponent))
    }
}

/// `numerator / denominator`, exact values, as a `Decimal`.
fn divide(numerator: &HugeDecimal, denominator: &HugeDecimal) -> Result<Decimal, NumberError> {
    if denominator.is_zero() {
        return Err(NumberError::ByZero);
    }
    let negative = numerator.is_negative() != denominator.is_negative();
    let preferred = numerator.exponent - denominator.exponent;
    if numerator.is_zero() {
        return round(false, Big::zero(), preferred, false);
    }
    let (top, bottom) = (numerator.coefficient.abs(), denominator.coefficient.abs());
    // Enough digits that the quotient has at least 35, one past those kept.
    let shift = (DIGITS + 1 + digit_count(&bottom)).saturating_sub(digit_count(&top));
    let Some((quotient, remainder)) = top.multiply(&ten_to(shift)).divide_truncating(&bottom) else {
        return Err(NumberError::ByZero);
    };
    let exponent = preferred - shift as i64;
    if remainder.is_zero() {
        let exact = HugeDecimal::new(quotient, exponent).trimmed(preferred);
        return round(negative, exact.coefficient, exact.exponent, false);
    }
    round(negative, quotient, exponent, true)
}

/// A magnitude cut to `precision` digits, down or up: `(coefficient, exponent)`.
fn cut(value: Big, exponent: i64, precision: u64, up: bool) -> (Big, i64) {
    let digits = digit_count(&value);
    if digits <= precision {
        return (value, exponent);
    }
    let dropped = digits - precision;
    let Some((quotient, remainder)) = value.divide_truncating(&ten_to(dropped)) else { return (value, exponent) };
    let quotient = if up && !remainder.is_zero() { quotient.add(&Big::from_u64(1)) } else { quotient };
    (quotient, exponent + dropped as i64)
}

/// A magnitude's power, between two bounds kept to `precision` digits.
fn power_bounds(coefficient: u128, exponent: i64, times: u64, precision: u64) -> (HugeDecimal, HugeDecimal) {
    let base = Big::from_u128(coefficient);
    let (mut low, mut high) = ((Big::from_u64(1), 0i64), (Big::from_u64(1), 0i64));
    let (mut base_low, mut base_high) = ((base.clone(), exponent), (base, exponent));
    let mut left = times;
    while left > 0 {
        if left & 1 == 1 {
            low = cut(low.0.multiply(&base_low.0), low.1 + base_low.1, precision, false);
            high = cut(high.0.multiply(&base_high.0), high.1 + base_high.1, precision, true);
        }
        left >>= 1;
        if left > 0 {
            base_low = cut(base_low.0.multiply(&base_low.0), base_low.1 * 2, precision, false);
            base_high = cut(base_high.0.multiply(&base_high.0), base_high.1 * 2, precision, true);
        }
    }
    (HugeDecimal::new(low.0, low.1), HugeDecimal::new(high.0, high.1))
}

/// One over a positive magnitude, to `precision` digits, rounded down or up.
fn reciprocal(value: &HugeDecimal, precision: u64, up: bool) -> HugeDecimal {
    let shift = precision + digit_count(&value.coefficient);
    let Some((quotient, remainder)) = ten_to(shift).divide_truncating(&value.coefficient) else {
        return HugeDecimal::default();
    };
    let quotient = if up && !remainder.is_zero() { quotient.add(&Big::from_u64(1)) } else { quotient };
    HugeDecimal::new(quotient, -(shift as i64) - value.exponent)
}

/// A number too big for a slot: a `HugeWhole`, a `Decimal` or a `HugeDecimal` (D115, D116). Each is made once and
/// never changed: an operation makes a new one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Number {
    Huge(Big),
    Decimal(Decimal),
    HugeDecimal(HugeDecimal),
}

impl Number {
    /// What it costs of the memory cap beyond the value itself.
    #[must_use]
    pub fn size(&self) -> usize {
        match self {
            Number::Huge(value) => value.size(),
            Number::Decimal(_) => 16,
            // Its exponent's zeros count too, since lining two up writes them out (D14).
            Number::HugeDecimal(value) => value
                .coefficient
                .size()
                .saturating_add(8)
                .saturating_add(usize::try_from(value.exponent.unsigned_abs() / 2).unwrap_or(usize::MAX)),
        }
    }
}

/// A magnitude rounded to a `Decimal`: to 34 digits and into the exponents it has, to the nearest, a tie to even.
/// `sticky` says the value is a little more than `magnitude × 10^exponent`, which has more than 34 digits then.
fn round(negative: bool, magnitude: Big, exponent: i64, sticky: bool) -> Result<Decimal, NumberError> {
    let digits = digit_count(&magnitude);
    let mut dropped = digits.saturating_sub(DIGITS) as i64;
    if exponent + dropped < EXPONENT_MIN {
        dropped = EXPONENT_MIN - exponent;
    }
    let (mut coefficient, mut exponent) = (magnitude, exponent);
    if dropped > 0 {
        let divisor = ten_to(dropped as u64);
        let Some((quotient, remainder)) = coefficient.divide_truncating(&divisor) else {
            return Err(NumberError::TooBig);
        };
        let up = match remainder.shift_left(1).compare(&divisor) {
            Ordering::Greater => true,
            Ordering::Less => false,
            Ordering::Equal => sticky || quotient.is_odd(),
        };
        coefficient = if up { quotient.add(&Big::from_u64(1)) } else { quotient };
        exponent += dropped;
        if digit_count(&coefficient) > DIGITS {
            // Rounding up made 10^34, which is 10^33 one place up.
            coefficient = ten_to(DIGITS - 1);
            exponent += 1;
        }
    }
    if coefficient.is_zero() {
        return Ok(Decimal {
            negative: false,
            coefficient: 0,
            exponent: exponent.clamp(EXPONENT_MIN, EXPONENT_MAX) as i32,
        });
    }
    if exponent > EXPONENT_MAX {
        // Zeros moved into the coefficient, when they fit.
        let padding = (exponent - EXPONENT_MAX) as u64;
        if digit_count(&coefficient) + padding > DIGITS {
            return Err(NumberError::TooBig);
        }
        coefficient = coefficient.multiply(&ten_to(padding));
        exponent = EXPONENT_MAX;
    }
    let coefficient = coefficient.to_u128_magnitude().ok_or(NumberError::TooBig)?;
    Ok(Decimal { negative, coefficient, exponent: exponent as i32 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(text: &str) -> Decimal {
        let (negative, rest) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let read = HugeDecimal::read(rest).unwrap_or_default();
        Decimal::rounded(&if negative { read.negated() } else { read }).unwrap_or_default()
    }

    fn shown(result: Result<Decimal, NumberError>) -> String {
        result.map_or_else(|error| format!("{error:?}"), |value| value.to_text())
    }

    // trace: D116, D124, D125
    #[test]
    #[allow(clippy::float_cmp)] // the nearest `Float` is exactly this one
    fn d116_d125_decimals_count_in_tens_and_keep_their_places() {
        assert_eq!(shown(decimal("1.50").add(&decimal("1.50"))), "3.00");
        assert_eq!(shown(decimal("4.99").add(&decimal("0.01"))), "5.00");
        assert_eq!(shown(decimal("0.1").add(&decimal("0.2"))), "0.3");
        assert_eq!(shown(decimal("4.99").multiply(&decimal("3"))), "14.97");
        assert_eq!(shown(decimal("10.0").divide(&decimal("3"))), "3.333333333333333333333333333333333");
        assert_eq!(shown(decimal("1").divide(&decimal("4"))), "0.25");
        assert_eq!(shown(decimal("10.00").divide(&decimal("2"))), "5.00");
        assert_eq!(shown(decimal("1").divide(&decimal("0"))), "ByZero");
        assert_eq!(decimal("3.00").compare(&decimal("3.0")), Ordering::Equal);
        assert_eq!(decimal("-0.5").compare(&decimal("0.25")), Ordering::Less);
        assert_eq!(shown(decimal("2.25").square_root()), "1.5");
        assert_eq!(shown(decimal("2").square_root()), "1.414213562373095048801688724209698");
        assert_eq!(shown(decimal("1.5").power(2)), "2.25");
        assert_eq!(shown(decimal("2").power(-1)), "0.5");
        assert_eq!(shown(decimal("1.0000001").power(1_000_000)), "1.105170912549793416638382709346716");
        let (each, left) = decimal("10.00").divide_whole(&decimal("3"), true).unwrap_or_default();
        assert_eq!((each.to_text(), left.to_text()), ("3".to_owned(), "1.00".to_owned()));
        assert_eq!(decimal("0.1").to_float(), 0.1);
        assert_eq!(shown(Decimal::from_float_shown(0.1)), "0.1");
        assert_eq!(shown(Decimal::from_float_nearest(0.1)), "0.1000000000000000055511151231257827");
        assert_eq!(
            shown(decimal("9999999999999999999999999999999999").add(&decimal("1"))),
            "10000000000000000000000000000000000"
        );
    }

    // trace: D116, D127
    #[test]
    fn d127_huge_decimals_are_exact_or_fail() {
        let read = |text: &str| HugeDecimal::read(text).unwrap_or_default();
        assert_eq!(read("1").divide(&read("3")), Err(NumberError::Inexact));
        assert_eq!(read("1").divide(&read("8")).map(|value| value.to_text()), Ok("0.125".to_owned()));
        assert_eq!(read("2.25").square_root().map(|value| value.to_text()), Ok("1.5".to_owned()));
        assert_eq!(read("2").square_root(), Err(NumberError::Inexact));
        assert_eq!(read("0.1").add(&read("0.2")).to_text(), "0.3");
        assert_eq!(read("1.10").multiply(&read("1.10")).to_text(), "1.2100");
        assert_eq!(read("2.5").to_whole(Rounding::Nearest), Big::from_u64(3));
        assert_eq!(read("2.5").to_whole(Rounding::NearestEven), Big::from_u64(2));
        assert_eq!(read("2.5").negated().to_whole(Rounding::Floor), Big::from_i64(-3));
    }
}

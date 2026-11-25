//! Numbers as text and back, by Unio's rules, which Seal takes (D16): how a `Float` is shown (Unio D155, D181 to
//! D183), whether a `Float` literal is exact (Unio D155), which text `be.Whole` reads (Unio D95, D143), and how
//! `be.Whole` takes a `Float`'s fraction (Unio D178).

use crate::check::tree::Rounding;

/// An unsigned whole number of any size, in base 2³², least significant part first. Just enough arithmetic to tell
/// whether a decimal number and a `Float` are exactly equal.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Big(Vec<u32>);

impl Big {
    fn from_u64(value: u64) -> Big {
        let mut big = Big(vec![(value & 0xFFFF_FFFF) as u32, (value >> 32) as u32]);
        big.trim();
        big
    }

    fn from_digits(digits: &str) -> Big {
        let mut big = Big(Vec::new());
        for digit in digits.bytes().filter(u8::is_ascii_digit) {
            big.multiply_small(10);
            big.add_small(u32::from(digit - b'0'));
        }
        big
    }

    fn trim(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }

    fn multiply_small(&mut self, factor: u32) {
        let mut carry = 0u64;
        for part in &mut self.0 {
            let product = u64::from(*part) * u64::from(factor) + carry;
            *part = (product & 0xFFFF_FFFF) as u32;
            carry = product >> 32;
        }
        if carry > 0 {
            self.0.push((carry & 0xFFFF_FFFF) as u32);
        }
    }

    fn add_small(&mut self, addend: u32) {
        let mut carry = u64::from(addend);
        for part in &mut self.0 {
            let sum = u64::from(*part) + carry;
            *part = (sum & 0xFFFF_FFFF) as u32;
            carry = sum >> 32;
            if carry == 0 {
                return;
            }
        }
        if carry > 0 {
            self.0.push((carry & 0xFFFF_FFFF) as u32);
        }
    }

    fn times_power_of_ten(mut self, power: u32) -> Big {
        for _ in 0..power {
            self.multiply_small(10);
        }
        self
    }

    fn times_power_of_two(mut self, power: u32) -> Big {
        for _ in 0..power / 16 {
            self.multiply_small(1 << 16);
        }
        self.multiply_small(1 << (power % 16));
        self
    }
}

/// Splits a finite `Float` into a whole number and a power of two: the value is `mantissa × 2^exponent` exactly.
fn decompose(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let exponent_bits = i32::try_from((bits >> 52) & 0x7FF).unwrap_or(0);
    let fraction = bits & 0x000F_FFFF_FFFF_FFFF;
    if exponent_bits == 0 { (fraction, -1074) } else { (fraction | (1 << 52), exponent_bits - 1075) }
}

/// Whether the decimal number `digits × 10^-scale` is exactly the `Float` `value` (ignoring signs).
fn decimal_equals_float(digits: &str, scale: u32, value: f64) -> bool {
    if !value.is_finite() {
        return false;
    }
    let (mantissa, exponent) = decompose(value.abs());
    // digits × 10^-scale == mantissa × 2^exponent, with both sides multiplied up to whole numbers.
    let mut decimal = Big::from_digits(digits);
    let mut binary = Big::from_u64(mantissa).times_power_of_ten(scale);
    if exponent >= 0 {
        binary = binary.times_power_of_two(exponent.unsigned_abs());
    } else {
        decimal = decimal.times_power_of_two(exponent.unsigned_abs());
    }
    decimal.trim();
    binary.trim();
    decimal == binary
}

/// Reads a `Float` literal, as `2.5`, giving its value and whether the literal is stored exactly (Unio D155).
/// Gives nothing when the number is too big for a `Float`.
#[must_use]
pub fn read_float_literal(text: &str) -> Option<(f64, bool)> {
    let value: f64 = text.parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let digits = format!("{whole}{fraction}");
    let scale = u32::try_from(fraction.len()).ok()?;
    Some((value, decimal_equals_float(&digits, scale, value)))
}

/// A `Float` as text (Unio D155, D181, D182, D183): the shortest digits that read back as the same `Float`, written
/// out in full with no `e`, with `.0` when there's no fraction, `~` in front when the stored value isn't exactly
/// those digits, and negative zero as `0.0`.
#[must_use]
pub fn float_to_text(value: f64) -> String {
    if value == 0.0 {
        return "0.0".to_owned();
    }
    // Rust's own formatting gives the shortest digits that read back the same, in full, with no `e`.
    let mut shortest = format!("{}", value.abs());
    let (whole, fraction) = match shortest.split_once('.') {
        Some((whole, fraction)) => (whole.to_owned(), fraction.to_owned()),
        None => (shortest.clone(), String::new()),
    };
    let digits = format!("{whole}{fraction}");
    let exact = decimal_equals_float(&digits, u32::try_from(fraction.len()).unwrap_or(u32::MAX), value);
    if fraction.is_empty() {
        shortest.push_str(".0");
    }
    let mut text = String::with_capacity(shortest.len() + 2);
    if !exact {
        text.push('~');
    }
    if value < 0.0 {
        text.push('-');
    }
    text.push_str(&shortest);
    text
}

/// A `Float` rounded to `places` digits after its point for showing, with no `~` (D94, Unio D175): to the nearest,
/// a tie going away from zero (Unio D178), and with no minus when it rounds to zero (Unio D176).
#[must_use]
pub fn float_with_places(value: f64, places: u8) -> String {
    // Every `Float` is exactly a decimal with at most 1074 digits after its point, so this is its exact value.
    let exact = format!("{:.1100}", value.abs());
    let (whole, fraction) = exact.split_once('.').unwrap_or((&exact, ""));
    let places = usize::from(places);
    let mut digits: Vec<u8> = whole.bytes().chain(fraction.bytes().take(places)).map(|digit| digit - b'0').collect();
    let next = fraction.as_bytes().get(places).map_or(0, |digit| digit - b'0');
    if next >= 5 {
        // Every digit after the kept ones is looked at once: a 5 or more is at least half, and a tie goes up too.
        let mut carry = true;
        for digit in digits.iter_mut().rev() {
            if !carry {
                break;
            }
            if *digit == 9 {
                *digit = 0;
            } else {
                *digit += 1;
                carry = false;
            }
        }
        if carry {
            digits.insert(0, 1);
        }
    }
    let whole_length = digits.len() - places;
    let mut text = String::with_capacity(digits.len() + 2);
    if value < 0.0 && digits.iter().any(|&digit| digit != 0) {
        text.push('-');
    }
    for (index, digit) in digits.iter().enumerate() {
        if index == whole_length {
            text.push('.');
        }
        text.push(char::from(b'0' + digit));
    }
    text
}

/// The whole number `be.Whole` reads from text (Unio D95, D143): spaces and tabs around it are ignored, a `+` or
/// `-` in front and leading zeros are fine, and anything else, or a number that doesn't fit, fails.
#[must_use]
pub fn text_to_whole(text: &str) -> Option<i64> {
    let trimmed = text.trim_matches(|character| character == ' ' || character == '\t');
    let (negative, digits) = match trimmed.as_bytes().first() {
        Some(b'-') => (true, trimmed.get(1..)?),
        Some(b'+') => (false, trimmed.get(1..)?),
        _ => (false, trimmed),
    };
    if digits.is_empty() || !digits.bytes().all(|digit| digit.is_ascii_digit()) {
        return None;
    }
    // Built as a negative number, so that the smallest `Whole` reads too.
    let mut value: i64 = 0;
    for digit in digits.bytes() {
        value = value.checked_mul(10)?.checked_sub(i64::from(digit - b'0'))?;
    }
    if negative { Some(value) } else { value.checked_neg() }
}

/// `be.Whole.floor` and the rest on a `Float` (Unio D178). Gives nothing when the result is too big for a `Whole`.
#[must_use]
pub fn float_to_whole(value: f64, rounding: Rounding) -> Option<i64> {
    // 2⁶³ is exactly a `Float`; anything from it up, or below -2⁶³, doesn't fit a `Whole`.
    const LIMIT: f64 = 9_223_372_036_854_775_808.0;
    let rounded = match rounding {
        Rounding::Floor => value.floor(),
        Rounding::Trunc => value.trunc(),
        Rounding::Ceil => value.ceil(),
        // A tie goes away from zero, as at school.
        Rounding::Nearest => value.round(),
        Rounding::NearestEven => value.round_ties_even(),
    };
    if (-LIMIT..LIMIT).contains(&rounded) {
        // The check above makes this cast exact: `rounded` is a whole number inside a `Whole`'s range.
        #[allow(clippy::cast_possible_truncation)]
        Some(rounded as i64)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: Unio D155, D181, D182, D183
    #[test]
    fn d94_places_rounds_ties_away_from_zero() {
        assert_eq!(float_with_places(0.125, 2), "0.13");
        assert_eq!(float_with_places(0.5, 3), "0.500");
        assert_eq!(float_with_places(2.5, 0), "3");
        assert_eq!(float_with_places(-0.001, 2), "0.00");
        assert_eq!(float_with_places(-0.006, 2), "-0.01");
        assert_eq!(float_with_places(9.999, 2), "10.00");
        assert_eq!(float_with_places(0.1 + 0.2, 2), "0.30");
    }

    #[test]
    fn unio_d155_floats_show_their_tilde() {
        assert_eq!(float_to_text(0.1 + 0.2), "~0.30000000000000004");
        assert_eq!(float_to_text(0.5 + 0.25), "0.75");
        assert_eq!(float_to_text(7.0), "7.0");
        assert_eq!(float_to_text(1e22), "10000000000000000000000.0");
        assert_eq!(float_to_text(1_180_591_620_717_411_303_424.0), "~1180591620717411300000.0");
        assert_eq!(float_to_text(0.00001), "~0.00001");
        assert_eq!(float_to_text(-0.0), "0.0");
        assert_eq!(float_to_text(10.0 / 3.0), "~3.3333333333333335");
        assert_eq!(float_to_text(-2.5), "-2.5");
        assert_eq!(float_to_text(-0.1), "~-0.1");
    }

    // trace: Unio D155
    #[test]
    fn unio_d155_exact_and_inexact_literals() {
        assert_eq!(read_float_literal("0.5"), Some((0.5, true)));
        assert_eq!(read_float_literal("0.1").map(|(_, exact)| exact), Some(false));
        assert_eq!(read_float_literal("7.0").map(|(_, exact)| exact), Some(true));
        assert_eq!(read_float_literal("0.30000000000000004").map(|(_, exact)| exact), Some(false));
    }

    // trace: Unio D95, D143
    #[test]
    fn unio_d95_which_text_be_whole_reads() {
        assert_eq!(text_to_whole("42"), Some(42));
        assert_eq!(text_to_whole(" 42\t"), Some(42));
        assert_eq!(text_to_whole("-7"), Some(-7));
        assert_eq!(text_to_whole("+5"), Some(5));
        assert_eq!(text_to_whole("007"), Some(7));
        assert_eq!(text_to_whole("-0"), Some(0));
        assert_eq!(text_to_whole("-9223372036854775808"), Some(i64::MIN));
        for bad in ["abc", "", "4 2", "4.5", "1,000", "1_000", "9223372036854775808", "-", "+"] {
            assert_eq!(text_to_whole(bad), None, "{bad}");
        }
    }

    // trace: Unio D178
    #[test]
    fn unio_d178_be_whole_on_a_float() {
        assert_eq!(float_to_whole(2.7, Rounding::Floor), Some(2));
        assert_eq!(float_to_whole(-2.7, Rounding::Floor), Some(-3));
        assert_eq!(float_to_whole(-2.7, Rounding::Trunc), Some(-2));
        assert_eq!(float_to_whole(2.3, Rounding::Ceil), Some(3));
        assert_eq!(float_to_whole(2.5, Rounding::Nearest), Some(3));
        assert_eq!(float_to_whole(-2.5, Rounding::Nearest), Some(-3));
        assert_eq!(float_to_whole(2.5, Rounding::NearestEven), Some(2));
        assert_eq!(float_to_whole(3.5, Rounding::NearestEven), Some(4));
        assert_eq!(float_to_whole(1e300, Rounding::Floor), None);
    }
}

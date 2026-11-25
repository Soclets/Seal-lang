//! Whole numbers of any size, for `HugeWhole` (D115), exact decimals (D116), and working maths out exactly or at any
//! precision, so it can be correctly rounded (D50).
//!
//! Written here, with no `unsafe` (Building rule 2): a number is its sign and its digits in base 2^32, least
//! significant first, with no zero digits at the top.
// Splitting a wide number into narrower digits keeps exactly the bits wanted, which is what these casts are for.
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)]

use std::cmp::Ordering;
use std::fmt::Write as _;

/// A whole number of any size.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Big {
    negative: bool,
    /// Base 2^32 digits, least significant first, with no zeros at the top; empty for zero.
    digits: Vec<u32>,
}

/// Adds two digit lists.
fn add_digits(left: &[u32], right: &[u32]) -> Vec<u32> {
    let (long, short) = if left.len() >= right.len() { (left, right) } else { (right, left) };
    let mut out = Vec::with_capacity(long.len() + 1);
    let mut carry = 0u64;
    for (index, &digit) in long.iter().enumerate() {
        let sum = u64::from(digit) + u64::from(short.get(index).copied().unwrap_or(0)) + carry;
        out.push(sum as u32);
        carry = sum >> 32;
    }
    if carry != 0 {
        out.push(carry as u32);
    }
    out
}

/// Takes `right` from `left`, which must be at least as big.
fn subtract_digits(left: &[u32], right: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(left.len());
    let mut borrow = 0i64;
    for (index, &digit) in left.iter().enumerate() {
        let mut difference = i64::from(digit) - i64::from(right.get(index).copied().unwrap_or(0)) - borrow;
        if difference < 0 {
            difference += 1 << 32;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out.push(difference as u32);
    }
    trim(&mut out);
    out
}

fn trim(digits: &mut Vec<u32>) {
    while digits.last() == Some(&0) {
        digits.pop();
    }
}

fn compare_digits(left: &[u32], right: &[u32]) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.iter().rev().cmp(right.iter().rev()))
}

fn multiply_digits(left: &[u32], right: &[u32]) -> Vec<u32> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    let mut out = vec![0u32; left.len() + right.len()];
    for (i, &a) in left.iter().enumerate() {
        let mut carry = 0u64;
        for (j, &b) in right.iter().enumerate() {
            let Some(slot) = out.get_mut(i + j) else { continue };
            let product = u64::from(a) * u64::from(b) + u64::from(*slot) + carry;
            *slot = product as u32;
            carry = product >> 32;
        }
        let mut position = i + right.len();
        while carry != 0 {
            let Some(slot) = out.get_mut(position) else { break };
            let sum = u64::from(*slot) + carry;
            *slot = sum as u32;
            carry = sum >> 32;
            position += 1;
        }
    }
    trim(&mut out);
    out
}

/// Divides by a single digit, giving the quotient and the remainder.
fn divide_small(digits: &[u32], divisor: u32) -> (Vec<u32>, u32) {
    let mut out = vec![0u32; digits.len()];
    let mut remainder = 0u64;
    for (index, &digit) in digits.iter().enumerate().rev() {
        let current = (remainder << 32) | u64::from(digit);
        if let Some(slot) = out.get_mut(index) {
            *slot = (current / u64::from(divisor)) as u32;
        }
        remainder = current % u64::from(divisor);
    }
    trim(&mut out);
    (out, remainder as u32)
}

/// Long division, Knuth's algorithm D: the quotient and remainder of two digit lists, the divisor not zero.
fn divide_digits(dividend: &[u32], divisor: &[u32]) -> (Vec<u32>, Vec<u32>) {
    if compare_digits(dividend, divisor) == Ordering::Less {
        return (Vec::new(), dividend.to_vec());
    }
    if let [single] = divisor {
        let (quotient, remainder) = divide_small(dividend, *single);
        let mut remainder = vec![remainder];
        trim(&mut remainder);
        return (quotient, remainder);
    }
    // Normalised so the divisor's top digit has its top bit set.
    let shift = divisor.last().map_or(0, |top| top.leading_zeros());
    let divisor = shift_left_digits(divisor, shift);
    let mut remainder = shift_left_digits(dividend, shift);
    remainder.push(0);
    let n = divisor.len();
    let m = remainder.len() - n;
    let mut quotient = vec![0u32; m];
    let top = u64::from(divisor.last().copied().unwrap_or(1));
    let next = u64::from(divisor.get(n.wrapping_sub(2)).copied().unwrap_or(0));
    for j in (0..m).rev() {
        let high = u64::from(remainder.get(j + n).copied().unwrap_or(0));
        let middle = u64::from(remainder.get(j + n - 1).copied().unwrap_or(0));
        let low = u64::from(remainder.get(j + n - 2).copied().unwrap_or(0));
        let numerator = (high << 32) | middle;
        let mut guess = numerator / top;
        let mut rest = numerator % top;
        while guess >= (1 << 32) || guess * next > ((rest << 32) | low) {
            guess -= 1;
            rest += top;
            if rest >= (1 << 32) {
                break;
            }
        }
        // Takes guess × divisor away from the remainder's window.
        let mut borrow = 0i64;
        let mut carry = 0u64;
        for i in 0..n {
            let product = guess * u64::from(divisor.get(i).copied().unwrap_or(0)) + carry;
            carry = product >> 32;
            let Some(slot) = remainder.get_mut(i + j) else { continue };
            let difference = i64::from(*slot) - i64::from(product as u32) - borrow;
            *slot = difference as u32;
            borrow = i64::from(difference < 0);
        }
        let Some(slot) = remainder.get_mut(j + n) else { continue };
        let difference = i64::from(*slot) - carry as i64 - borrow;
        *slot = difference as u32;
        if difference < 0 {
            // The guess was one too many: add the divisor back.
            guess -= 1;
            let mut carry = 0u64;
            for i in 0..n {
                let Some(slot) = remainder.get_mut(i + j) else { continue };
                let sum = u64::from(*slot) + u64::from(divisor.get(i).copied().unwrap_or(0)) + carry;
                *slot = sum as u32;
                carry = sum >> 32;
            }
            if let Some(slot) = remainder.get_mut(j + n) {
                *slot = slot.wrapping_add(carry as u32);
            }
        }
        if let Some(slot) = quotient.get_mut(j) {
            *slot = guess as u32;
        }
    }
    trim(&mut quotient);
    remainder.truncate(n);
    let mut remainder = shift_right_digits(&remainder, shift);
    trim(&mut remainder);
    (quotient, remainder)
}

fn shift_left_digits(digits: &[u32], bits: u32) -> Vec<u32> {
    if bits == 0 {
        return digits.to_vec();
    }
    let mut out = Vec::with_capacity(digits.len() + 1);
    let mut carry = 0u32;
    for &digit in digits {
        out.push((digit << bits) | carry);
        carry = digit >> (32 - bits);
    }
    if carry != 0 {
        out.push(carry);
    }
    out
}

fn shift_right_digits(digits: &[u32], bits: u32) -> Vec<u32> {
    if bits == 0 {
        return digits.to_vec();
    }
    let mut out = Vec::with_capacity(digits.len());
    for (index, &digit) in digits.iter().enumerate() {
        let next = digits.get(index + 1).copied().unwrap_or(0);
        out.push((digit >> bits) | (next << (32 - bits)));
    }
    trim(&mut out);
    out
}

impl Big {
    #[must_use]
    pub fn zero() -> Big {
        Big::default()
    }

    #[must_use]
    pub fn from_u64(value: u64) -> Big {
        let mut digits = vec![value as u32, (value >> 32) as u32];
        trim(&mut digits);
        Big { negative: false, digits }
    }

    #[must_use]
    pub fn from_i64(value: i64) -> Big {
        let mut big = Big::from_u64(value.unsigned_abs());
        big.negative = value < 0 && !big.digits.is_empty();
        big
    }

    #[must_use]
    pub fn from_u128(value: u128) -> Big {
        let mut digits = vec![value as u32, (value >> 32) as u32, (value >> 64) as u32, (value >> 96) as u32];
        trim(&mut digits);
        Big { negative: false, digits }
    }

    fn from_parts(negative: bool, mut digits: Vec<u32>) -> Big {
        trim(&mut digits);
        Big { negative: negative && !digits.is_empty(), digits }
    }

    /// The number written in decimal digits, with a `-` in front when negative; nothing if it isn't one.
    #[must_use]
    pub fn parse(text: &str) -> Option<Big> {
        let (negative, digits) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text.strip_prefix('+').unwrap_or(text)),
        };
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        let mut value: Vec<u32> = Vec::new();
        // Nine decimal digits at a time fit in a base 2^32 digit.
        for chunk in digits.as_bytes().chunks(9) {
            let mut piece = 0u64;
            let mut scale = 1u64;
            for &byte in chunk {
                piece = piece * 10 + u64::from(byte - b'0');
                scale *= 10;
            }
            let mut carry = piece;
            for digit in &mut value {
                let product = u64::from(*digit) * scale + carry;
                *digit = product as u32;
                carry = product >> 32;
            }
            while carry != 0 {
                value.push(carry as u32);
                carry >>= 32;
            }
        }
        Some(Big::from_parts(negative, value))
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// The number of bits needed for its size, 0 for zero.
    #[must_use]
    pub fn bits(&self) -> u64 {
        match self.digits.last() {
            Some(top) => (self.digits.len() as u64 - 1) * 32 + u64::from(32 - top.leading_zeros()),
            None => 0,
        }
    }

    /// How many bytes it takes, for counting against a script's memory cap (D14).
    #[must_use]
    pub fn size(&self) -> usize {
        self.digits.len() * 4
    }

    #[must_use]
    pub fn negated(&self) -> Big {
        Big::from_parts(!self.negative, self.digits.clone())
    }

    #[must_use]
    pub fn abs(&self) -> Big {
        Big { negative: false, digits: self.digits.clone() }
    }

    /// The value as an `i64`, when it fits.
    #[must_use]
    pub fn to_i64(&self) -> Option<i64> {
        let magnitude = self.to_u64_magnitude()?;
        if self.negative {
            if magnitude == 1 << 63 { Some(i64::MIN) } else { i64::try_from(magnitude).ok().map(|value| -value) }
        } else {
            i64::try_from(magnitude).ok()
        }
    }

    fn to_u64_magnitude(&self) -> Option<u64> {
        match self.digits.as_slice() {
            [] => Some(0),
            [low] => Some(u64::from(*low)),
            [low, high] => Some(u64::from(*low) | (u64::from(*high) << 32)),
            _ => None,
        }
    }

    /// The magnitude as a `u128`, when it fits.
    #[must_use]
    pub fn to_u128_magnitude(&self) -> Option<u128> {
        if self.digits.len() > 4 {
            return None;
        }
        Some(self.digits.iter().rev().fold(0u128, |value, &digit| (value << 32) | u128::from(digit)))
    }

    #[must_use]
    pub fn add(&self, other: &Big) -> Big {
        if self.negative == other.negative {
            return Big::from_parts(self.negative, add_digits(&self.digits, &other.digits));
        }
        match compare_digits(&self.digits, &other.digits) {
            Ordering::Less => Big::from_parts(other.negative, subtract_digits(&other.digits, &self.digits)),
            _ => Big::from_parts(self.negative, subtract_digits(&self.digits, &other.digits)),
        }
    }

    #[must_use]
    pub fn subtract(&self, other: &Big) -> Big {
        self.add(&other.negated())
    }

    #[must_use]
    pub fn multiply(&self, other: &Big) -> Big {
        Big::from_parts(self.negative != other.negative, multiply_digits(&self.digits, &other.digits))
    }

    /// Divides, the quotient rounded towards zero, with the remainder taking the dividend's sign; nothing when
    /// dividing by zero.
    #[must_use]
    pub fn divide_truncating(&self, other: &Big) -> Option<(Big, Big)> {
        if other.is_zero() {
            return None;
        }
        let (quotient, remainder) = divide_digits(&self.digits, &other.digits);
        Some((Big::from_parts(self.negative != other.negative, quotient), Big::from_parts(self.negative, remainder)))
    }

    /// Divides, the quotient rounded down, with the remainder taking the divisor's sign (Unio D70, D71).
    #[must_use]
    pub fn divide_floor(&self, other: &Big) -> Option<(Big, Big)> {
        let (quotient, remainder) = self.divide_truncating(other)?;
        if !remainder.is_zero() && remainder.negative != other.negative {
            Some((quotient.subtract(&Big::from_u64(1)), remainder.add(other)))
        } else {
            Some((quotient, remainder))
        }
    }

    /// Shifted left by `bits`: multiplied by 2^bits.
    #[must_use]
    pub fn shift_left(&self, bits: u64) -> Big {
        let whole = (bits / 32) as usize;
        let mut digits = vec![0u32; whole];
        digits.extend(shift_left_digits(&self.digits, (bits % 32) as u32));
        Big::from_parts(self.negative, digits)
    }

    /// Shifted right by `bits`, rounding the magnitude down.
    #[must_use]
    pub fn shift_right(&self, bits: u64) -> Big {
        let whole = (bits / 32) as usize;
        let rest = self.digits.get(whole..).unwrap_or(&[]);
        Big::from_parts(self.negative, shift_right_digits(rest, (bits % 32) as u32))
    }

    /// Whether any of the bits below `bits` is set.
    #[must_use]
    pub fn has_bits_below(&self, bits: u64) -> bool {
        let whole = (bits / 32) as usize;
        let part = (bits % 32) as u32;
        self.digits.iter().take(whole).any(|&digit| digit != 0)
            || (part > 0 && self.digits.get(whole).is_some_and(|&digit| digit & ((1u32 << part) - 1) != 0))
    }

    /// Raised to a whole power.
    #[must_use]
    pub fn power(&self, mut exponent: u64) -> Big {
        let mut result = Big::from_u64(1);
        let mut base = self.clone();
        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result.multiply(&base);
            }
            exponent >>= 1;
            if exponent > 0 {
                base = base.multiply(&base);
            }
        }
        result
    }

    /// The largest whole number whose square is at most this one, which must not be negative.
    #[must_use]
    pub fn square_root_floor(&self) -> Big {
        if self.is_zero() {
            return Big::zero();
        }
        // Newton's method from above: start at a power of two past the root.
        let mut guess = Big::from_u64(1).shift_left(self.bits().div_ceil(2));
        loop {
            let Some((quotient, _)) = self.divide_truncating(&guess) else { return Big::zero() };
            let next = guess.add(&quotient).shift_right(1);
            if next.compare(&guess) != Ordering::Less {
                return guess;
            }
            guess = next;
        }
    }

    #[must_use]
    pub fn compare(&self, other: &Big) -> Ordering {
        match (self.negative, other.negative) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => compare_digits(&self.digits, &other.digits),
            (true, true) => compare_digits(&other.digits, &self.digits),
        }
    }

    /// The number in decimal digits, with a `-` in front when negative.
    #[must_use]
    pub fn to_decimal(&self) -> String {
        if self.digits.is_empty() {
            return "0".to_owned();
        }
        // Nine decimal digits at a time.
        let mut pieces = Vec::new();
        let mut rest = self.digits.clone();
        while !rest.is_empty() {
            let (quotient, remainder) = divide_small(&rest, 1_000_000_000);
            pieces.push(remainder);
            rest = quotient;
        }
        let mut text = String::with_capacity(pieces.len() * 9 + 1);
        if self.negative {
            text.push('-');
        }
        for (index, piece) in pieces.iter().rev().enumerate() {
            if index == 0 {
                text.push_str(&piece.to_string());
            } else {
                let _ = write!(text, "{piece:09}");
            }
        }
        text
    }

    /// The lowest bit's value: whether the number is odd.
    #[must_use]
    pub fn is_odd(&self) -> bool {
        self.digits.first().is_some_and(|digit| digit & 1 == 1)
    }
}

impl PartialOrd for Big {
    fn partial_cmp(&self, other: &Big) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Big {
    fn cmp(&self, other: &Big) -> Ordering {
        self.compare(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big(text: &str) -> Big {
        Big::parse(text).unwrap_or_default()
    }

    #[test]
    fn reading_and_writing_decimal() {
        for text in ["0", "7", "-7", "4294967296", "-18446744073709551616", "123456789012345678901234567890"] {
            assert_eq!(big(text).to_decimal(), text);
        }
        assert_eq!(Big::parse("12a"), None);
        assert_eq!(Big::from_i64(i64::MIN).to_decimal(), "-9223372036854775808");
        assert_eq!(Big::from_i64(i64::MIN).to_i64(), Some(i64::MIN));
        assert_eq!(big("9223372036854775808").to_i64(), None);
    }

    #[test]
    fn arithmetic() {
        let a = big("123456789012345678901234567890");
        let b = big("-987654321098765432109876543210");
        assert_eq!(a.add(&b).to_decimal(), "-864197532086419753208641975320");
        assert_eq!(a.subtract(&b).to_decimal(), "1111111110111111111011111111100");
        assert_eq!(a.multiply(&b).to_decimal(), "-121932631137021795226185032733622923332237463801111263526900");
        let (quotient, remainder) = b.divide_truncating(&a).unwrap_or_default();
        assert_eq!(
            (quotient.to_decimal(), remainder.to_decimal()),
            ("-8".to_owned(), "-9000000000900000000090".to_owned())
        );
        let (quotient, remainder) = b.divide_floor(&a).unwrap_or_default();
        assert_eq!(
            (quotient.to_decimal(), remainder.to_decimal()),
            ("-9".to_owned(), "123456780012345678001234567800".to_owned())
        );
        assert_eq!(big("2").power(100).to_decimal(), "1267650600228229401496703205376");
        assert_eq!(big("1267650600228229401496703205376").square_root_floor().to_decimal(), "1125899906842624");
        assert_eq!(big("99").square_root_floor().to_decimal(), "9");
    }

    #[test]
    fn long_division_agrees_with_multiplication() {
        // Divisors whose second digit makes the first guess too big, the case Knuth's algorithm D corrects.
        let numbers = [
            "340282366920938463463374607431768211455",
            "18446744073709551615",
            "79228162514264337593543950335",
            "1000000000000000000000000000000000001",
            "4294967297",
            "123456789123456789123456789",
        ];
        for dividend in numbers {
            for divisor in numbers {
                let (a, b) = (big(dividend), big(divisor));
                let (quotient, remainder) = a.divide_truncating(&b).unwrap_or_default();
                assert_eq!(quotient.multiply(&b).add(&remainder), a, "{dividend} / {divisor}");
                assert_eq!(remainder.abs().compare(&b.abs()), Ordering::Less, "{dividend} / {divisor}");
            }
        }
    }
}

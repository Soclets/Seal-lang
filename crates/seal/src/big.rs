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

/// The fewest digits both sides have before Karatsuba's method beats the schoolbook one, measured on Tankun's Mac.
const KARATSUBA_DIGITS: usize = 64;

fn multiply_digits(left: &[u32], right: &[u32]) -> Vec<u32> {
    if left.len().min(right.len()) < KARATSUBA_DIGITS {
        return schoolbook_digits(left, right);
    }
    let mut out = vec![0u32; left.len() + right.len()];
    let (long, short) = if left.len() >= right.len() { (left, right) } else { (right, left) };
    // A long side is taken a short side's length at a time, so each part is balanced.
    for (index, part) in long.chunks(short.len()).enumerate() {
        let product =
            if part.len().min(short.len()) >= TOOM_DIGITS { toom_three(part, short) } else { karatsuba(part, short) };
        add_into(&mut out, &product, index * short.len());
    }
    trim(&mut out);
    out
}

/// The fewest digits both sides have before Toom-Cook's three-way method beats Karatsuba's, measured on Tankun's Mac.
const TOOM_DIGITS: usize = 256;

/// Toom-Cook's three-way method: each side split in three, five products of a third the size instead of nine, so its
/// time grows as the length to the power 1.46. The sides are evaluated at 0, 1, −1, −2 and infinity, and the
/// products put back together with Bodrato's sequence, whose divisions by 2 and 3 are always exact.
fn toom_three(left: &[u32], right: &[u32]) -> Vec<u32> {
    let third = left.len().max(right.len()).div_ceil(3);
    let part = |digits: &[u32], index: usize| {
        let from = (index * third).min(digits.len());
        let to = ((index + 1) * third).min(digits.len());
        Big::from_parts(false, digits.get(from..to).unwrap_or(&[]).to_vec())
    };
    let evaluate = |digits: &[u32]| {
        let (low, middle, high) = (part(digits, 0), part(digits, 1), part(digits, 2));
        let outer = low.add(&high);
        let at_one = outer.add(&middle);
        let at_minus_one = outer.subtract(&middle);
        let at_minus_two = at_minus_one.add(&high).shift_left(1).subtract(&low);
        [low, at_one, at_minus_one, at_minus_two, high]
    };
    let ([l0, l1, lm1, lm2, linf], [r0, r1, rm1, rm2, rinf]) = (evaluate(left), evaluate(right));
    let at_zero = l0.multiply(&r0);
    let at_one = l1.multiply(&r1);
    let at_minus_one = lm1.multiply(&rm1);
    let at_minus_two = lm2.multiply(&rm2);
    let at_infinity = linf.multiply(&rinf);
    let exact = |value: Big, divisor: u64| value.divide_truncating(&Big::from_u64(divisor)).unwrap_or_default().0;
    // Bodrato's interpolation, each line exact.
    let mut third_term = exact(at_minus_two.subtract(&at_one), 3);
    let first_half = exact(at_one.subtract(&at_minus_one), 2);
    let mut second_term = at_minus_one.subtract(&at_zero);
    third_term = exact(second_term.subtract(&third_term), 2).add(&at_infinity.shift_left(1));
    second_term = second_term.add(&first_half).subtract(&at_infinity);
    let first_term = first_half.subtract(&third_term);
    let shift = 32 * third as u64;
    let product = at_zero
        .add(&first_term.shift_left(shift))
        .add(&second_term.shift_left(2 * shift))
        .add(&third_term.shift_left(3 * shift))
        .add(&at_infinity.shift_left(4 * shift));
    product.digits
}

/// Karatsuba's method: three half-size products instead of four, so its time grows as the length to the power 1.58
/// rather than 2.
fn karatsuba(left: &[u32], right: &[u32]) -> Vec<u32> {
    if left.len().min(right.len()) < KARATSUBA_DIGITS {
        return schoolbook_digits(left, right);
    }
    let half = left.len().max(right.len()) / 2;
    let (left_low, left_high) = left.split_at(half.min(left.len()));
    let (right_low, right_high) = right.split_at(half.min(right.len()));
    let (mut left_low, mut right_low) = (left_low.to_vec(), right_low.to_vec());
    trim(&mut left_low);
    trim(&mut right_low);
    let low = multiply_digits(&left_low, &right_low);
    let high = multiply_digits(left_high, right_high);
    let middle = multiply_digits(&add_digits(&left_low, left_high), &add_digits(&right_low, right_high));
    // (low + high) is at most the middle product, so this takes nothing below zero.
    let middle = subtract_digits(&subtract_digits(&middle, &low), &high);
    let mut out = vec![0u32; left.len() + right.len() + 1];
    add_into(&mut out, &low, 0);
    add_into(&mut out, &middle, half);
    add_into(&mut out, &high, 2 * half);
    trim(&mut out);
    out
}

/// Adds `digits` into `out` from the place `offset` on; `out` has room for the sum.
fn add_into(out: &mut [u32], digits: &[u32], offset: usize) {
    let mut carry = 0u64;
    let mut index = offset;
    for &digit in digits {
        let Some(slot) = out.get_mut(index) else { return };
        let sum = u64::from(*slot) + u64::from(digit) + carry;
        *slot = sum as u32;
        carry = sum >> 32;
        index += 1;
    }
    while carry != 0 {
        let Some(slot) = out.get_mut(index) else { return };
        let sum = u64::from(*slot) + carry;
        *slot = sum as u32;
        carry = sum >> 32;
        index += 1;
    }
}

/// The schoolbook method, best for short numbers.
fn schoolbook_digits(left: &[u32], right: &[u32]) -> Vec<u32> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    let mut out = vec![0u32; left.len() + right.len()];
    for (i, &a) in left.iter().enumerate() {
        // Each row is added in from its own place, a pair of digits at a time with nothing to look up.
        let Some(row) = out.get_mut(i..) else { continue };
        let (row, rest) = row.split_at_mut(right.len().min(row.len()));
        let mut carry = 0u64;
        for (slot, &b) in row.iter_mut().zip(right) {
            let product = u64::from(a) * u64::from(b) + u64::from(*slot) + carry;
            *slot = product as u32;
            carry = product >> 32;
        }
        for slot in rest {
            if carry == 0 {
                break;
            }
            let sum = u64::from(*slot) + carry;
            *slot = sum as u32;
            carry = sum >> 32;
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

/// The fewest digits both a divisor and a quotient have before dividing through the divisor's reciprocal, worked
/// out with Newton's method, beats long division, measured on Tankun's Mac.
const NEWTON_DIGITS: usize = 768;

/// Extra bits the reciprocal is worked out to, past those the quotient needs, so it's off by a few at most.
const GUARD_BITS: u64 = 64;

/// The quotient and remainder of two digit lists, the divisor not zero.
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
    if divisor.len() >= NEWTON_DIGITS
        && dividend.len() - divisor.len() >= NEWTON_DIGITS
        && let Some(divided) = divide_by_reciprocal(dividend, divisor)
    {
        return divided;
    }
    long_division(dividend, divisor)
}

/// Divides through the divisor's reciprocal, so it takes about as long as a few multiplications: the quotient is
/// worked out to within a few, then put right exactly against the remainder. Nothing, for long division to take
/// over, if putting it right takes more than a few steps, which it never should.
fn divide_by_reciprocal(dividend: &[u32], divisor: &[u32]) -> Option<(Vec<u32>, Vec<u32>)> {
    let dividend = Big::from_parts(false, dividend.to_vec());
    let divisor = Big::from_parts(false, divisor.to_vec());
    let divisor_bits = divisor.bits();
    // The quotient has at most this many bits.
    let quotient_bits = dividend.bits().saturating_sub(divisor_bits) + 1;
    let precision = quotient_bits + GUARD_BITS;
    // About 2^(divisor_bits + precision) / divisor.
    let reciprocal = Big::reciprocal(&divisor, precision);
    // The dividend's lowest bits change the quotient by less than one, so only its top ones are multiplied.
    let dropped = divisor_bits.saturating_sub(2 * GUARD_BITS);
    let top = dividend.shift_right(dropped);
    let mut quotient = top.multiply(&reciprocal).shift_right(divisor_bits + precision - dropped);
    let mut remainder = dividend.subtract(&quotient.multiply(&divisor));
    let one = Big::from_u64(1);
    for _ in 0..8 {
        if remainder.negative {
            quotient = quotient.subtract(&one);
            remainder = remainder.add(&divisor);
        } else if compare_digits(&remainder.digits, &divisor.digits) != Ordering::Less {
            quotient = quotient.add(&one);
            remainder = remainder.subtract(&divisor);
        } else {
            return Some((quotient.digits, remainder.digits));
        }
    }
    None
}

/// Long division, Knuth's algorithm D: the quotient and remainder of two digit lists, the divisor at least two
/// digits long.
fn long_division(dividend: &[u32], divisor: &[u32]) -> (Vec<u32>, Vec<u32>) {
    if compare_digits(dividend, divisor) == Ordering::Less {
        return (Vec::new(), dividend.to_vec());
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

/// The most base 2^32 digits a number has and is still written or read nine decimal digits at a time; past it,
/// halves are, split at a power of ten, so the time grows as a multiplication's does rather than with the square
/// of the length. Measured on Tankun's Mac.
const SHORT_TEXT_DIGITS: usize = 48;

/// 10^(9·2^level), squared from the level below and kept for the next use in the same conversion.
fn power_of_ten(level: usize, powers: &mut Vec<Big>) -> Big {
    while powers.len() <= level {
        let next = match powers.last() {
            Some(below) => below.multiply(below),
            None => Big::from_u64(1_000_000_000),
        };
        powers.push(next);
    }
    powers.get(level).cloned().unwrap_or_default()
}

/// How many decimal digits 10^(9·2^level) splits off.
fn split_width(level: usize) -> usize {
    9usize.checked_shl(level as u32).unwrap_or(usize::MAX)
}

/// Writes a magnitude's decimal digits, padded with zeros to `width` when one is given.
fn write_decimal(digits: &[u32], width: Option<usize>, powers: &mut Vec<Big>, text: &mut String) {
    if digits.len() <= SHORT_TEXT_DIGITS {
        let short = Big { negative: false, digits: digits.to_vec() }.short_decimal();
        let short = if width.is_some() && digits.is_empty() { String::new() } else { short };
        for _ in short.len()..width.unwrap_or(0) {
            text.push('0');
        }
        text.push_str(&short);
        return;
    }
    // The split is at the largest power whose square isn't much past the number, so the halves are about even.
    let bits = (digits.len() as u64) * 32;
    let mut level = 0;
    while 2 * ((split_width(level + 1) as u64) * 10 / 3) <= bits {
        level += 1;
    }
    let power = power_of_ten(level, powers);
    let (high, low) = divide_digits(digits, &power.digits);
    let low_width = split_width(level);
    // The split is below the number, so the high half is never zero when no width is asked for.
    if !(high.is_empty() && width.is_none()) {
        write_decimal(&high, width.map(|width| width.saturating_sub(low_width)), powers, text);
    }
    write_decimal(&low, Some(low_width), powers, text);
}

/// Reads decimal digits into a magnitude: a short run nine at a time, a long one as halves joined by a power of ten.
fn read_decimal(digits: &[u8], powers: &mut Vec<Big>) -> Vec<u32> {
    if digits.len() <= SHORT_TEXT_DIGITS * 9 {
        return Big::read_short(digits);
    }
    let mut level = 0;
    while split_width(level + 1) < digits.len() {
        level += 1;
    }
    let (high, low) = digits.split_at(digits.len() - split_width(level));
    let high = read_decimal(high, powers);
    let low = read_decimal(low, powers);
    let power = power_of_ten(level, powers);
    let mut joined = multiply_digits(&high, &power.digits);
    joined.resize(joined.len().max(low.len()) + 1, 0);
    add_into(&mut joined, &low, 0);
    trim(&mut joined);
    joined
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
        Some(Big::from_parts(negative, read_decimal(digits.as_bytes(), &mut Vec::new())))
    }

    /// The number of decimal digits short enough to read nine at a time; a longer number is read in halves.
    fn read_short(digits: &[u8]) -> Vec<u32> {
        let mut value: Vec<u32> = Vec::new();
        // Nine decimal digits at a time fit in a base 2^32 digit.
        for chunk in digits.chunks(9) {
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
        value
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

    /// About 2^(bits + precision) / self, for a positive number of `bits` bits, off by a few at most: worked out at
    /// half the precision first, then doubled with one step of Newton's method, y + y·(1 − self·y), so it takes about
    /// as long as a few multiplications at full precision.
    fn reciprocal(&self, precision: u64) -> Big {
        let bits = self.bits();
        // Only the number's top bits matter at this precision; 2^(bits + p) / self is the same as for its top part.
        let keep = precision + 2 * GUARD_BITS;
        let top = if bits > keep { self.shift_right(bits - keep) } else { self.clone() };
        let top_bits = top.bits();
        if precision <= 32 * NEWTON_DIGITS as u64 {
            let power = Big::from_u64(1).shift_left(top_bits + precision);
            return Big::from_parts(false, long_division(&power.digits, &top.digits).0);
        }
        let half = precision / 2 + GUARD_BITS;
        let rough = top.reciprocal(half);
        // The error left by the rough reciprocal, at its own scale: 2^(top_bits + half) − top·rough.
        let error = Big::from_u64(1).shift_left(top_bits + half).subtract(&top.multiply(&rough));
        let correction = rough.multiply(&error).shift_right(top_bits + 2 * half - precision);
        rough.shift_left(precision - half).add(&correction)
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
        // Newton's method from above, from a power of two past the root for a short number. A long one starts from
        // the root of its top half, which already has half the root's digits right, so a step or two at full length
        // finish it rather than one for every doubling.
        let bits = self.bits();
        let mut guess = if bits <= 4 * 32 * KARATSUBA_DIGITS as u64 {
            Big::from_u64(1).shift_left(bits.div_ceil(2))
        } else {
            let quarter = bits / 4;
            // (top + 1)·2^quarter is past the root, since (top + 1)² is past self / 4^quarter.
            let top = self.shift_right(2 * quarter).square_root_floor();
            top.add(&Big::from_u64(1)).shift_left(quarter)
        };
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
        let mut text = String::with_capacity(self.digits.len() * 10 + 1);
        if self.negative {
            text.push('-');
        }
        write_decimal(&self.digits, None, &mut Vec::new(), &mut text);
        text
    }

    /// A short number's decimal digits, nine at a time.
    fn short_decimal(&self) -> String {
        if self.digits.is_empty() {
            return "0".to_owned();
        }
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
        // A long number's root, started from its top half's, is still the exact floor.
        let mut seed = 0x1357_9bdf_2468_ace0;
        for count in [300, 1000, 2500] {
            let root = Big::from_parts(false, random_digits(&mut seed, count));
            let square = root.multiply(&root);
            assert_eq!(square.square_root_floor(), root, "{count} digits");
            assert_eq!(square.add(&root).add(&root).square_root_floor(), root, "{count} digits, just below the next");
            assert_eq!(square.subtract(&Big::from_u64(1)).square_root_floor(), root.subtract(&Big::from_u64(1)));
        }
    }

    /// Random digits from a fixed seed, so a failure can be run again.
    fn random_digits(seed: &mut u64, count: usize) -> Vec<u32> {
        let mut digits = Vec::with_capacity(count);
        for _ in 0..count {
            *seed ^= *seed << 13;
            *seed ^= *seed >> 7;
            *seed ^= *seed << 17;
            // Now and then all ones or zeros, where carries and borrows run the furthest.
            digits.push(match *seed % 8 {
                0 => u32::MAX,
                1 => 0,
                _ => *seed as u32,
            });
        }
        trim(&mut digits);
        digits
    }

    #[test]
    fn faster_multiplication_agrees_with_the_schoolbook() {
        let mut seed = 0x9e37_79b9_7f4a_7c15;
        for (left, right) in [
            (32, 32),
            (33, 47),
            (64, 64),
            (100, 37),
            (257, 255),
            (300, 31),
            (513, 129),
            (700, 700),
            (769, 1000),
            (2000, 2000),
        ] {
            for _ in 0..4 {
                let (a, b) = (random_digits(&mut seed, left), random_digits(&mut seed, right));
                assert_eq!(multiply_digits(&a, &b), schoolbook_digits(&a, &b), "{left} × {right}");
            }
        }
    }

    #[test]
    fn dividing_through_the_reciprocal_agrees_with_long_division() {
        let mut seed = 0x2545_f491_4f6c_dd1d;
        for (dividend, divisor) in [(130, 64), (200, 65), (300, 100), (513, 256), (1000, 500), (1200, 70), (2100, 1000)]
        {
            for _ in 0..3 {
                let (a, b) = (random_digits(&mut seed, dividend), random_digits(&mut seed, divisor));
                let fast = divide_by_reciprocal(&a, &b);
                assert!(fast.is_some(), "{dividend} / {divisor} fell back to long division");
                assert_eq!(fast, Some(long_division(&a, &b)), "{dividend} / {divisor}");
            }
        }
    }

    #[test]
    fn long_numbers_go_to_text_and_back() {
        let mut seed = 0x5851_f42d_4c95_7f2d;
        for count in [48, 49, 60, 100, 333, 1000, 2500] {
            for _ in 0..3 {
                let digits = random_digits(&mut seed, count);
                let number = Big::from_parts(seed % 2 == 0, digits.clone());
                let text = number.to_decimal();
                assert_eq!(text, number.short_decimal(), "{count} digits");
                assert_eq!(Big::parse(&text), Some(number), "{count} digits");
            }
        }
        // Zeros where the halves meet are kept.
        let round = Big::from_u64(10).power(2000).add(&Big::from_u64(7));
        assert_eq!(round.to_decimal(), format!("1{}7", "0".repeat(1999)));
        assert_eq!(Big::parse(&round.to_decimal()), Some(round));
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

//! A whole number of up to 256 bits, in four 64-bit limbs, for `Decimal`'s everyday operations: the exact sum or
//! product of two decimal128 coefficients always fits, so it can be rounded to 34 digits with nothing allocated
//! (D116, Building rule 5).
// The limbs are halves and quarters of `u128`s, and the counts of digits are tiny.
#![allow(clippy::cast_possible_truncation)]

use std::cmp::Ordering;

/// A whole number below 2^256, least significant limb first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wide([u64; 4]);

/// 10^0 to 10^77, the powers of ten below 2^256.
const POWERS: [Wide; 78] = {
    let mut powers = [Wide([0; 4]); 78];
    powers[0] = Wide([1, 0, 0, 0]);
    let mut index = 1;
    while index < 78 {
        let previous = powers[index - 1].0;
        let mut limbs = [0u64; 4];
        let mut carry = 0u128;
        let mut limb = 0;
        while limb < 4 {
            let product = previous[limb] as u128 * 10 + carry;
            limbs[limb] = product as u64;
            carry = product >> 64;
            limb += 1;
        }
        powers[index] = Wide(limbs);
        index += 1;
    }
    powers
};

impl Wide {
    #[must_use]
    pub fn from_u128(value: u128) -> Wide {
        Wide([value as u64, (value >> 64) as u64, 0, 0])
    }

    /// The value, when it fits a `u128`.
    #[must_use]
    pub fn to_u128(self) -> Option<u128> {
        (self.0[2] == 0 && self.0[3] == 0).then(|| u128::from(self.0[0]) | (u128::from(self.0[1]) << 64))
    }

    /// `left × right`, which always fits.
    #[must_use]
    pub fn product(left: u128, right: u128) -> Wide {
        let (a, b) = ([left as u64, (left >> 64) as u64], [right as u64, (right >> 64) as u64]);
        let mut limbs = [0u64; 4];
        for (i, &x) in a.iter().enumerate() {
            let mut carry = 0u128;
            for (j, &y) in b.iter().enumerate() {
                let slot = &mut limbs[i + j];
                let sum = u128::from(x) * u128::from(y) + u128::from(*slot) + carry;
                *slot = sum as u64;
                carry = sum >> 64;
            }
            limbs[i + 2] = carry as u64;
        }
        Wide(limbs)
    }

    /// `self × by`, or nothing past 2^256.
    #[must_use]
    pub fn times(self, by: u64) -> Option<Wide> {
        let mut limbs = [0u64; 4];
        let mut carry = 0u128;
        for (slot, &limb) in limbs.iter_mut().zip(&self.0) {
            let product = u128::from(limb) * u128::from(by) + carry;
            *slot = product as u64;
            carry = product >> 64;
        }
        (carry == 0).then_some(Wide(limbs))
    }

    /// `self × 10^power`, or nothing past 2^256.
    #[must_use]
    pub fn times_ten_to(self, mut power: u32) -> Option<Wide> {
        let mut value = self;
        while power > 0 {
            let step = power.min(19);
            value = value.times(POWERS[step as usize].0[0])?;
            power -= step;
        }
        Some(value)
    }

    /// `self + other`, or nothing past 2^256.
    #[must_use]
    pub fn plus(self, other: Wide) -> Option<Wide> {
        let mut limbs = [0u64; 4];
        let mut carry = false;
        for ((slot, &x), &y) in limbs.iter_mut().zip(&self.0).zip(&other.0) {
            let (sum, first) = x.overflowing_add(y);
            let (sum, second) = sum.overflowing_add(u64::from(carry));
            *slot = sum;
            carry = first || second;
        }
        (!carry).then_some(Wide(limbs))
    }

    /// `self - other`, where `other` is at most `self`.
    #[must_use]
    pub fn minus(self, other: Wide) -> Wide {
        let mut limbs = [0u64; 4];
        let mut borrow = false;
        for ((slot, &x), &y) in limbs.iter_mut().zip(&self.0).zip(&other.0) {
            let (difference, first) = x.overflowing_sub(y);
            let (difference, second) = difference.overflowing_sub(u64::from(borrow));
            *slot = difference;
            borrow = first || second;
        }
        Wide(limbs)
    }

    /// `self ÷ by` and what's left, for `by` above 0.
    #[must_use]
    fn divide_small(self, by: u64) -> (Wide, u64) {
        let mut limbs = [0u64; 4];
        let mut remainder = 0u128;
        for (slot, &limb) in limbs.iter_mut().zip(&self.0).rev() {
            let current = (remainder << 64) | u128::from(limb);
            *slot = (current / u128::from(by)) as u64;
            remainder = current % u128::from(by);
        }
        (Wide(limbs), remainder as u64)
    }

    /// `self ÷ 10^power` and what's left, for a power up to 38, whose remainder fits a `u128`.
    #[must_use]
    pub fn divide_ten_to(self, power: u32) -> (Wide, u128) {
        let mut value = self;
        let mut remainder = 0u128;
        let mut scale = 1u128;
        let mut left = power;
        while left > 0 {
            let step = left.min(19);
            let (quotient, part) = value.divide_small(POWERS[step as usize].0[0]);
            remainder += u128::from(part) * scale;
            scale *= u128::from(POWERS[step as usize].0[0]);
            value = quotient;
            left -= step;
        }
        (value, remainder)
    }

    /// How many decimal digits it has; 0 for zero.
    #[must_use]
    pub fn digits(self) -> u32 {
        let bits = 256 - self.leading_zeros();
        // log10(2) is a little over 0.30103, so this is the count or one less.
        let estimate = (bits * 30_103 / 100_000) as usize;
        match POWERS.get(estimate) {
            Some(&power) if self.compare(power) != Ordering::Less => estimate as u32 + 1,
            _ => estimate as u32,
        }
    }

    fn leading_zeros(self) -> u32 {
        let mut zeros = 0;
        for &limb in self.0.iter().rev() {
            if limb != 0 {
                return zeros + limb.leading_zeros();
            }
            zeros += 64;
        }
        zeros
    }

    #[must_use]
    pub fn compare(self, other: Wide) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_whole_numbers_agree_with_u128_maths() {
        let a = 9_999_999_999_999_999_999_999_999_999_999_999u128;
        let product = Wide::product(a, a);
        assert_eq!(product.digits(), 68);
        let (quotient, remainder) = product.divide_ten_to(34);
        assert_eq!(quotient.to_u128(), Some(a - 1));
        assert_eq!(remainder, 1);
        assert_eq!(Wide::from_u128(0).digits(), 0);
        assert_eq!(Wide::from_u128(9).digits(), 1);
        assert_eq!(Wide::from_u128(10).digits(), 2);
        let scaled = Wide::from_u128(123).times_ten_to(40).unwrap_or_default();
        assert_eq!(scaled.digits(), 43);
        assert_eq!(scaled.plus(Wide::from_u128(7)).map(|sum| sum.divide_ten_to(40)), Some((Wide::from_u128(123), 7)));
        let ten_to_40 = Wide::from_u128(1).times_ten_to(40).unwrap_or_default();
        assert_eq!(ten_to_40.minus(Wide::from_u128(1)).digits(), 40);
        assert_eq!(ten_to_40.digits(), 41);
    }
}

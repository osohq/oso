use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;
use std::num::FpCategory;
use std::ops::{Add, Div, Mul, Sub};

use rand::prelude::*;

use super::types::*;

impl Add for Numeric {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => a.checked_add(b).map(Numeric::Integer),
            (Numeric::Integer(a), Numeric::Float(b)) => Some(Numeric::Float(a as f64 + b)),
            (Numeric::Float(a), Numeric::Integer(b)) => Some(Numeric::Float(a + b as f64)),
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(a + b)),
        }
    }
}

impl Sub for Numeric {
    type Output = Option<Self>;

    fn sub(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => a.checked_sub(b).map(Numeric::Integer),
            (Numeric::Integer(a), Numeric::Float(b)) => Some(Numeric::Float(a as f64 - b)),
            (Numeric::Float(a), Numeric::Integer(b)) => Some(Numeric::Float(a - b as f64)),
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(a - b)),
        }
    }
}

impl Mul for Numeric {
    type Output = Option<Self>;

    fn mul(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => a.checked_mul(b).map(Numeric::Integer),
            (Numeric::Integer(a), Numeric::Float(b)) => Some(Numeric::Float(a as f64 * b)),
            (Numeric::Float(a), Numeric::Integer(b)) => Some(Numeric::Float(a * b as f64)),
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(a * b)),
        }
    }
}

impl Div for Numeric {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => Some(Numeric::Float(a as f64 / b as f64)),
            (Numeric::Integer(a), Numeric::Float(b)) => Some(Numeric::Float(a as f64 / b)),
            (Numeric::Float(a), Numeric::Integer(b)) => Some(Numeric::Float(a / b as f64)),
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(a / b)),
        }
    }
}

impl PartialEq for Numeric {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Equal))
    }
}

impl Eq for Numeric {}

/// There are 53 bits of mantissa in an IEEE 754 double precision float.
const MOST_POSITIVE_EXACT_FLOAT: i64 = 1 << 53;

/// -i64::MIN is 2**63. The maximum positive i64 is 2**63 - 1, but this
/// isn't representable as a double. So, we first cast i64::MIN to f64
/// then flip the sign to get 2 ** 63.
const MOST_POSITIVE_I64_FLOAT: f64 = -(i64::MIN as f64);
const MOST_NEGATIVE_I64_FLOAT: f64 = i64::MIN as f64;

impl Hash for Numeric {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Numeric::Integer(i) => {
                discriminant(self).hash(state);
                *i as u64
            }
            Numeric::Float(f) => match f.classify() {
                FpCategory::Zero => {
                    // Canonicalize zero representations.
                    discriminant(&Numeric::Integer(0)).hash(state);
                    0u64
                }
                FpCategory::Nan => {
                    // Randomize NaN hashes so they always miss.
                    discriminant(&FpCategory::Nan).hash(state);
                    f64::from_bits(f.to_bits() | random::<u64>()).to_bits()
                }
                FpCategory::Infinite | FpCategory::Subnormal => {
                    // Infinities and subnormals are canonical.
                    discriminant(self).hash(state);
                    f.to_bits()
                }
                FpCategory::Normal => {
                    // Hash floats the same as numerically equal integers.
                    if f.fract() == 0.0 {
                        if MOST_NEGATIVE_I64_FLOAT <= *f && *f < MOST_POSITIVE_I64_FLOAT {
                            // The integral part of the float is representable as an i64.
                            discriminant(&Numeric::Integer(0)).hash(state);
                            (*f as i64) as u64
                        } else {
                            // The magnitude of the float is greater than any representable integer.
                            discriminant(self).hash(state);
                            f.to_bits()
                        }
                    } else {
                        // The number is not an integer.
                        discriminant(self).hash(state);
                        f.to_bits()
                    }
                }
            },
        }
        .hash(state)
    }
}

impl PartialOrd for Numeric {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Compare the integer `i` with the float `f`.
        // Adapted from MongoDB's `compareLongToDouble`.
        let partial_cmp = |i: i64, f: f64| {
            if f.is_nan() {
                None
            } else if -MOST_POSITIVE_EXACT_FLOAT < i && i < MOST_POSITIVE_EXACT_FLOAT {
                // The integer is exactly representable as a float.
                (i as f64).partial_cmp(&f)
            } else if f >= MOST_POSITIVE_I64_FLOAT {
                // The float is greater than any representable integer.
                Some(Ordering::Less)
            } else if f < MOST_NEGATIVE_I64_FLOAT {
                // The float is less than any representable integer.
                Some(Ordering::Greater)
            } else {
                // The integral part of the float is representable as an i64.
                // Floats in this range do not have any fractional components.
                i.partial_cmp(&(f as i64))
            }
        };
        match (*self, *other) {
            (Self::Integer(left), Self::Integer(right)) => left.partial_cmp(&right),
            (Self::Integer(i), Self::Float(f)) => partial_cmp(i, f),
            (Self::Float(f), Self::Integer(i)) => partial_cmp(i, f).map(Ordering::reverse),
            (Self::Float(left), Self::Float(right)) => left.partial_cmp(&right),
        }
    }
}

impl From<i64> for Numeric {
    fn from(other: i64) -> Self {
        Self::Integer(other)
    }
}
impl From<f64> for Numeric {
    fn from(other: f64) -> Self {
        Self::Float(other)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::hash_map::DefaultHasher;

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    /// Test mixed comparison of longs & doubles.
    fn test_mixed_comparison() {
        // Nothing compares equal to NaN.
        assert!(Numeric::Integer(1) != Numeric::Float(f64::NAN));
        assert!(Numeric::Integer(-1) != Numeric::Float(f64::NAN));
        assert!(!(Numeric::Integer(1) < Numeric::Float(f64::NAN)));
        assert!(!(Numeric::Integer(1) > Numeric::Float(f64::NAN)));
        assert!(!(Numeric::Integer(-1) > Numeric::Float(f64::NAN)));
        assert!(Numeric::Float(f64::NAN) != Numeric::Float(f64::NAN));

        // All zeros equal.
        assert!(Numeric::Integer(0) == Numeric::Float(0.0));
        assert!(Numeric::Integer(0) == Numeric::Float(-0.0));
        assert!(Numeric::Float(0.0) == Numeric::Float(-0.0));

        // Infinity to int compares greater than all ints.
        assert!(Numeric::Integer(1) < Numeric::Float(f64::INFINITY));
        assert!(Numeric::Integer(i64::MAX) < Numeric::Float(f64::INFINITY));
        assert!(Numeric::Integer(i64::MIN) < Numeric::Float(f64::INFINITY));
        assert!(Numeric::Integer(i64::MIN) > Numeric::Float(f64::NEG_INFINITY));
        assert!(Numeric::Integer(0) > Numeric::Float(f64::NEG_INFINITY));
        assert!(Numeric::Integer(i64::MAX) > Numeric::Float(f64::NEG_INFINITY));

        // Float representable as long compares correctly.
        assert!(Numeric::Integer(1) == Numeric::Float(1.0));
        assert!(Numeric::Integer(-1) != Numeric::Float(1.0));
        assert!(Numeric::Integer(2) > Numeric::Float(1.0));
        assert!(Numeric::Integer(-2) < Numeric::Float(1.0));
        assert!(Numeric::Integer(1 << 52) == Numeric::Float((2.0 as f64).powi(52)));
        assert!(Numeric::Integer(1 << 53) == Numeric::Float((2.0 as f64).powi(53)));
        assert!(Numeric::Integer((1 << 52) + 1) == Numeric::Float((2.0 as f64).powi(52) + 1.0));
        assert!(Numeric::Integer(1 << 52) < Numeric::Float((2.0 as f64).powi(52) + 1.0));
        assert!(Numeric::Integer((1 << 52) + 1) > Numeric::Float((2.0 as f64).powi(52)));
        assert!(Numeric::Integer(-(1 << 52) - 1) < Numeric::Float(-(2.0 as f64).powi(52)));

        // Long not exactly representable as float compares correctly.
        assert!(Numeric::Integer((1 << 53) + 1) > Numeric::Float((2.0 as f64).powi(53)));
        assert!(Numeric::Integer((1 << 53) - 1) == Numeric::Float((2.0 as f64).powi(53) - 1.0));
        assert!(Numeric::Integer(-(1 << 53) - 1) < Numeric::Float(-(2.0 as f64).powi(53)));
        assert!(Numeric::Integer(-(1 << 54)) < Numeric::Float(-(2.0 as f64).powi(53)));
        assert!(Numeric::Integer(1 << 54) > Numeric::Float((2.0 as f64).powi(53)));
        assert!(Numeric::Integer(1 << 56) > Numeric::Float((2.0 as f64).powi(54)));

        // Float larger than max long compares correctly
        assert!(Numeric::Integer(1 << 56) < Numeric::Float((2.0 as f64).powi(70)));

        // Float smaller than min long compares correctly.
        assert!(Numeric::Integer(1 << 56) > Numeric::Float(-(2.0 as f64).powi(70)));
        assert!(Numeric::Integer(-(1 << 56)) > Numeric::Float(-(2.0 as f64).powi(70)));
        assert!(Numeric::Integer(i64::MIN) > Numeric::Float(-(2.0 as f64).powi(70)));
        assert!(Numeric::Integer(i64::MAX) < Numeric::Float((2.0 as f64).powi(65) + 3.1));

        // i64 max is 2 ** 63 - 1. This value is not representable as a f64.
        assert!(Numeric::Integer(i64::MAX) < Numeric::Float((2.0 as f64).powi(63)));
        // 2 ** 63 - 2 ** 10 is the next representable float down
        assert!(Numeric::Integer(i64::MAX) > Numeric::Float((2.0 as f64).powi(63) - 1024.0));
        // 2 ** 63 + 2 ** 11 is the next representable float up
        assert!(Numeric::Integer(i64::MAX) < Numeric::Float((2.0 as f64).powi(63) + 2048.0));

        // i64 min is 2 ** 63. This value is exactly representable as a f64.
        assert!(Numeric::Integer(i64::MIN) == Numeric::Float(-(2.0 as f64).powi(63)));
        // next value down is 2 ** 63 - 2048
        assert!(Numeric::Integer(i64::MIN) > Numeric::Float(-(2.0 as f64).powi(63) - 2048.0));
        // next value up is 2 ** 63 + 1024
        assert!(Numeric::Integer(i64::MIN) < Numeric::Float(-(2.0 as f64).powi(63) + 1024.0));

        assert!(Numeric::Integer(i64::MIN) < Numeric::Float(-(2.0 as f64).powi(62)));
        assert!(Numeric::Integer(i64::MIN) > Numeric::Float(-(2.0 as f64).powi(65)));

        // Long exactly representable as float compares correctly
        assert!(Numeric::Integer(2) == Numeric::Float(2.0));
        assert!(Numeric::Integer(2) < Numeric::Float(2.1));
        // 2 * epsilon here since 2 takes up 2 bits of the mantissa, so 2.0 + e == 2.0.
        assert!(Numeric::Integer(2) < Numeric::Float(2.0 + 2.0 * f64::EPSILON));
        assert!(Numeric::Integer(2) > Numeric::Float(2.0 - 2.0 * f64::EPSILON));
        assert!(Numeric::Integer(1) < Numeric::Float(1.0 + f64::EPSILON));
        assert!(Numeric::Integer(1) > Numeric::Float(1.0 - f64::EPSILON));
        assert!(Numeric::Integer(2) < Numeric::Float(3.0));
    }

    #[test]
    fn test_numeric_hash() {
        assert_ne!(
            hash(&Numeric::Float(f64::NAN)),
            hash(&Numeric::Float(f64::NAN))
        );
        assert_eq!(hash(&Numeric::Float(1.0)), hash(&Numeric::Float(1.0)));
        assert_eq!(hash(&Numeric::Float(0.0)), hash(&Numeric::Float(-0.0)));
        assert_eq!(
            hash(&Numeric::Float(f64::INFINITY)),
            hash(&Numeric::Float(f64::INFINITY))
        );
        assert_ne!(
            hash(&Numeric::Float(f64::INFINITY)),
            hash(&Numeric::Float(f64::NEG_INFINITY))
        );
        assert_eq!(
            hash(&Numeric::Float(f64::NEG_INFINITY)),
            hash(&Numeric::Float(f64::NEG_INFINITY))
        );

        assert_eq!(hash(&Numeric::Integer(0)), hash(&Numeric::Float(0.0)));
        assert_eq!(hash(&Numeric::Integer(1)), hash(&Numeric::Float(1.0)));
        assert_ne!(hash(&Numeric::Integer(-1)), hash(&Numeric::Float(1.0)));
        assert_eq!(hash(&Numeric::Integer(-1)), hash(&Numeric::Float(-1.0)));
    }
}

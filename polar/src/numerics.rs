use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Sub};

use super::types::*;
use ordered_float::OrderedFloat;

impl Add for Numeric {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => a.checked_add(b).map(Numeric::Integer),
            (Numeric::Integer(a), Numeric::Float(b)) => {
                Some(Numeric::Float(OrderedFloat(a as f64 + b.0)))
            }
            (Numeric::Float(a), Numeric::Integer(b)) => {
                Some(Numeric::Float(OrderedFloat(a.0 + b as f64)))
            }
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(OrderedFloat(a.0 + b.0))),
        }
    }
}

impl Sub for Numeric {
    type Output = Option<Self>;

    fn sub(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => a.checked_sub(b).map(Numeric::Integer),
            (Numeric::Integer(a), Numeric::Float(b)) => {
                Some(Numeric::Float(OrderedFloat(a as f64 - b.0)))
            }
            (Numeric::Float(a), Numeric::Integer(b)) => {
                Some(Numeric::Float(OrderedFloat(a.0 - b as f64)))
            }
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(OrderedFloat(a.0 - b.0))),
        }
    }
}

impl Mul for Numeric {
    type Output = Option<Self>;

    fn mul(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => a.checked_mul(b).map(Numeric::Integer),
            (Numeric::Integer(a), Numeric::Float(b)) => {
                Some(Numeric::Float(OrderedFloat(a as f64 * b.0)))
            }
            (Numeric::Float(a), Numeric::Integer(b)) => {
                Some(Numeric::Float(OrderedFloat(a.0 * b as f64)))
            }
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(OrderedFloat(a.0 * b.0))),
        }
    }
}

impl Div for Numeric {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => {
                if b == 0 {
                    None
                } else {
                    Some(Numeric::Float(OrderedFloat(a as f64 / b as f64)))
                }
            }
            (Numeric::Integer(a), Numeric::Float(b)) => {
                if b.0 == 0.0 {
                    None
                } else {
                    Some(Numeric::Float(OrderedFloat(a as f64 / b.0)))
                }
            }
            (Numeric::Float(a), Numeric::Integer(b)) => {
                if b == 0 {
                    None
                } else {
                    Some(Numeric::Float(OrderedFloat(a.0 / b as f64)))
                }
            }
            (Numeric::Float(a), Numeric::Float(b)) => {
                if b.0 == 0.0 {
                    None
                } else {
                    Some(Numeric::Float(OrderedFloat(a.0 / b.0)))
                }
            }
        }
    }
}

impl PartialEq for Numeric {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Equal))
    }
}

/// There are 53 bits of mantissa in an IEEE 754 double precision float.
const MOST_POSITIVE_EXACT_FLOAT: i64 = 1 << 53;

/// Floats larger than this are not representable as signed 64-bit integers.
const MOST_POSITIVE_INTEGER: i64 = 0x7fffffffffffffff;

impl PartialOrd for Numeric {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Compare the integer `i` with the float `f`.
        // Adapted from MongoDB's `compareLongToDouble`.
        let partial_cmp = |i: i64, f: Float| {
            if f.is_nan() {
                None
            } else if -MOST_POSITIVE_EXACT_FLOAT < i && i < MOST_POSITIVE_EXACT_FLOAT {
                // The integer is exactly representable as a float.
                (i as f64).partial_cmp(&f)
            } else if f > OrderedFloat(MOST_POSITIVE_INTEGER as f64) {
                // The float is greater than any representable integer.
                Some(Ordering::Less)
            } else if f < OrderedFloat(-MOST_POSITIVE_INTEGER as f64) {
                // The float is less than any representable integer.
                Some(Ordering::Greater)
            } else {
                // The integral part of the float is representable as an integer.
                i.partial_cmp(&(f.0 as i64))
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
        Self::Float(other.into())
    }
}

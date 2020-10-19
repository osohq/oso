use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;
use std::num::FpCategory;
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Numeric {
    Integer(i64),

    #[serde(
        serialize_with = "serialize_float",
        deserialize_with = "deserialize_float"
    )]
    Float(f64),
}

/// Since JSON does not support ±∞ or NaN (RFC 8259 §6),
/// we encode them as magic strings.
fn serialize_float<S>(f: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match f.classify() {
        FpCategory::Nan => s.serialize_str("NaN"),
        FpCategory::Infinite => s.serialize_str(if *f == f64::INFINITY {
            "Infinity"
        } else {
            "-Infinity"
        }),
        FpCategory::Zero | FpCategory::Subnormal | FpCategory::Normal => s.serialize_f64(*f),
    }
}

/// Decode a magic ±∞ or NaN value.
fn deserialize_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct FloatVisitor;

    impl<'de> de::Visitor<'de> for FloatVisitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("JSON encoded data")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "Infinity" => Ok(f64::INFINITY),
                "-Infinity" => Ok(f64::NEG_INFINITY),
                "NaN" => Ok(f64::NAN),
                _ => Err(de::Error::custom("invalid float")),
            }
        }
    }

    deserializer.deserialize_any(FloatVisitor)
}

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

impl Numeric {
    pub fn modulo(self, modulus: Self) -> Option<Self> {
        fn modulo(a: f64, b: f64) -> f64 {
            ((a % b) + b) % b
        }

        match (self, modulus) {
            (Numeric::Integer(a), Numeric::Integer(b)) => {
                a.checked_rem(b).map(|c| (c + b) % b).map(Numeric::Integer)
            }
            (Numeric::Integer(a), Numeric::Float(b)) => Some(Numeric::Float(modulo(a as f64, b))),
            (Numeric::Float(a), Numeric::Integer(b)) => Some(Numeric::Float(modulo(a, b as f64))),
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(modulo(a, b))),
        }
    }
}

impl Rem for Numeric {
    type Output = Option<Self>;

    fn rem(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Numeric::Integer(a), Numeric::Integer(b)) => a.checked_rem(b).map(Numeric::Integer),
            (Numeric::Integer(a), Numeric::Float(b)) => Some(Numeric::Float((a as f64) % b)),
            (Numeric::Float(a), Numeric::Integer(b)) => Some(Numeric::Float(a % (b as f64))),
            (Numeric::Float(a), Numeric::Float(b)) => Some(Numeric::Float(a % b)),
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
pub const MOST_POSITIVE_EXACT_FLOAT: i64 = 1 << 53;

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
                FpCategory::Nan | FpCategory::Infinite | FpCategory::Subnormal => {
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

    use serde_json::{from_str as from_json, to_string as to_json};

    use std::collections::hash_map::DefaultHasher;

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    /// Test mixed comparison of longs & doubles.
    fn mixed_comparison() {
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
    fn numeric_hash() {
        let nan1 = f64::NAN;
        let nan2 = f64::from_bits(f64::NAN.to_bits() | 0xDEADBEEF); // frob the payload
        assert!(nan1.is_nan() && nan2.is_nan());

        assert_eq!(hash(&Numeric::Float(nan1)), hash(&Numeric::Float(nan1)));
        assert_ne!(hash(&Numeric::Float(nan1)), hash(&Numeric::Float(nan2)));
        assert_eq!(hash(&Numeric::Float(nan2)), hash(&Numeric::Float(nan2)));

        let inf = f64::INFINITY;
        let ninf = f64::NEG_INFINITY;
        assert!(inf.is_infinite() && ninf.is_infinite());
        assert_eq!(hash(&Numeric::Float(inf)), hash(&Numeric::Float(inf)));
        assert_ne!(hash(&Numeric::Float(inf)), hash(&Numeric::Float(ninf)));
        assert_eq!(hash(&Numeric::Float(ninf)), hash(&Numeric::Float(ninf)));

        // Integral float hashing.
        assert_eq!(hash(&Numeric::Float(0.0)), hash(&Numeric::Float(0.0)));
        assert_eq!(hash(&Numeric::Float(0.0)), hash(&Numeric::Float(-0.0)));
        assert_eq!(hash(&Numeric::Float(1.0)), hash(&Numeric::Float(1.0)));
        assert_ne!(hash(&Numeric::Float(1.0)), hash(&Numeric::Float(-1.0)));
        assert_eq!(hash(&Numeric::Float(1e100)), hash(&Numeric::Float(1e100)));
        assert_ne!(hash(&Numeric::Float(1e100)), hash(&Numeric::Float(2e100)));

        // Fractional float hashing.
        let eps = f64::EPSILON;
        assert!(eps.is_normal() && eps > 0.0);
        assert_eq!(hash(&Numeric::Float(1.1)), hash(&Numeric::Float(1.1)));
        assert_ne!(hash(&Numeric::Float(1.1)), hash(&Numeric::Float(1.1 + eps)));
        assert_ne!(hash(&Numeric::Float(1.1)), hash(&Numeric::Float(1.1 - eps)));

        // Mixed hashing.
        let min = i64::MIN;
        let max = i64::MAX;
        let mid = 1_i64 << 53;
        let fmin = MOST_NEGATIVE_I64_FLOAT;
        let fmax = MOST_POSITIVE_I64_FLOAT;
        let fmid = 2_f64.powi(53);
        assert_eq!(hash(&Numeric::Integer(0)), hash(&Numeric::Float(-0.0)));
        assert_eq!(hash(&Numeric::Integer(0)), hash(&Numeric::Float(0.0)));
        assert_eq!(hash(&Numeric::Integer(1)), hash(&Numeric::Float(1.0)));
        assert_ne!(hash(&Numeric::Integer(-1)), hash(&Numeric::Float(1.0)));
        assert_eq!(hash(&Numeric::Integer(-1)), hash(&Numeric::Float(-1.0)));
        assert_eq!(hash(&Numeric::Integer(min)), hash(&Numeric::Float(fmin)));

        assert_ne!(
            hash(&Numeric::Integer(mid)),
            hash(&Numeric::Float(fmid - 1.0)) // representationally distinct
        );
        assert_eq!(
            hash(&Numeric::Integer(mid - 1)),
            hash(&Numeric::Float(fmid - 1.0))
        );
        assert_eq!(hash(&Numeric::Integer(mid)), hash(&Numeric::Float(fmid)));
        assert_ne!(hash(&Numeric::Integer(max)), hash(&Numeric::Float(fmax)));

        assert_ne!(
            hash(&Numeric::Integer(max)),
            hash(&Numeric::Float(fmax + 2048.0)) // next representationally distinct float up
        );
        assert_ne!(
            hash(&Numeric::Integer(max)),
            hash(&Numeric::Float(fmax - 1024.0)) // next representationally distinct float down
        );
        assert_ne!(
            hash(&Numeric::Integer(min)),
            hash(&Numeric::Float(fmin + 2048.0)) // next representationally distinct float up
        );
        assert_ne!(
            hash(&Numeric::Integer(min)),
            hash(&Numeric::Float(fmin - 2048.0)) // next representationally distinct float down
        );
    }

    #[test]
    fn json_serialization() {
        assert_eq!(to_json(&Numeric::Integer(0)).unwrap(), r#"{"Integer":0}"#);
        assert_eq!(to_json(&Numeric::Integer(1)).unwrap(), r#"{"Integer":1}"#);
        assert_eq!(to_json(&Numeric::Integer(-1)).unwrap(), r#"{"Integer":-1}"#);
        assert_eq!(
            to_json(&Numeric::Integer(i64::MAX)).unwrap(),
            r#"{"Integer":9223372036854775807}"#
        );

        assert_eq!(
            to_json(&Numeric::Float(MOST_POSITIVE_EXACT_FLOAT as f64)).unwrap(),
            r#"{"Float":9007199254740992.0}"#
        );
        assert_eq!(to_json(&Numeric::Float(1.0)).unwrap(), r#"{"Float":1.0}"#);
        assert_eq!(
            to_json(&Numeric::Float(f64::EPSILON)).unwrap(),
            r#"{"Float":2.220446049250313e-16}"#
        );
        assert_eq!(to_json(&Numeric::Float(0.0)).unwrap(), r#"{"Float":0.0}"#);
        assert_eq!(to_json(&Numeric::Float(-0.0)).unwrap(), r#"{"Float":-0.0}"#);
        assert_eq!(to_json(&Numeric::Float(-1.0)).unwrap(), r#"{"Float":-1.0}"#);
        assert_eq!(
            to_json(&Numeric::Float(f64::NEG_INFINITY)).unwrap(),
            r#"{"Float":"-Infinity"}"#
        );
        assert_eq!(
            to_json(&Numeric::Float(f64::INFINITY)).unwrap(),
            r#"{"Float":"Infinity"}"#
        );
        assert_eq!(
            to_json(&Numeric::Float(f64::NAN)).unwrap(),
            r#"{"Float":"NaN"}"#
        );
    }

    #[test]
    fn json_deserialization() {
        // Integers.
        assert_eq!(
            from_json::<Numeric>(r#"{"Integer":0}"#).unwrap(),
            Numeric::Integer(0)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Integer":1}"#).unwrap(),
            Numeric::Integer(1)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Integer":-1}"#).unwrap(),
            Numeric::Integer(-1)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Integer":9223372036854775807}"#).unwrap(),
            Numeric::Integer(i64::MAX)
        );

        // Floats.
        assert_eq!(
            from_json::<Numeric>(r#"{"Float":9007199254740992.0}"#).unwrap(),
            Numeric::Float(MOST_POSITIVE_EXACT_FLOAT as f64)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Float":1.0}"#).unwrap(),
            Numeric::Float(1.0)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Float":2.220446049250313e-16}"#).unwrap(),
            Numeric::Float(f64::EPSILON)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Float":0.0}"#).unwrap(),
            Numeric::Float(0.0)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Float":-0.0}"#).unwrap(),
            Numeric::Float(-0.0)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Float":-1.0}"#).unwrap(),
            Numeric::Float(-1.0)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Float":"-Infinity"}"#).unwrap(),
            Numeric::Float(f64::NEG_INFINITY)
        );
        assert_eq!(
            from_json::<Numeric>(r#"{"Float":"Infinity"}"#).unwrap(),
            Numeric::Float(f64::INFINITY)
        );
        assert!(match from_json::<Numeric>(r#"{"Float":"NaN"}"#).unwrap() {
            Numeric::Float(f) => f.is_nan(),
            _ => panic!("expected a float"),
        });
    }
}

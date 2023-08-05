use serde::{
    de::{Unexpected, Visitor},
    forward_to_deserialize_any, Deserialize, Deserializer, Serialize,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
};

use crate::ValueError;

#[derive(Clone, Copy)]
pub enum Number {
    Unsigned(u64),
    Signed(i64),
    Float(f64),
}

impl Debug for Number {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Number({})", self)
    }
}

impl Display for Number {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Unsigned(number) => Display::fmt(&number, formatter),
            Number::Signed(number) => Display::fmt(&number, formatter),
            Number::Float(number) => Display::fmt(&number, formatter),
        }
    }
}

impl Number {
    fn total_cmp(&self, other: &Self) -> Ordering {
        match (*self, *other) {
            (Number::Unsigned(a), Number::Unsigned(b)) => a.cmp(&b),
            (Number::Signed(a), Number::Signed(b)) => a.cmp(&b),
            (Number::Unsigned(a), Number::Signed(b)) => (a as i64).cmp(&b),
            (Number::Signed(a), Number::Unsigned(b)) => a.cmp(&(b as i64)),
            (Number::Float(a), Number::Float(b)) => {
                // FIXME: change to total_cmp for Rust >= 1.62.0
                a.partial_cmp(&b).unwrap_or_else(|| {
                    // arbitrarily sort the NaN last
                    if !a.is_nan() {
                        Ordering::Less
                    } else if !b.is_nan() {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
            }
            (Number::Signed(a), Number::Float(b)) => {
                // FIXME: change to total_cmp for Rust >= 1.62.0
                (a as f64).partial_cmp(&b).unwrap_or_else(|| {
                    // arbitrarily sort the NaN last
                    if !(a as f64).is_nan() {
                        Ordering::Less
                    } else if !b.is_nan() {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
            }
            (Number::Unsigned(a), Number::Float(b)) => {
                // FIXME: change to total_cmp for Rust >= 1.62.0
                (a as f64).partial_cmp(&b).unwrap_or_else(|| {
                    // arbitrarily sort the NaN last
                    if !(a as f64).is_nan() {
                        Ordering::Less
                    } else if !b.is_nan() {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
            }
            (Number::Float(a), Number::Signed(b)) => {
                // FIXME: change to total_cmp for Rust >= 1.62.0
                a.partial_cmp(&(b as f64)).unwrap_or_else(|| {
                    // arbitrarily sort the NaN last
                    if !a.is_nan() {
                        Ordering::Less
                    } else if !(b as f64).is_nan() {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
            }
            (Number::Float(a), Number::Unsigned(b)) => {
                // FIXME: change to total_cmp for Rust >= 1.62.0
                a.partial_cmp(&(b as f64)).unwrap_or_else(|| {
                    // arbitrarily sort the NaN last
                    if !a.is_nan() {
                        Ordering::Less
                    } else if !(b as f64).is_nan() {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
            }
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Number::Unsigned(a), Number::Unsigned(b)) => a == b,
            (Number::Signed(a), Number::Signed(b)) => a == b,
            (Number::Float(a), Number::Float(b)) => a == b,
            (Number::Unsigned(a), Number::Signed(b)) => (a as i64) == b,
            (Number::Signed(a), Number::Unsigned(b)) => a == (b as i64),
            (Number::Unsigned(a), Number::Float(b)) => (a as f64) == b,
            (Number::Signed(a), Number::Float(b)) => (a as f64) == b,
            (Number::Float(a), Number::Unsigned(b)) => a == (b as f64),
            (Number::Float(a), Number::Signed(b)) => a == (b as f64),
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.total_cmp(other))
    }
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        {
            match self {
                Number::Unsigned(u) => serializer.serialize_u64(*u),
                Number::Signed(s) => serializer.serialize_i64(*s),
                Number::Float(f) => serializer.serialize_f64(*f),
            }
        }
    }
}

impl<'de> Deserialize<'de> for Number {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NumberVisitor;

        impl<'de> Visitor<'de> for NumberVisitor {
            type Value = Number;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a number")
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Number, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Number, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Number, E> {
                Ok(value.into())
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

impl<'de> Deserializer<'de> for Number {
    type Error = ValueError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Number::Unsigned(u) => visitor.visit_u64(u),
            Number::Signed(s) => visitor.visit_i64(s),
            Number::Float(f) => visitor.visit_f64(f),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a> Deserializer<'de> for &'a Number {
    type Error = ValueError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Number::Unsigned(u) => visitor.visit_u64(*u),
            Number::Signed(s) => visitor.visit_i64(*s),
            Number::Float(f) => visitor.visit_f64(*f),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

macro_rules! from_signed {
    ($($signed_ty:ident)*) => {
        $(
            impl From<$signed_ty> for Number {
                #[inline]
                #[allow(clippy::cast_sign_loss)]
                fn from(s: $signed_ty) -> Self {
                    Number::Signed(s as i64)
                }
            }
        )*
    };
}

macro_rules! from_unsigned {
    ($($unsigned_ty:ident)*) => {
        $(
            impl From<$unsigned_ty> for Number {
                #[inline]
                fn from(u: $unsigned_ty) -> Self {
                    Number::Unsigned(u as u64)
                }
            }
        )*
    };
}

macro_rules! from_float {
    ($($float_ty:ident)*) => {
        $(
            impl From<$float_ty> for Number {
                #[inline]
                fn from(f: $float_ty) -> Self {
                    Number::Float(f as f64)
                }
            }
        )*
    }
}

from_signed!(i8 i16 i32 i64 isize);
from_unsigned!(u8 u16 u32 u64 usize);
from_float!(f32 f64);

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Number::Unsigned(u) => u.hash(state),
            Number::Signed(s) => s.hash(state),
            Number::Float(_f) => unimplemented!(),
        }
    }
}

pub(crate) fn unexpected(number: &Number) -> Unexpected {
    match number {
        Number::Unsigned(u) => Unexpected::Unsigned(*u),
        Number::Signed(s) => Unexpected::Signed(*s),
        Number::Float(f) => Unexpected::Float(*f),
    }
}

#[cfg(test)]
mod test {
    use crate::{Number, Value};
    use pretty_assertions::assert_eq;

    #[test]
    fn number_compare_test() {
        // unsigned
        assert_eq!(
            true,
            Value::Number(Number::Unsigned(2)) == Value::Number(Number::Unsigned(2))
        );

        assert_eq!(
            true,
            Value::Number(Number::Unsigned(3)) > Value::Number(Number::Unsigned(2))
        );

        assert_eq!(
            true,
            Value::Number(Number::Unsigned(2)) < Value::Number(Number::Unsigned(3))
        );

        // signed
        assert_eq!(
            true,
            Value::Number(Number::Signed(2)) == Value::Number(Number::Signed(2))
        );

        assert_eq!(
            true,
            Value::Number(Number::Signed(3)) > Value::Number(Number::Signed(2))
        );

        assert_eq!(
            true,
            Value::Number(Number::Signed(2)) < Value::Number(Number::Signed(3))
        );

        // float
        assert_eq!(
            true,
            Value::Number(Number::Float(2.0)) == Value::Number(Number::Float(2.0))
        );

        assert_eq!(
            true,
            Value::Number(Number::Float(3.0)) > Value::Number(Number::Float(2.0))
        );

        assert_eq!(
            true,
            Value::Number(Number::Float(2.0)) < Value::Number(Number::Float(3.0))
        );

        // unsigned with signed
        assert_eq!(
            true,
            Value::Number(Number::Unsigned(2)) == Value::Number(Number::Signed(2))
        );

        assert_eq!(
            true,
            Value::Number(Number::Unsigned(3)) > Value::Number(Number::Signed(2))
        );

        assert_eq!(
            true,
            Value::Number(Number::Unsigned(2)) < Value::Number(Number::Signed(3))
        );

        // signed with unsigned
        assert_eq!(
            true,
            Value::Number(Number::Signed(2)) == Value::Number(Number::Unsigned(2))
        );

        assert_eq!(
            true,
            Value::Number(Number::Signed(3)) > Value::Number(Number::Unsigned(2))
        );

        assert_eq!(
            true,
            Value::Number(Number::Signed(2)) < Value::Number(Number::Unsigned(3))
        );

        // signed with float
        assert_eq!(
            true,
            Value::Number(Number::Signed(2)) == Value::Number(Number::Float(2.0))
        );

        assert_eq!(
            true,
            Value::Number(Number::Signed(3)) > Value::Number(Number::Float(2.0))
        );

        assert_eq!(
            true,
            Value::Number(Number::Signed(2)) < Value::Number(Number::Float(3.0))
        );

        // unsigned with float
        assert_eq!(
            true,
            Value::Number(Number::Unsigned(2)) == Value::Number(Number::Float(2.0))
        );

        assert_eq!(
            true,
            Value::Number(Number::Unsigned(3)) > Value::Number(Number::Float(2.0))
        );

        assert_eq!(
            true,
            Value::Number(Number::Unsigned(2)) < Value::Number(Number::Float(3.0))
        );
    }
}

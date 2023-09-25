use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    serde::float::{deserialize, serialize},
    Decimal,
};
use serde::{
    de::{Unexpected, Visitor},
    forward_to_deserialize_any, Deserialize, Deserializer, Serialize,
};
use std::ops::{Add, Deref, Div, Mul, Neg, Rem, Sub};
use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

use crate::SerdeValueError;

#[derive(Clone, Copy)]
pub struct Number(Decimal);

impl Debug for Number {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, formatter)
    }
}

impl Display for Number {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

impl From<Decimal> for Number {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl Deref for Number {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Number {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize(deserializer).map(Number)
    }
}

impl<'de> Deserializer<'de> for Number {
    type Error = SerdeValueError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let Some(f) = self.0.to_f64() else {
            return Err(SerdeValueError::NumberOutOfRange);
        };
        visitor.visit_f64(f)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a> Deserializer<'de> for &'a Number {
    type Error = SerdeValueError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let Some(f) = self.0.to_f64() else {
            return Err(SerdeValueError::NumberOutOfRange);
        };
        visitor.visit_f64(f)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                #[allow(clippy::cast_sign_loss)]
                fn from(n: $ty) -> Self {
                    Number(n.into())
                }
            }
        )*
    };
}

macro_rules! from_float {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                #[allow(clippy::cast_sign_loss)]
                fn from(n: $ty) -> Self {
                    // TODO what to do
                    Number(Decimal::from_f64(n as f64).unwrap())
                }
            }
        )*
    };
}

from_integer!(i8 i16 i32 i64 isize);
from_integer!(u8 u16 u32 u64 usize);
from_float!(f32 f64);

pub(crate) fn unexpected(_number: &Number) -> Unexpected {
    Unexpected::Other("number")
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Number {
    fn assert_receiver_is_total_eq(&self) {
        self.0.assert_receiver_is_total_eq()
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Decimal::hash(&self.0, state)
    }
}

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Self::Output {
        self.0.neg().into()
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, rhs: Self) -> Self::Output {
        self.0.add(rhs.0).into()
    }
}

impl Sub for Number {
    type Output = Number;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.sub(rhs.0).into()
    }
}

impl Mul for Number {
    type Output = Number;

    fn mul(self, rhs: Self) -> Self::Output {
        self.0.mul(rhs.0).into()
    }
}

impl Div for Number {
    type Output = Number;

    fn div(self, rhs: Self) -> Self::Output {
        self.0.div(rhs.0).into()
    }
}

impl Rem for Number {
    type Output = Number;

    fn rem(self, rhs: Self) -> Self::Output {
        self.0.rem(rhs.0).into()
    }
}

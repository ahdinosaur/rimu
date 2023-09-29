// https://github.com/serde-rs/json/blob/master/src/value/ser.rs

use indexmap::IndexMap;
use serde::{ser::Impossible, Serialize};
use std::fmt::Display;

use super::{to_serde_value, SerdeValue, SerdeValueError, SerdeValueObject};

impl Serialize for SerdeValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SerdeValue::Null => serializer.serialize_unit(),
            SerdeValue::Boolean(boolean) => serializer.serialize_bool(*boolean),
            SerdeValue::Number(number) => number.serialize(serializer),
            SerdeValue::String(string) => serializer.serialize_str(string),
            SerdeValue::List(list) => list.serialize(serializer),
            SerdeValue::Object(object) => object.serialize(serializer),
            // TODO
            SerdeValue::Function(_) => serializer.serialize_unit(),
        }
    }
}

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = SerdeValue;
    type Error = SerdeValueError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<SerdeValue, SerdeValueError> {
        Ok(SerdeValue::Boolean(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<SerdeValue, SerdeValueError> {
        Ok(SerdeValue::Number(value.into()))
    }

    fn serialize_i128(self, value: i128) -> Result<SerdeValue, SerdeValueError> {
        if let Ok(value) = u64::try_from(value) {
            Ok(SerdeValue::Number(value.into()))
        } else if let Ok(value) = i64::try_from(value) {
            Ok(SerdeValue::Number(value.into()))
        } else {
            Err(SerdeValueError::NumberOutOfRange)
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<SerdeValue, SerdeValueError> {
        Ok(SerdeValue::Number(value.into()))
    }

    fn serialize_u128(self, value: u128) -> Result<SerdeValue, SerdeValueError> {
        if let Ok(value) = u64::try_from(value) {
            Ok(SerdeValue::Number(value.into()))
        } else {
            Err(SerdeValueError::NumberOutOfRange)
        }
    }

    #[inline]
    fn serialize_f32(self, float: f32) -> Result<SerdeValue, SerdeValueError> {
        Ok(SerdeValue::from(float))
    }

    #[inline]
    fn serialize_f64(self, float: f64) -> Result<SerdeValue, SerdeValueError> {
        Ok(SerdeValue::from(float))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<SerdeValue, SerdeValueError> {
        let mut s = String::new();
        s.push(value);
        Ok(SerdeValue::String(s))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<SerdeValue, SerdeValueError> {
        Ok(SerdeValue::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<SerdeValue, SerdeValueError> {
        let vec = value
            .iter()
            .map(|&b| SerdeValue::Number(b.into()))
            .collect();
        Ok(SerdeValue::List(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<SerdeValue, SerdeValueError> {
        Ok(SerdeValue::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<SerdeValue, SerdeValueError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<SerdeValue, SerdeValueError>
    where
        T: ?Sized + Serialize,
    {
        let mut values = SerdeValueObject::new();
        values.insert(String::from(variant), to_serde_value(value)?);
        Ok(SerdeValue::Object(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<SerdeValue, SerdeValueError> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<SerdeValue, SerdeValueError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, SerdeValueError> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, SerdeValueError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerdeValueError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerdeValueError> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, SerdeValueError> {
        Ok(SerializeMap::Map {
            map: SerdeValueObject::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, SerdeValueError> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerdeValueError> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: SerdeValueObject::new(),
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<SerdeValue, SerdeValueError>
    where
        T: ?Sized + Display,
    {
        Ok(SerdeValue::String(value.to_string()))
    }
}

pub struct SerializeVec {
    vec: Vec<SerdeValue>,
}

pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<SerdeValue>,
}

pub enum SerializeMap {
    Map {
        map: IndexMap<String, SerdeValue>,
        next_key: Option<String>,
    },
}

pub struct SerializeStructVariant {
    name: String,
    map: IndexMap<String, SerdeValue>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = SerdeValue;
    type Error = SerdeValueError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(to_serde_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<SerdeValue, SerdeValueError> {
        Ok(SerdeValue::List(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = SerdeValue;
    type Error = SerdeValueError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<SerdeValue, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = SerdeValue;
    type Error = SerdeValueError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = SerdeValue;
    type Error = SerdeValueError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(to_serde_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut object = SerdeValueObject::new();

        object.insert(self.name, SerdeValue::List(self.vec));

        Ok(SerdeValue::Object(object))
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = SerdeValue;
    type Error = SerdeValueError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SerializeMap::Map { next_key, .. } => {
                *next_key = Some(key.serialize(MapKeySerializer)?);
                Ok(())
            }
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SerializeMap::Map { map, next_key } => {
                let key = next_key.take();
                // Panic because this indicates a bug in the program rather than an
                // expected failure.
                let key = key.expect("serialize_value called before serialize_key");
                map.insert(key, to_serde_value(value)?);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            SerializeMap::Map { map, .. } => Ok(SerdeValue::Object(map)),
        }
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> SerdeValueError {
    SerdeValueError::KeyMustBeAString
}

fn float_key_must_be_finite() -> SerdeValueError {
    SerdeValueError::FloatKeyMustBeFinite
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = SerdeValueError;

    type SerializeSeq = Impossible<String, SerdeValueError>;
    type SerializeTuple = Impossible<String, SerdeValueError>;
    type SerializeTupleStruct = Impossible<String, SerdeValueError>;
    type SerializeTupleVariant = Impossible<String, SerdeValueError>;
    type SerializeMap = Impossible<String, SerdeValueError>;
    type SerializeStruct = Impossible<String, SerdeValueError>;
    type SerializeStructVariant = Impossible<String, SerdeValueError>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String, Self::Error> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<String, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<String, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, value: i8) -> Result<String, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<String, SerdeValueError> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String, SerdeValueError> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String, SerdeValueError> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String, SerdeValueError> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String, SerdeValueError> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String, SerdeValueError> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String, SerdeValueError> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<String, SerdeValueError> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(float_key_must_be_finite())
        }
    }

    fn serialize_f64(self, value: f64) -> Result<String, SerdeValueError> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(float_key_must_be_finite())
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String, SerdeValueError> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String, SerdeValueError> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String, SerdeValueError> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String, SerdeValueError> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, SerdeValueError> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, SerdeValueError>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String, SerdeValueError> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String, SerdeValueError>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerdeValueError> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T>(self, value: &T) -> Result<String, SerdeValueError>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = SerdeValue;
    type Error = SerdeValueError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::serialize_entry(self, key, value),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::end(self),
        }
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = SerdeValue;
    type Error = SerdeValueError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), SerdeValueError>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(String::from(key), to_serde_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, SerdeValueError> {
        let mut object = SerdeValueObject::new();

        object.insert(self.name, SerdeValue::Object(self.map));

        Ok(SerdeValue::Object(object))
    }
}

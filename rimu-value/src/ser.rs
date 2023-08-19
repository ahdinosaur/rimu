// https://github.com/serde-rs/json/blob/master/src/value/ser.rs

use serde::{ser::Impossible, Serialize};
use std::{collections::BTreeMap, fmt::Display};

use crate::{to_value, Object, Value, ValueError};

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Boolean(boolean) => serializer.serialize_bool(*boolean),
            Value::Number(number) => number.serialize(serializer),
            Value::String(string) => serializer.serialize_str(string),
            Value::List(list) => list.serialize(serializer),
            Value::Object(object) => object.serialize(serializer),
            Value::Function(_) => todo!(),
        }
    }
}

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = ValueError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value, ValueError> {
        Ok(Value::Boolean(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value, ValueError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value, ValueError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value, ValueError> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Value, ValueError> {
        Ok(Value::Number(value.into()))
    }

    fn serialize_i128(self, value: i128) -> Result<Value, ValueError> {
        if let Ok(value) = u64::try_from(value) {
            Ok(Value::Number(value.into()))
        } else if let Ok(value) = i64::try_from(value) {
            Ok(Value::Number(value.into()))
        } else {
            Err(ValueError::NumberOutOfRange)
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value, ValueError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value, ValueError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value, ValueError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value, ValueError> {
        Ok(Value::Number(value.into()))
    }

    fn serialize_u128(self, value: u128) -> Result<Value, ValueError> {
        if let Ok(value) = u64::try_from(value) {
            Ok(Value::Number(value.into()))
        } else {
            Err(ValueError::NumberOutOfRange)
        }
    }

    #[inline]
    fn serialize_f32(self, float: f32) -> Result<Value, ValueError> {
        Ok(Value::from(float))
    }

    #[inline]
    fn serialize_f64(self, float: f64) -> Result<Value, ValueError> {
        Ok(Value::from(float))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value, ValueError> {
        let mut s = String::new();
        s.push(value);
        Ok(Value::String(s))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value, ValueError> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value, ValueError> {
        let vec = value.iter().map(|&b| Value::Number(b.into())).collect();
        Ok(Value::List(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value, ValueError> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, ValueError> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, ValueError> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, ValueError>
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
    ) -> Result<Value, ValueError>
    where
        T: ?Sized + Serialize,
    {
        let mut values = Object::new();
        values.insert(String::from(variant), to_value(value)?);
        Ok(Value::Object(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value, ValueError> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Value, ValueError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, ValueError> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, ValueError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, ValueError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, ValueError> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, ValueError> {
        Ok(SerializeMap::Map {
            map: Object::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, ValueError> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, ValueError> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: Object::new(),
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Value, ValueError>
    where
        T: ?Sized + Display,
    {
        Ok(Value::String(value.to_string()))
    }
}

pub struct SerializeVec {
    vec: Vec<Value>,
}

pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value>,
}

pub enum SerializeMap {
    Map {
        map: BTreeMap<String, Value>,
        next_key: Option<String>,
    },
}

pub struct SerializeStructVariant {
    name: String,
    map: BTreeMap<String, Value>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        Ok(Value::List(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Self::Error> {
        let mut object = Object::new();

        object.insert(self.name, Value::List(self.vec));

        Ok(Value::Object(object))
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = ValueError;

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
                map.insert(key, to_value(value)?);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Value, Self::Error> {
        match self {
            SerializeMap::Map { map, .. } => Ok(Value::Object(map)),
        }
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> ValueError {
    ValueError::KeyMustBeAString
}

fn float_key_must_be_finite() -> ValueError {
    ValueError::FloatKeyMustBeFinite
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = ValueError;

    type SerializeSeq = Impossible<String, ValueError>;
    type SerializeTuple = Impossible<String, ValueError>;
    type SerializeTupleStruct = Impossible<String, ValueError>;
    type SerializeTupleVariant = Impossible<String, ValueError>;
    type SerializeMap = Impossible<String, ValueError>;
    type SerializeStruct = Impossible<String, ValueError>;
    type SerializeStructVariant = Impossible<String, ValueError>;

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

    fn serialize_i16(self, value: i16) -> Result<String, ValueError> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String, ValueError> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String, ValueError> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String, ValueError> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String, ValueError> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String, ValueError> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String, ValueError> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<String, ValueError> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(float_key_must_be_finite())
        }
    }

    fn serialize_f64(self, value: f64) -> Result<String, ValueError> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(float_key_must_be_finite())
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String, ValueError> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String, ValueError> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String, ValueError> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String, ValueError> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, ValueError> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, ValueError>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String, ValueError> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String, ValueError>
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
    ) -> Result<Self::SerializeStructVariant, ValueError> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T>(self, value: &T) -> Result<String, ValueError>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::serialize_entry(self, key, value),
        }
    }

    fn end(self) -> Result<Value, Self::Error> {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::end(self),
        }
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(String::from(key), to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        let mut object = Object::new();

        object.insert(self.name, Value::Object(self.map));

        Ok(Value::Object(object))
    }
}

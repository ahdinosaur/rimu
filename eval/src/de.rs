use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, EnumAccess, Error as SError, Expected,
    IntoDeserializer, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use std::{fmt, vec};
use indexmap::IndexMap;

use crate::{number, List, Object, Value, ValueError};

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any value")
            }

            fn visit_bool<E>(self, b: bool) -> Result<Value, E>
            where
                E: SError,
            {
                Ok(Value::Boolean(b))
            }

            fn visit_i64<E>(self, i: i64) -> Result<Value, E>
            where
                E: SError,
            {
                Ok(Value::Number(i.into()))
            }

            fn visit_u64<E>(self, u: u64) -> Result<Value, E>
            where
                E: SError,
            {
                Ok(Value::Number(u.into()))
            }

            fn visit_f64<E>(self, f: f64) -> Result<Value, E>
            where
                E: SError,
            {
                Ok(Value::Number(f.into()))
            }

            fn visit_str<E>(self, s: &str) -> Result<Value, E>
            where
                E: SError,
            {
                Ok(Value::String(s.to_owned()))
            }

            fn visit_string<E>(self, s: String) -> Result<Value, E>
            where
                E: SError,
            {
                Ok(Value::String(s))
            }

            fn visit_unit<E>(self) -> Result<Value, E>
            where
                E: SError,
            {
                Ok(Value::Null)
            }

            fn visit_none<E>(self) -> Result<Value, E>
            where
                E: SError,
            {
                Ok(Value::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(element) = visitor.next_element()? {
                    vec.push(element);
                }

                Ok(Value::List(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut values = IndexMap::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Value::Object(values))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Value {
    fn deserialize_number<'de, V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(n) => n.deserialize_any(visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }
}

fn visit_list<'de, V>(list: List, visitor: V) -> Result<V::Value, ValueError>
where
    V: Visitor<'de>,
{
    let len = list.len();
    let mut deserializer = SeqDeserializer::new(list);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(ValueError::invalid_length(len, &"fewer elements in list"))
    }
}

fn visit_object<'de, V>(object: Object, visitor: V) -> Result<V::Value, ValueError>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapDeserializer::new(object);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(ValueError::invalid_length(len, &"fewer elements in map"))
    }
}

impl<'de> IntoDeserializer<'de, ValueError> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = ValueError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Boolean(v) => visitor.visit_bool(v),
            Value::Number(n) => n.deserialize_any(visitor),
            Value::String(v) => visitor.visit_string(v),
            Value::List(v) => visit_list(v, visitor),
            Value::Object(v) => visit_object(v, visitor),
            Value::Function(_f) => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Boolean(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(v) => visitor.visit_string(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(v) => visitor.visit_string(v),
            Value::List(v) => visit_list(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::List(v) => visit_list(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Object(v) => visit_object(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::List(v) => visit_list(v, visitor),
            Value::Object(v) => visit_object(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            Value::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(ValueError::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(ValueError::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (Value::String(variant), Some(value))
            }
            Value::String(variant) => (Value::String(variant), None),
            other => {
                return Err(ValueError::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

struct EnumDeserializer {
    variant: Value,
    value: Option<Value>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = ValueError;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), ValueError>
    where
        V: DeserializeSeed<'de>,
    {
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(self.variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = ValueError;

    fn unit_variant(self) -> Result<(), ValueError> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, ValueError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(ValueError::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::List(v)) => Deserializer::deserialize_any(SeqDeserializer::new(v), visitor),
            Some(other) => Err(ValueError::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(ValueError::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Object(v)) => {
                Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
            }
            Some(other) => Err(ValueError::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(ValueError::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> Deserializer<'de> for SeqDeserializer {
    type Error = ValueError;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = visitor.visit_seq(&mut self)?;
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(ValueError::invalid_length(len, &"fewer elements in list"))
            }
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = ValueError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, ValueError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer {
    iter: <Object as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: Object) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = ValueError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, ValueError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, ValueError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => panic!("visit_value called before visit_key"),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> Deserializer<'de> for MapDeserializer {
    type Error = ValueError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier
    }
}

impl Value {
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: de::Error,
    {
        de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected {
        match self {
            Value::Null => Unexpected::Unit,
            Value::Boolean(b) => Unexpected::Bool(*b),
            Value::Number(n) => number::unexpected(n),
            Value::String(s) => Unexpected::Str(s),
            Value::List(_) => Unexpected::Seq,
            Value::Object(_) => Unexpected::Map,
            Value::Function(_f) => todo!(),
        }
    }
}

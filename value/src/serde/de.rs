use indexmap::IndexMap;
use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, EnumAccess, Error as SError, Expected,
    IntoDeserializer, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use std::{fmt, vec};

use crate::{number, SerdeValue, SerdeValueError, SerdeValueList, SerdeValueObject};

impl<'de> Deserialize<'de> for SerdeValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = SerdeValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any value")
            }

            fn visit_bool<E>(self, b: bool) -> Result<Self::Value, E>
            where
                E: SError,
            {
                Ok(Self::Value::Boolean(b))
            }

            fn visit_i64<E>(self, i: i64) -> Result<Self::Value, E>
            where
                E: SError,
            {
                Ok(Self::Value::Number(i.into()))
            }

            fn visit_u64<E>(self, u: u64) -> Result<Self::Value, E>
            where
                E: SError,
            {
                Ok(Self::Value::Number(u.into()))
            }

            fn visit_f64<E>(self, f: f64) -> Result<Self::Value, E>
            where
                E: SError,
            {
                Ok(Self::Value::Number(f.into()))
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: SError,
            {
                Ok(Self::Value::String(s.to_owned()))
            }

            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: SError,
            {
                Ok(Self::Value::String(s))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: SError,
            {
                Ok(Self::Value::Null)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: SError,
            {
                Ok(Self::Value::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(element) = visitor.next_element()? {
                    vec.push(element);
                }

                Ok(Self::Value::List(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut values = IndexMap::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Self::Value::Object(values))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl SerdeValue {
    fn deserialize_number<'de, V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::Number(n) => n.deserialize_any(visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }
}

fn visit_list<'de, V>(list: SerdeValueList, visitor: V) -> Result<V::Value, SerdeValueError>
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
        Err(SerdeValueError::invalid_length(
            len,
            &"fewer elements in list",
        ))
    }
}

fn visit_object<'de, V>(object: SerdeValueObject, visitor: V) -> Result<V::Value, SerdeValueError>
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
        Err(SerdeValueError::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

impl<'de> IntoDeserializer<'de, SerdeValueError> for SerdeValue {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> Deserializer<'de> for SerdeValue {
    type Error = SerdeValueError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::Null => visitor.visit_unit(),
            SerdeValue::Boolean(v) => visitor.visit_bool(v),
            SerdeValue::Number(n) => n.deserialize_any(visitor),
            SerdeValue::String(v) => visitor.visit_string(v),
            SerdeValue::List(v) => visit_list(v, visitor),
            SerdeValue::Object(v) => visit_object(v, visitor),
            SerdeValue::Function(_f) => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::Boolean(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_number(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::String(v) => visitor.visit_string(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::String(v) => visitor.visit_string(v),
            SerdeValue::List(v) => visit_list(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::Null => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::List(v) => visit_list(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, SerdeValueError>
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
    ) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::Object(v) => visit_object(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::List(v) => visit_list(v, visitor),
            SerdeValue::Object(v) => visit_object(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            SerdeValue::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(SerdeValueError::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(SerdeValueError::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (SerdeValue::String(variant), Some(value))
            }
            SerdeValue::String(variant) => (SerdeValue::String(variant), None),
            other => {
                return Err(SerdeValueError::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

struct EnumDeserializer {
    variant: SerdeValue,
    value: Option<SerdeValue>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = SerdeValueError;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), SerdeValueError>
    where
        V: DeserializeSeed<'de>,
    {
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(self.variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<SerdeValue>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = SerdeValueError;

    fn unit_variant(self) -> Result<(), SerdeValueError> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, SerdeValueError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(SerdeValueError::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(SerdeValue::List(v)) => {
                Deserializer::deserialize_any(SeqDeserializer::new(v), visitor)
            }
            Some(other) => Err(SerdeValueError::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(SerdeValueError::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(SerdeValue::Object(v)) => {
                Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
            }
            Some(other) => Err(SerdeValueError::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(SerdeValueError::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<SerdeValue>,
}

impl SeqDeserializer {
    fn new(vec: Vec<SerdeValue>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> Deserializer<'de> for SeqDeserializer {
    type Error = SerdeValueError;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, SerdeValueError>
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
                Err(SerdeValueError::invalid_length(
                    len,
                    &"fewer elements in list",
                ))
            }
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
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
    type Error = SerdeValueError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, SerdeValueError>
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
    iter: <SerdeValueObject as IntoIterator>::IntoIter,
    value: Option<SerdeValue>,
}

impl MapDeserializer {
    fn new(map: SerdeValueObject) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = SerdeValueError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, SerdeValueError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(SerdeValue::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, SerdeValueError>
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
    type Error = SerdeValueError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, SerdeValueError>
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

impl SerdeValue {
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
            SerdeValue::Null => Unexpected::Unit,
            SerdeValue::Boolean(b) => Unexpected::Bool(*b),
            SerdeValue::Number(n) => number::unexpected(n),
            SerdeValue::String(s) => Unexpected::Str(s),
            SerdeValue::List(_) => Unexpected::Seq,
            SerdeValue::Object(_) => Unexpected::Map,
            SerdeValue::Function(_f) => todo!(),
        }
    }
}

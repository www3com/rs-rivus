use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{
    Serialize, Serializer,
    ser::{
        self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    F64(f64),
    Str(String),
    Bytes(Vec<u8>),
    Date(NaiveDate),
    Time(NaiveTime),
    DateTime(NaiveDateTime),
    DateTimeUtc(DateTime<Utc>),
    Decimal(Decimal),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

#[derive(Debug)]
pub enum SqlParam {
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Bool(bool),
    Date(NaiveDate),
    Time(NaiveTime),
    DateTime(NaiveDateTime),
    DateTimeUtc(DateTime<Utc>),
    Decimal(Decimal),
    Null,
}

pub fn value_to_param(v: &Value) -> SqlParam {
    match v {
        Value::I16(v) => SqlParam::I16(*v),
        Value::I32(v) => SqlParam::I32(*v),
        Value::I64(v) => SqlParam::I64(*v),
        Value::U8(v) => SqlParam::U8(*v),
        Value::F64(v) => SqlParam::F64(*v),
        Value::Str(v) => SqlParam::String(v.clone()),
        Value::Bytes(v) => SqlParam::Bytes(v.clone()),
        Value::Bool(v) => SqlParam::Bool(*v),
        Value::Date(v) => SqlParam::Date(*v),
        Value::Time(v) => SqlParam::Time(*v),
        Value::DateTime(v) => SqlParam::DateTime(*v),
        Value::DateTimeUtc(v) => SqlParam::DateTimeUtc(*v),
        Value::Decimal(v) => SqlParam::Decimal(*v),
        Value::Null => SqlParam::Null,
        _ => unreachable!("Cannot convert complex value (List/Map) to SqlParam"),
    }
}

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

// Serializer
pub struct ValueSerializer;

impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = ListSerializer;
    type SerializeTuple = ListSerializer;
    type SerializeTupleStruct = ListSerializer;
    type SerializeTupleVariant = ListSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = MapSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I16(v as i16))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(v as i64))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(v as i64))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(v as i64))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::F64(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Str(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Str(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Str(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ListSerializer {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer {
            map: HashMap::with_capacity(len.unwrap_or(0)),
            key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(MapSerializer {
            map: HashMap::with_capacity(len),
            key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(MapSerializer {
            map: HashMap::with_capacity(len),
            key: None,
        })
    }
}

pub struct ListSerializer {
    vec: Vec<Value>,
}

macro_rules! impl_serialize_seq {
    ($trait:ident, $method:ident) => {
        impl $trait for ListSerializer {
            type Ok = Value;
            type Error = Error;

            fn $method<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
                self.vec.push(value.serialize(ValueSerializer)?);
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Ok(Value::List(self.vec))
            }
        }
    };
}

impl_serialize_seq!(SerializeSeq, serialize_element);
impl_serialize_seq!(SerializeTuple, serialize_element);
impl_serialize_seq!(SerializeTupleStruct, serialize_field);
impl_serialize_seq!(SerializeTupleVariant, serialize_field);

pub struct MapSerializer {
    map: HashMap<String, Value>,
    key: Option<String>,
}

impl SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let k = key.serialize(ValueSerializer)?;
        match k {
            Value::Str(s) => {
                self.key = Some(s);
                Ok(())
            }
            _ => Err(ser::Error::custom("Map key must be a string")),
        }
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let v = value.serialize(ValueSerializer)?;
        let key = self
            .key
            .take()
            .ok_or_else(|| ser::Error::custom("Missing key for value"))?;
        self.map.insert(key, v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Map(self.map))
    }
}

macro_rules! impl_serialize_struct {
    ($trait:ident) => {
        impl $trait for MapSerializer {
            type Ok = Value;
            type Error = Error;

            fn serialize_field<T: ?Sized + Serialize>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error> {
                let v = value.serialize(ValueSerializer)?;
                self.map.insert(key.to_string(), v);
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Ok(Value::Map(self.map))
            }
        }
    };
}

impl_serialize_struct!(SerializeStruct);
impl_serialize_struct!(SerializeStructVariant);

pub fn to_value<T: Serialize>(t: &T) -> Value {
    t.serialize(ValueSerializer).unwrap()
}

macro_rules! impl_from {
    ($type:ty, $variant:ident) => {
        impl From<$type> for Value {
            fn from(v: $type) -> Self {
                Value::$variant(v)
            }
        }
    };
}

impl_from!(i64, I64);
impl_from!(i32, I32);
impl_from!(i16, I16);
impl_from!(u8, U8);
impl_from!(f64, F64);
impl_from!(String, Str);
impl_from!(bool, Bool);
impl_from!(Vec<u8>, Bytes);
impl_from!(NaiveDate, Date);
impl_from!(NaiveTime, Time);
impl_from!(NaiveDateTime, DateTime);
impl_from!(DateTime<Utc>, DateTimeUtc);
impl_from!(Decimal, Decimal);

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::I64(v as i64)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Str(v.to_string())
    }
}

use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde::forward_to_deserialize_any;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::marker::PhantomData;
use sqlx::mysql::MySqlRow;
use sqlx::postgres::PgRow;
use sqlx::sqlite::SqliteRow;

/// Trait to abstract row access for different database drivers
pub trait RowReader {
    fn column_count(&self) -> usize;
    fn column_name(&self, idx: usize) -> &str;
    fn is_null(&self, idx: usize) -> bool;
    fn type_name(&self, idx: usize) -> &str;
    
    fn get_bool(&self, idx: usize) -> Result<bool, String>;
    fn get_i64(&self, idx: usize) -> Result<i64, String>;
    fn get_f64(&self, idx: usize) -> Result<f64, String>;
    fn get_string(&self, idx: usize) -> Result<String, String>;
    fn get_json(&self, idx: usize) -> Result<serde_json::Value, String>;
}

macro_rules! impl_row_reader {
    ($row_type:ty) => {
        impl RowReader for $row_type {
            fn column_count(&self) -> usize {
                self.columns().len()
            }
            fn column_name(&self, idx: usize) -> &str {
                self.column(idx).name()
            }
            fn is_null(&self, idx: usize) -> bool {
                self.try_get_raw(idx).map(|r| r.is_null()).unwrap_or(true)
            }
            fn type_name(&self, idx: usize) -> &str {
                self.column(idx).type_info().name()
            }
            fn get_bool(&self, idx: usize) -> Result<bool, String> {
                self.try_get::<bool, _>(idx).map_err(|e| e.to_string())
            }
            fn get_i64(&self, idx: usize) -> Result<i64, String> {
                self.try_get::<i64, _>(idx).map_err(|e| e.to_string())
            }
            fn get_f64(&self, idx: usize) -> Result<f64, String> {
                self.try_get::<f64, _>(idx).map_err(|e| e.to_string())
            }
            fn get_string(&self, idx: usize) -> Result<String, String> {
                 // Try generic string first
                 if let Ok(v) = self.try_get::<String, _>(idx) {
                     return Ok(v);
                 }
                 // Try NaiveDateTime
                 if let Ok(v) = self.try_get::<sqlx::types::chrono::NaiveDateTime, _>(idx) {
                     return Ok(v.to_string());
                 }
                 // Try DateTime<Utc>
                 if let Ok(v) = self.try_get::<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>, _>(idx) {
                     return Ok(v.to_rfc3339());
                 }
                 Err(format!("Cannot convert column {} to string", self.column(idx).name()))
            }
            fn get_json(&self, idx: usize) -> Result<serde_json::Value, String> {
                self.try_get::<serde_json::Value, _>(idx).map_err(|e| e.to_string())
            }
        }
    };
}

impl_row_reader!(MySqlRow);
impl_row_reader!(PgRow);
impl_row_reader!(SqliteRow);

pub struct RowDeserializer<'a, R: RowReader> {
    row: &'a R,
    col_idx: usize,
    count: usize,
    _marker: PhantomData<R>,
}

impl<'a, R: RowReader> RowDeserializer<'a, R> {
    pub fn new(row: &'a R) -> Self {
        Self {
            row,
            col_idx: 0,
            count: row.column_count(),
            _marker: PhantomData,
        }
    }
}

impl<'de, 'a, R: RowReader> de::Deserializer<'de> for RowDeserializer<'a, R> {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a, R: RowReader> MapAccess<'de> for RowDeserializer<'a, R> {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.col_idx >= self.count {
            return Ok(None);
        }
        let col_name = self.row.column_name(self.col_idx);
        let name_de = de::IntoDeserializer::into_deserializer(col_name);
        seed.deserialize(name_de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value_de = ColValueDeserializer {
            row: self.row,
            col_idx: self.col_idx,
        };
        self.col_idx += 1;
        seed.deserialize(value_de)
    }
}

struct ColValueDeserializer<'a, R: RowReader> {
    row: &'a R,
    col_idx: usize,
}

impl<'de, 'a, R: RowReader> de::Deserializer<'de> for ColValueDeserializer<'a, R> {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let type_name = self.row.type_name(self.col_idx);

        if self.row.is_null(self.col_idx) {
            return visitor.visit_unit();
        }

        match type_name {
            "TINYINT" | "SMALLINT" | "INT" | "INTEGER" | "BIGINT" | "INT2" | "INT4" | "INT8" => {
                let v = self.row.get_i64(self.col_idx).map_err(de::Error::custom)?;
                visitor.visit_i64(v)
            }
            "FLOAT" | "DOUBLE" | "REAL" | "FLOAT4" | "FLOAT8" => {
                let v = self.row.get_f64(self.col_idx).map_err(de::Error::custom)?;
                visitor.visit_f64(v)
            }
            "BOOLEAN" | "BOOL" => {
                 let v = self.row.get_bool(self.col_idx).map_err(de::Error::custom)?;
                 visitor.visit_bool(v)
            }
            "VARCHAR" | "TEXT" | "CHAR" | "NAME" | "String" => {
                let v = self.row.get_string(self.col_idx).map_err(de::Error::custom)?;
                visitor.visit_string(v)
            }
            "JSON" | "JSONB" => {
                 let v = self.row.get_json(self.col_idx).map_err(de::Error::custom)?;
                 v.deserialize_any(visitor).map_err(de::Error::custom)
            }
            "DATETIME" | "TIMESTAMP" | "TIMESTAMPTZ" => {
                 let v = self.row.get_string(self.col_idx).map_err(de::Error::custom)?;
                 visitor.visit_string(v)
            }
            _ if type_name == "BLOB" => {
                 visitor.visit_unit()
            }
            _ => {
                // Fallback attempts
                if let Ok(v) = self.row.get_string(self.col_idx) {
                     visitor.visit_string(v)
                } else if let Ok(v) = self.row.get_i64(self.col_idx) {
                     visitor.visit_i64(v)
                } else if let Ok(v) = self.row.get_f64(self.col_idx) {
                     visitor.visit_f64(v)
                } else {
                     visitor.visit_unit()
                }
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.row.is_null(self.col_idx) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Ok(v) = self.row.get_bool(self.col_idx) {
            visitor.visit_bool(v)
        } else {
            self.deserialize_any(visitor)
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Ok(v) = self.row.get_i64(self.col_idx) {
            visitor.visit_i64(v)
        } else {
             self.deserialize_any(visitor)
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Ok(v) = self.row.get_f64(self.col_idx) {
            visitor.visit_f64(v)
        } else {
             self.deserialize_any(visitor)
        }
    }
    
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Ok(v) = self.row.get_string(self.col_idx) {
            visitor.visit_string(v)
        } else {
             self.deserialize_any(visitor)
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i128 u8 u16 u32 u64 u128 f32 char str 
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

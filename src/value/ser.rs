use std::fmt::Display;

use serde::ser;

use crate::error::{Error, Result};
use crate::path;
use crate::value::Value;

#[derive(Default, Debug)]
pub struct ValueSerializer {
    keys: Vec<(String, Option<usize>)>,
    pub output: Value,
}

impl ValueSerializer {
    fn serialize_primitive<T>(&mut self, value: T) -> Result<()>
    where
        T: Into<Value> + Display,
    {
        let key = match self.last_key_index_pair() {
            Some((key, Some(index))) => format!("{}[{}]", key, index),
            Some((key, None)) => key.to_string(),
            None => {
                return Err(Error::serde(format!(
                    "key is not found for value {}",
                    value
                )));
            }
        };
        let path: path::Path = key.parse()?;
        self.output.set(path, value.into())?;
        Ok(())
    }

    fn last_key_index_pair(&self) -> Option<(&str, Option<usize>)> {
        let len = self.keys.len();
        if len > 0 {
            self.keys
                .get(len - 1)
                .map(|&(ref key, opt)| (key.as_str(), opt))
        } else {
            None
        }
    }

    fn inc_last_key_index(&mut self) -> Result<()> {
        let len = self.keys.len();
        if len > 0 {
            self.keys
                .get_mut(len - 1)
                .map(|pair| pair.1 = pair.1.map(|i| i + 1).or(Some(0)))
                .ok_or_else(|| Error::serde(format!("last key is not found in {} keys", len)))
        } else {
            Err(Error::serde("keys is empty".to_string()))
        }
    }

    fn make_full_key(&self, key: &str) -> String {
        let len = self.keys.len();
        if len > 0 {
            if let Some(&(ref prev_key, index)) = self.keys.get(len - 1) {
                return if let Some(index) = index {
                    format!("{}[{}]/{}", prev_key, index, key)
                } else {
                    format!("{}/{}", prev_key, key)
                };
            }
        }
        format!("/{}", key)
    }

    fn push_key(&mut self, key: &str) {
        let full_key = self.make_full_key(key);
        self.keys.push((full_key, None));
    }

    fn pop_key(&mut self) -> Option<(String, Option<usize>)> {
        self.keys.pop()
    }
}

impl<'a> ser::Serializer for &'a mut ValueSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.serialize_primitive(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.serialize_primitive(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        if v > (i64::max_value() as u64) {
            Err(Error::serde(format!(
                "value {} is greater than the max {}",
                v,
                i64::max_value()
            )))
        } else {
            self.serialize_i64(v as i64)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.serialize_primitive(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_primitive(v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.serialize_primitive(v.to_string())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.serialize_primitive(Value::Nil)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(&variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        self.push_key(&variant);
        value.serialize(&mut *self)?;
        self.pop_key();
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.push_key(&variant);
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.push_key(&variant);
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.inc_last_key_index()?;
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.inc_last_key_index()?;
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.inc_last_key_index()?;
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.inc_last_key_index()?;
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.pop_key();
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let key_serializer = ToStringSerializer;
        let key = key.serialize(key_serializer)?;
        self.push_key(&key);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)?;
        self.pop_key();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.push_key(key);
        value.serialize(&mut **self)?;
        self.pop_key();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut ValueSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.push_key(key);
        value.serialize(&mut **self)?;
        self.pop_key();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.pop_key();
        Ok(())
    }
}

pub struct ToStringSerializer;

impl ser::Serializer for ToStringSerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(v.to_string())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        Ok(String::from_utf8_lossy(v).to_string())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(String::new())
    }

    fn serialize_unit_struct(self, _name: &str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &str,
        _variant_index: u32,
        variant: &str,
    ) -> Result<Self::Ok> {
        Ok(variant.to_string())
    }

    fn serialize_newtype_struct<T>(self, _name: &str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &str,
        _variant_index: u32,
        _variant: &str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::serde("seq can't serialize to string".to_string()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::serde("tuple can't serialize to string".to_string()))
    }

    fn serialize_tuple_struct(self, name: &str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        Err(Error::serde(format!(
            "tuple struct {} can't serialize to string",
            name
        )))
    }

    fn serialize_tuple_variant(
        self,
        name: &str,
        _variant_index: u32,
        variant: &str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::serde(format!(
            "tuple variant {}::{} can't serialize to string",
            name, variant
        )))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::serde(
            "map can't serialize to string key".to_string(),
        ))
    }

    fn serialize_struct(self, name: &str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::serde(format!(
            "struct {} can't serialize to string",
            name
        )))
    }

    fn serialize_struct_variant(
        self,
        name: &str,
        _variant_index: u32,
        variant: &str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::serde(format!(
            "struct variant {}::{} can't serialize to string",
            name, variant
        )))
    }
}

impl ser::SerializeSeq for ToStringSerializer {
    type Ok = String;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeTuple for ToStringSerializer {
    type Ok = String;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeTupleStruct for ToStringSerializer {
    type Ok = String;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeTupleVariant for ToStringSerializer {
    type Ok = String;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeMap for ToStringSerializer {
    type Ok = String;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeStruct for ToStringSerializer {
    type Ok = String;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeStructVariant for ToStringSerializer {
    type Ok = String;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

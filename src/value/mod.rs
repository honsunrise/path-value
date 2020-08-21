use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Display;

use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};
use serde::Serialize;

use crate::error::{Error, Result, Unexpected};
use crate::path::{Path, PathNode};
use crate::value::ser::ValueSerializer;

mod de;
mod ser;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Integer(BigInt),
    Float(f64),
    Boolean(bool),
    String(String),
    Map(HashMap<String, Value>),
    Array(Vec<Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::String(ref value) => write!(f, "{}", value),
            Value::Boolean(value) => write!(f, "{}", value),
            Value::Integer(ref value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Nil => write!(f, "nil"),
            Value::Map(ref map) => write!(f, "{:?}", map),
            Value::Array(ref array) => write!(f, "{:?}", array),
        }
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Value::Nil,
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(value: &'a str) -> Self {
        Value::String(value.into())
    }
}

macro_rules! impl_from_int_to_value {
    ($ty:ty) => {
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Value::Integer(BigInt::from(value))
            }
        }
    };
}

impl_from_int_to_value!(i8);
impl_from_int_to_value!(i16);
impl_from_int_to_value!(i32);
impl_from_int_to_value!(i64);
impl_from_int_to_value!(isize);

impl_from_int_to_value!(u8);
impl_from_int_to_value!(u16);
impl_from_int_to_value!(u32);
impl_from_int_to_value!(u64);
impl_from_int_to_value!(usize);

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value as f64)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl<T> From<HashMap<String, T>> for Value
where
    T: Into<Value>,
{
    fn from(values: HashMap<String, T>) -> Self {
        let mut r = HashMap::new();

        for (k, v) in values {
            r.insert(k.clone(), v.into());
        }

        Value::Map(r)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(values: Vec<T>) -> Self {
        let mut l = Vec::new();

        for v in values {
            l.push(v.into());
        }

        Value::Array(l)
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Boolean(value) => Ok(value),
            Value::Integer(value) => Ok(value.ne(&Zero::zero())),
            Value::Float(value) => Ok(value != 0.0),

            Value::String(ref value) => {
                match value.to_lowercase().as_ref() {
                    "1" | "true" | "on" | "yes" => Ok(true),
                    "0" | "false" | "off" | "no" => Ok(false),

                    // Unexpected string value
                    s => Err(Error::invalid_type(Unexpected::Str(s.into()), "a boolean")),
                }
            }

            // Unexpected type
            Value::Nil => Err(Error::invalid_type(Unexpected::Unit, "a boolean")),
            Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "a boolean")),
            Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "a boolean")),
        }
    }
}

macro_rules! impl_try_from_value_to_int {
    ($ty:ty, $ident:ident) => {
        impl TryFrom<Value> for $ty {
            type Error = Error;

            fn try_from(value: Value) -> Result<Self> {
                match value {
                    Value::Integer(value) => match value.$ident() {
                        Some(v) => Ok(v),
                        None => Err(Error::too_large(value)),
                    },
                    Value::String(ref s) => {
                        match s.to_lowercase().as_ref() {
                            "true" | "on" | "yes" => Ok(1),
                            "false" | "off" | "no" => Ok(0),
                            _ => {
                                s.parse().map_err(|_| {
                                    // Unexpected string
                                    Error::invalid_type(Unexpected::Str(s.clone()), "an integer")
                                })
                            }
                        }
                    }
                    Value::Boolean(value) => Ok(if value { 1 } else { 0 }),
                    Value::Float(value) => Ok(value.round() as $ty),

                    // Unexpected type
                    Value::Nil => Err(Error::invalid_type(Unexpected::Unit, "an integer")),
                    Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "an integer")),
                    Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "an integer")),
                }
            }
        }
    };
}

impl_try_from_value_to_int!(i8, to_i8);
impl_try_from_value_to_int!(i16, to_i16);
impl_try_from_value_to_int!(i32, to_i32);
impl_try_from_value_to_int!(i64, to_i64);
impl_try_from_value_to_int!(isize, to_isize);
impl_try_from_value_to_int!(u8, to_u8);
impl_try_from_value_to_int!(u16, to_u16);
impl_try_from_value_to_int!(u32, to_u32);
impl_try_from_value_to_int!(u64, to_u64);
impl_try_from_value_to_int!(usize, to_usize);

macro_rules! impl_try_from_value_to_float {
    ($ty:ty, $ident:ident) => {
        impl TryFrom<Value> for $ty {
            type Error = Error;

            fn try_from(value: Value) -> Result<Self> {
                match value {
                    Value::Float(value) => Ok(value as $ty),

                    Value::String(ref s) => {
                        match s.to_lowercase().as_ref() {
                            "true" | "on" | "yes" => Ok(1.0),
                            "false" | "off" | "no" => Ok(0.0),
                            _ => {
                                s.parse().map_err(|_| {
                                    // Unexpected string
                                    Error::invalid_type(
                                        Unexpected::Str(s.clone()),
                                        "a floating point",
                                    )
                                })
                            }
                        }
                    }

                    Value::Integer(value) => match value.$ident() {
                        Some(v) => Ok(v),
                        None => Err(Error::too_large(value)),
                    },
                    Value::Boolean(value) => Ok(if value { 1.0 } else { 0.0 }),

                    // Unexpected type
                    Value::Nil => Err(Error::invalid_type(Unexpected::Unit, "a floating point")),
                    Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "a floating point")),
                    Value::Array(_) => {
                        Err(Error::invalid_type(Unexpected::Array, "a floating point"))
                    }
                }
            }
        }
    };
}

impl_try_from_value_to_float!(f32, to_f32);
impl_try_from_value_to_float!(f64, to_f64);

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::String(value) => Ok(value),

            Value::Boolean(value) => Ok(value.to_string()),
            Value::Integer(value) => Ok(value.to_string()),
            Value::Float(value) => Ok(value.to_string()),

            // Cannot convert
            Value::Nil => Err(Error::invalid_type(Unexpected::Unit, "a string")),
            Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "a string")),
            Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "a string")),
        }
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Array(value) => Ok(value),

            // Cannot convert
            Value::Float(value) => Err(Error::invalid_type(Unexpected::Float(value), "an array")),
            Value::String(value) => Err(Error::invalid_type(Unexpected::Str(value), "an array")),
            Value::Integer(value) => {
                Err(Error::invalid_type(Unexpected::Integer(value), "an array"))
            }
            Value::Boolean(value) => Err(Error::invalid_type(Unexpected::Bool(value), "an array")),
            Value::Nil => Err(Error::invalid_type(Unexpected::Unit, "an array")),
            Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "an array")),
        }
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Map(value) => Ok(value),

            // Cannot convert
            Value::Float(value) => Err(Error::invalid_type(Unexpected::Float(value), "a map")),
            Value::String(value) => Err(Error::invalid_type(Unexpected::Str(value), "a map")),
            Value::Integer(value) => Err(Error::invalid_type(Unexpected::Integer(value), "a map")),
            Value::Boolean(value) => Err(Error::invalid_type(Unexpected::Bool(value), "a map")),
            Value::Nil => Err(Error::invalid_type(Unexpected::Unit, "a map")),
            Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "a map")),
        }
    }
}

impl TryFrom<Value> for () {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        Ok(())
    }
}

impl Value {
    pub fn merge(&mut self, source: Value) -> Result<()> {
        match self {
            Value::Boolean(v_t) => match source {
                Value::Boolean(v_s) => {
                    *v_t = v_s;
                    Ok(())
                }

                // Cannot convert
                Value::Float(value) => Err(Error::invalid_type(Unexpected::Float(value), "a bool")),
                Value::String(value) => Err(Error::invalid_type(Unexpected::Str(value), "a bool")),
                Value::Integer(value) => {
                    Err(Error::invalid_type(Unexpected::Integer(value), "a bool"))
                }
                Value::Nil => Ok(()),
                Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "a bool")),
                Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "a bool")),
            },
            Value::Integer(v_t) => match source {
                Value::Integer(v_s) => {
                    *v_t = v_s;
                    Ok(())
                }

                // Cannot convert
                Value::Float(value) => {
                    Err(Error::invalid_type(Unexpected::Float(value), "a integer"))
                }
                Value::String(value) => {
                    Err(Error::invalid_type(Unexpected::Str(value), "a integer"))
                }
                Value::Boolean(value) => {
                    Err(Error::invalid_type(Unexpected::Bool(value), "a integer"))
                }
                Value::Nil => Ok(()),
                Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "a integer")),
                Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "a integer")),
            },
            Value::Float(v_t) => match source {
                Value::Float(v_s) => {
                    *v_t = v_s;
                    Ok(())
                }

                // Cannot convert
                Value::Integer(value) => {
                    Err(Error::invalid_type(Unexpected::Integer(value), "a float"))
                }
                Value::String(value) => Err(Error::invalid_type(Unexpected::Str(value), "a float")),
                Value::Boolean(value) => {
                    Err(Error::invalid_type(Unexpected::Bool(value), "a float"))
                }
                Value::Nil => Ok(()),
                Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "a float")),
                Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "a float")),
            },
            Value::String(v_t) => match source {
                Value::String(v_s) => {
                    *v_t = v_s;
                    Ok(())
                }

                // Cannot convert
                Value::Integer(value) => {
                    Err(Error::invalid_type(Unexpected::Integer(value), "a string"))
                }
                Value::Float(value) => {
                    Err(Error::invalid_type(Unexpected::Float(value), "a string"))
                }
                Value::Boolean(value) => {
                    Err(Error::invalid_type(Unexpected::Bool(value), "a string"))
                }
                Value::Nil => Ok(()),
                Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "a string")),
                Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "a string")),
            },
            Value::Nil => match source {
                Value::Nil => Ok(()),
                _ => {
                    *self = source;
                    Ok(())
                }
            },
            Value::Map(v_t) => match source {
                Value::Map(v_s) => {
                    for (k, v) in v_s {
                        match v_t.get_mut(&k) {
                            Some(j) => Value::merge(j, v)?,
                            None => {
                                v_t.insert(k, v);
                            }
                        }
                    }
                    Ok(())
                }
                // Cannot convert
                Value::Integer(value) => {
                    Err(Error::invalid_type(Unexpected::Integer(value), "a map"))
                }
                Value::Float(value) => Err(Error::invalid_type(Unexpected::Float(value), "a map")),
                Value::Boolean(value) => Err(Error::invalid_type(Unexpected::Bool(value), "a map")),
                Value::String(value) => Err(Error::invalid_type(Unexpected::Str(value), "a map")),
                Value::Nil => Ok(()),
                Value::Array(_) => Err(Error::invalid_type(Unexpected::Array, "a map")),
            },
            Value::Array(v_t) => match source {
                Value::Array(v_s) => {
                    for (index, v) in v_s.into_iter().enumerate() {
                        match v_t.get_mut(index) {
                            Some(j) => Value::merge(j, v)?,
                            None => {
                                v_t.push(v);
                            }
                        }
                    }
                    Ok(())
                }
                // Cannot convert
                Value::Integer(value) => {
                    Err(Error::invalid_type(Unexpected::Integer(value), "a array"))
                }
                Value::Float(value) => {
                    Err(Error::invalid_type(Unexpected::Float(value), "a array"))
                }
                Value::Boolean(value) => {
                    Err(Error::invalid_type(Unexpected::Bool(value), "a array"))
                }
                Value::String(value) => Err(Error::invalid_type(Unexpected::Str(value), "a array")),
                Value::Nil => Ok(()),
                Value::Map(_) => Err(Error::invalid_type(Unexpected::Map, "a array")),
            },
        }
    }

    pub fn set<P, IntoValue, IntoErr>(
        &mut self,
        path: P,
        input_value: IntoValue,
    ) -> Result<Value, Error>
    where
        P: TryInto<Path, Error = IntoErr>,
        IntoValue: Into<Value>,
        IntoErr: Into<Error>,
    {
        let input_value = input_value.into();
        let path = path.try_into().map_err(|err| err.into())?;
        unsafe {
            let mut parent = self as *mut Value;
            let mut target = self as *mut Value;
            let v = vec![1];
            for sub_path in path.iter() {
                match *sub_path {
                    PathNode::Identifier(ref ident) => match &mut *parent {
                        Value::Map(parent_map) => {
                            target = parent_map.entry(ident.clone()).or_default();
                            parent = target;
                        }

                        _ => {
                            *parent = HashMap::<String, Value>::new().into();
                            if let Value::Map(parent_map) = &mut *parent {
                                target = parent_map.entry(ident.clone()).or_default();
                                parent = target;
                            } else {
                                unreachable!()
                            }
                        }
                    },
                    PathNode::Index(index) => match &mut *parent {
                        Value::Array(parent_array) => {
                            target = Value::get_array_slot(parent_array, index);
                            parent = target;
                        }

                        _ => {
                            *parent = vec![Value::default()].into();
                            if let Value::Array(parent_array) = &mut *parent {
                                target = Value::get_array_slot(parent_array, index);
                                parent = target;
                            } else {
                                unreachable!()
                            }
                        }
                    },
                }
            }

            let result = (*target).clone();
            *target = input_value;
            Ok(result)
        }
    }

    pub fn get<T, P, IntoErr>(&self, path: P) -> Result<Option<T>, Error>
    where
        T: std::convert::TryFrom<Value, Error = IntoErr>,
        P: TryInto<Path, Error = IntoErr>,
        IntoErr: Into<Error>,
    {
        let path = path.try_into().map_err(|err| err.into())?;
        let value = path
            .iter()
            .scan(self, |value, child_path| {
                let result = match *child_path {
                    PathNode::Identifier(ref id) => match **value {
                        Value::Map(ref map) => map.get(id),
                        _ => None,
                    },

                    PathNode::Index(index) => match **value {
                        Value::Array(ref array) => {
                            let index = Value::map_index(index, array.len());

                            if index >= array.len() {
                                None
                            } else {
                                Some(&array[index])
                            }
                        }
                        _ => None,
                    },
                };
                if let Some(v) = result {
                    *value = v;
                }
                result
            })
            .last();
        match value {
            None => Ok(None),
            Some(value) => Ok(Some(
                Value::try_into(value.clone()).map_err(|err: IntoErr| err.into())?,
            )),
        }
    }

    fn map_index(index: isize, len: usize) -> usize {
        if index >= 0 {
            index as usize
        } else {
            len - (index.abs() as usize)
        }
    }

    unsafe fn get_array_slot(array: &mut Vec<Value>, index: isize) -> *mut Value {
        let index = Value::map_index(index, array.len());
        match array.get_mut(index) {
            Some(v) => v,
            None => {
                array.insert(index, Value::default());
                match array.get_mut(index) {
                    Some(v) => v,
                    None => unreachable!(),
                }
            }
        }
    }
}

pub fn to_value<T>(from: T) -> Result<Value>
where
    T: Serialize,
{
    let mut serializer = ValueSerializer::default();
    from.serialize(&mut serializer)?;
    Ok(serializer.output)
}

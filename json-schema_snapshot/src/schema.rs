
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use serde;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Type {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "boolean")]
    Boolean,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> Deref for OneOrMany<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        match *self {
            OneOrMany::One(ref v) => unsafe { ::std::slice::from_raw_parts(v, 1) },
            OneOrMany::Many(ref v) => v,
        }
    }
}

impl<T> DerefMut for OneOrMany<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        match *self {
            OneOrMany::One(ref mut v) => unsafe { ::std::slice::from_raw_parts_mut(v, 1) },
            OneOrMany::Many(ref mut v) => v,
        }
    }
}

impl<T> Default for OneOrMany<T> {
    fn default() -> OneOrMany<T> {
        OneOrMany::Many(Vec::new())
    }
}

impl<T> serde::Deserialize for OneOrMany<T>
    where T: serde::Deserialize
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        T::deserialize(deserializer)
            .map(OneOrMany::One)
            .or_else(|_| Vec::<T>::deserialize(deserializer).map(OneOrMany::Many))
    }
}

impl<T> serde::Serialize for OneOrMany<T>
    where T: serde::Serialize
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        match *self {
            OneOrMany::One(ref one) => one.serialize(serializer),
            OneOrMany::Many(ref many) => many.serialize(serializer),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct Schema {
    pub id: Option<String>,

    #[serde(rename = "$ref")]
    pub ref_: Option<String>,

    #[serde(rename = "$schema")]
    pub schema: Option<String>,

    pub title: Option<String>,

    pub description: Option<String>,

    pub default: Option<Value>,

    pub multipleOf: Option<f64>,

    pub maximum: Option<f64>,

    #[serde(default)]
    pub exclusiveMaximum: bool,

    pub minimum: Option<f64>,

    #[serde(default)]
    pub exclusiveMinimum: bool,

    pub maxLength: Option<u64>,

    #[serde(default)]
    pub minLength: u64,

    pub pattern: Option<String>,

    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default)]
    pub additionalItems: Option<Value>,

    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default)]
    pub items: Option<Box<Schema>>,

    pub maxItems: Option<u64>,

    #[serde(default)]
    pub minItems: u64,

    #[serde(default)]
    pub uniqueItems: bool,

    pub maxProperties: Option<u64>,

    #[serde(default)]
    pub minProperties: u64,

    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub required: Vec<String>,

    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default)]
    pub additionalProperties: Option<Box<Schema>>,

    #[serde(skip_serializing_if="HashMap::is_empty")]
    #[serde(default)]
    pub definitions: HashMap<String, Schema>,

    #[serde(skip_serializing_if="HashMap::is_empty")]
    #[serde(default)]
    pub properties: HashMap<String, Schema>,

    #[serde(skip_serializing_if="HashMap::is_empty")]
    #[serde(default)]
    pub patternProperties: HashMap<String, Schema>,

    #[serde(skip_serializing_if="HashMap::is_empty")]
    #[serde(default)]
    pub dependencies: HashMap<String, Value>,

    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    #[serde(rename = "enum")]
    pub enum_: Vec<String>,

    #[serde(default)]
    #[serde(rename = "type")]
    pub type_: OneOrMany<Type>,

    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub allOf: Vec<Schema>,

    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub anyOf: Vec<Schema>,

    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub oneOf: Vec<Schema>,

    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default)]
    pub not: Option<Box<Schema>>,
}

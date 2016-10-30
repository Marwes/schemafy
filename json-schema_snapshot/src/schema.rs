
    use serde_json;
    use serde;
    
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq)]
pub enum OneOrMany<T> {
    One(Box<T>),
    Many(Vec<T>),
}

impl<T> Deref for OneOrMany<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        match *self {
            OneOrMany::One(ref v) => unsafe { ::std::slice::from_raw_parts(&**v, 1) },
            OneOrMany::Many(ref v) => v,
        }
    }
}

impl<T> DerefMut for OneOrMany<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        match *self {
            OneOrMany::One(ref mut v) => unsafe { ::std::slice::from_raw_parts_mut(&mut **v, 1) },
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
            .map(|one| OneOrMany::One(Box::new(one)))
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
pub type positiveInteger = i64;
pub type positiveIntegerDefault0 = serde_json::Value;
pub type schemaArray = Vec<Schema>;
# [ derive ( Clone , PartialEq , Debug , Deserialize , Serialize ) ]
pub enum simpleTypes {
    array,
    boolean,
    integer,
    null,
    number,
    object,
    string,
}
pub type stringArray = Vec<String>;
# [ derive ( Clone , PartialEq , Debug , Deserialize , Serialize ) ]
pub struct Schema {
    # [ serde ( rename = "$ref" ) ]
    pub ref_: Option<String>,
    # [ serde ( rename = "$schema" ) ]
    pub schema: Option<String>,
    # [ serde ( rename = "additionalItems" ) ]
    pub additional_items: Option<serde_json::Value>,
    # [ serde ( rename = "additionalProperties" ) ]
    pub additional_properties: Option<serde_json::Value>,
    # [ serde ( rename = "allOf" ) ]
    pub all_of: Option<schemaArray>,
    # [ serde ( rename = "anyOf" ) ]
    pub any_of: Option<schemaArray>,
    pub default: Option<serde_json::Value>,
    # [ serde ( default ) ]
    pub definitions: ::std::collections::BTreeMap<String, Schema>,
    pub dependencies: Option<::std::collections::BTreeMap<String, serde_json::Value>>,
    pub description: Option<String>,
    # [ serde ( rename = "enum" ) ]
    pub enum_: Option<Vec<serde_json::Value>>,
    # [ serde ( rename = "exclusiveMaximum" ) ]
    pub exclusive_maximum: Option<bool>,
    # [ serde ( rename = "exclusiveMinimum" ) ]
    pub exclusive_minimum: Option<bool>,
    pub id: Option<String>,
    # [ serde ( default ) ]
    pub items: OneOrMany<Schema>,
    # [ serde ( rename = "maxItems" ) ]
    pub max_items: Option<positiveInteger>,
    # [ serde ( rename = "maxLength" ) ]
    pub max_length: Option<positiveInteger>,
    # [ serde ( rename = "maxProperties" ) ]
    pub max_properties: Option<positiveInteger>,
    pub maximum: Option<f64>,
    # [ serde ( rename = "minItems" ) ]
    pub min_items: Option<positiveIntegerDefault0>,
    # [ serde ( rename = "minLength" ) ]
    pub min_length: Option<positiveIntegerDefault0>,
    # [ serde ( rename = "minProperties" ) ]
    pub min_properties: Option<positiveIntegerDefault0>,
    pub minimum: Option<f64>,
    # [ serde ( rename = "multipleOf" ) ]
    pub multiple_of: Option<f64>,
    pub not: Option<Box<Schema>>,
    # [ serde ( rename = "oneOf" ) ]
    pub one_of: Option<schemaArray>,
    pub pattern: Option<String>,
    # [ serde ( default ) ]
    # [ serde ( rename = "patternProperties" ) ]
    pub pattern_properties: ::std::collections::BTreeMap<String, Schema>,
    # [ serde ( default ) ]
    pub properties: ::std::collections::BTreeMap<String, Schema>,
    pub required: Option<stringArray>,
    pub title: Option<String>,
    # [ serde ( default ) ]
    # [ serde ( rename = "type" ) ]
    pub type_: OneOrMany<simpleTypes>,
    # [ serde ( rename = "uniqueItems" ) ]
    pub unique_items: Option<bool>,
}

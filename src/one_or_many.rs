use std::ops::{Deref, DerefMut};

use serde;

#[derive(Clone, PartialEq, Debug)]
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

pub fn deserialize<T, D>(deserializer: D) -> Result<OneOrMany<T>, D::Error>
    where T: serde::Deserialize,
          D: serde::Deserializer
{
    serde::Deserialize::deserialize(deserializer)
}

impl<T> serde::Deserialize for OneOrMany<T>
    where T: serde::Deserialize
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        use std::marker::PhantomData;
        use std::fmt;

        use serde::de::{self, Deserialize};
        use serde::de::value::{MapVisitorDeserializer, ValueDeserializer, SeqVisitorDeserializer};

        struct OneOrManyDeserializer<T>(PhantomData<T>);
        impl<T> serde::de::Visitor for OneOrManyDeserializer<T>
            where T: Deserialize
        {
            type Value = OneOrMany<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("one or many")
            }

            fn visit_i64<E>(self, value: i64) -> Result<OneOrMany<T>, E>
                where E: de::Error
            {
                Deserialize::deserialize(value.into_deserializer()).map(OneOrMany::One)
            }

            fn visit_u64<E>(self, value: u64) -> Result<OneOrMany<T>, E>
                where E: de::Error
            {
                Deserialize::deserialize(value.into_deserializer()).map(OneOrMany::One)
            }

            fn visit_str<E>(self, value: &str) -> Result<OneOrMany<T>, E>
                where E: de::Error
            {
                Deserialize::deserialize(value.into_deserializer()).map(OneOrMany::One)
            }

            fn visit_string<E>(self, value: String) -> Result<OneOrMany<T>, E>
                where E: de::Error
            {
                Deserialize::deserialize(value.into_deserializer()).map(OneOrMany::One)
            }

            fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::MapVisitor
            {
                Deserialize::deserialize(MapVisitorDeserializer::new(visitor)).map(OneOrMany::One)
            }

            fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
                where V: serde::de::SeqVisitor
            {
                Deserialize::deserialize(SeqVisitorDeserializer::new(visitor)).map(OneOrMany::Many)
            }
        }
        deserializer.deserialize(OneOrManyDeserializer(PhantomData::<T>))
    }
}

pub fn serialize<T, S>(value: &OneOrMany<T>, serializer: S) -> Result<S::Ok, S::Error>
    where T: serde::Serialize,
          S: serde::Serializer
{
    serde::Serialize::serialize(value, serializer)
}

impl<T> serde::Serialize for OneOrMany<T>
    where T: serde::Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        match *self {
            OneOrMany::One(ref one) => one.serialize(serializer),
            OneOrMany::Many(ref many) => many.serialize(serializer),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::from_str;

    #[test]
    fn deserialize_one_int() {
        assert_eq!(from_str::<OneOrMany<i32>>("1").unwrap(),
                   OneOrMany::One(Box::new(1)));
    }

    #[test]
    fn deserialize_many_int() {
        assert_eq!(from_str::<OneOrMany<i32>>("[1, 2, 3]").unwrap(),
                   OneOrMany::Many(vec![1, 2, 3]));
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Test {
        x: i32,
        y: Option<String>,
    }

    #[test]
    fn deserialize_one_struct() {
        assert_eq!(from_str::<OneOrMany<Test>>(r#"{ "x" : 10, "y" : "test" }"#).unwrap(),
                   OneOrMany::One(Box::new(Test {
                       x: 10,
                       y: Some("test".to_string()),
                   })));
    }

    #[test]
    fn deserialize_one_struct_missing_field() {
        assert_eq!(from_str::<OneOrMany<Test>>(r#"{ "x" : 10 }"#).unwrap(),
                   OneOrMany::One(Box::new(Test { x: 10, y: None })));
    }


    #[test]
    fn deserialize_many_struct() {
        assert_eq!(from_str::<OneOrMany<Test>>(r#"[{ "x" : 10 }, { "x" : 0, "y" : "a" }]"#)
                       .unwrap(),
                   OneOrMany::Many(vec![Test { x: 10, y: None },
                                        Test {
                                            x: 0,
                                            y: Some("a".to_string()),
                                        }]));
    }
}

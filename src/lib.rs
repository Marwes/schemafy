extern crate json_schema;
extern crate serde_json;

#[macro_use]
extern crate quote;

use std::borrow::Cow;
use std::error::Error;

use serde_json::Value;

use json_schema::{Schema, Type};

use quote::{Tokens, ToTokens};

struct Ident<S>(S);

impl<S: AsRef<str>> ToTokens for Ident<S> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(self.0.as_ref())
    }
}

const ONE_OR_MANY: &'static str = r#"
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
"#;

fn rename_keyword(prefix: &str, s: &str) -> Option<Tokens> {
    if ["type", "struct", "enum"].iter().any(|&keyword| keyword == s) {
        let n = Ident(format!("{}_", s));
        let prefix = Ident(prefix);
        Some(quote!{
            #[serde(rename = #s)]
            #prefix #n
        })
    } else {
        None
    }
}

fn field(s: &str) -> Tokens {
    if let Some(t) = rename_keyword("pub", s) {
        t
    } else {
        let mut snake = String::new();
        let mut chars = s.chars();
        let mut prev_was_upper = false;
        while let Some(c) = chars.next() {
            if c.is_uppercase() {
                if !prev_was_upper {
                    snake.push('_');
                }
                snake.extend(c.to_lowercase());
                prev_was_upper = true;
            } else {
                snake.push(c);
                prev_was_upper = false;
            }
        }
        if snake != s || snake.contains(|c: char| c == '$' || c == '#') {
            let field = Ident(snake.replace('$', "").replace('#', ""));
            quote!{
                #[serde(rename = #s)]
                pub #field
            }
        } else {
            let field = Ident(s);
            quote!( pub #field )
        }
    }
}

fn merge(result: &mut Schema, r: &Schema) {
    use std::collections::hash_map::Entry;

    for (k, v) in &r.properties {
        match result.properties.entry(k.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(v.clone());
            }
            Entry::Occupied(mut entry) => merge(entry.get_mut(), v),
        }
    }
}

struct FieldExpander<'a, 'r: 'a> {
    default: bool,
    expander: &'a mut Expander<'r>,
}

impl<'a, 'r> FieldExpander<'a, 'r> {
    fn expand_fields(&mut self, type_name: &str, schema: &Schema) -> Vec<Tokens> {
        let schema = self.expander.schema(schema);
        schema.properties
            .iter()
            .map(|(field_name, value)| {
                let key = field(field_name);
                let required = schema.required.iter().any(|req| req == field_name);
                let field_type = self.expander.expand_type(type_name, required, value);
                if !field_type.typ.starts_with("Option<") {
                    self.default = false;
                }
                let typ = Ident(field_type.typ);
                if field_type.default {
                    quote!( #[serde(default)] #key : #typ )
                } else {
                    quote!( #key : #typ )
                }
            })
            .collect()
    }
}

struct Expander<'r> {
    root_name: Option<&'r str>,
    root: &'r Schema,
    needs_one_or_many: bool,
}

struct FieldType {
    typ: String,
    default: bool,
}

impl<S> From<S> for FieldType
    where S: Into<String>
{
    fn from(s: S) -> FieldType {
        FieldType {
            typ: s.into(),
            default: false,
        }
    }
}

impl<'r> Expander<'r> {
    fn new(root_name: Option<&'r str>, root: &'r Schema) -> Expander<'r> {
        Expander {
            root_name: root_name,
            root: root,
            needs_one_or_many: false,
        }
    }

    fn type_ref(&self, s: &str) -> String {
        if s == "#" {
            self.root_name.expect("Root name").into()
        } else {
            s.split('/').last().expect("Component").into()
        }
    }

    fn schema(&self, schema: &'r Schema) -> Cow<'r, Schema> {
        let result = match schema.allOf.first() {
            Some(result) => {
                schema.allOf
                    .iter()
                    .skip(1)
                    .fold(Cow::Borrowed(result), |mut result, def| {
                        merge(result.to_mut(), &self.schema(def));
                        result
                    })
            }
            None => Cow::Borrowed(schema),
        };
        if let Some(ref ref_) = result.ref_ {
            return Cow::Borrowed(self.schema_ref(ref_));
        }
        result
    }

    fn schema_ref(&self, s: &str) -> &'r Schema {
        s.split('/').fold(self.root, |schema, comp| {
            if comp == "#" {
                self.root
            } else if comp == "definitions" {
                schema
            } else {
                schema.definitions
                    .get(comp)
                    .unwrap_or_else(|| panic!("Expected definition: `{}` {}", s, comp))
            }
        })
    }

    fn expand_type(&mut self, type_name: &str, required: bool, typ: &Schema) -> FieldType {
        let mut result = self.expand_type_(typ);
        if type_name == result.typ {
            result.typ = format!("Box<{}>", result.typ)
        }
        if !required && !result.default {
            result.typ = format!("Option<{}>", result.typ)
        }
        result
    }

    fn expand_type_(&mut self, typ: &Schema) -> FieldType {
        if let Some(ref ref_) = typ.ref_ {
            self.type_ref(ref_).into()
        } else if typ.anyOf.len() == 2 {
            let simple = self.schema(&typ.anyOf[0]);
            let array = self.schema(&typ.anyOf[1]);
            match array.items {
                Some(ref item_schema) => {
                    if array.type_[0] == Type::Array && simple == self.schema(item_schema) {
                        self.needs_one_or_many = true;
                        return format!("OneOrMany<{}>", self.expand_type_(&typ.anyOf[0]).typ)
                            .into();
                    }
                }
                _ => (),
            }
            return "serde_json::Value".into();
        } else if typ.type_.len() == 1 {
            match typ.type_[0] {
                Type::String => {
                    if !typ.enum_.is_empty() {
                        "serde_json::Value".into()
                    } else {
                        "String".into()
                    }
                }
                Type::Integer => "i64".into(),
                Type::Boolean => "bool".into(),
                Type::Number => "f64".into(),
                Type::Object if typ.additionalProperties.is_some() => {
                    let prop = typ.additionalProperties.as_ref().unwrap();
                    let result =
                        format!("::std::collections::HashMap<String, {}>", self.expand_type_(prop).typ);
                    FieldType {
                        typ: result,
                        default: typ.default == Some(Value::Object(Default::default())),
                    }
                }
                Type::Array => {
                    let item_type =
                        typ.items.as_ref().map_or("serde_json::Value".into(),
                                                  |item_schema| self.expand_type_(item_schema));
                    format!("Vec<{}>", item_type.typ).into()
                }
                _ => "serde_json::Value".into(),
            }
        } else {
            "serde_json::Value".into()
        }
    }

    pub fn expand_definitions(&mut self, schema: &Schema) -> Vec<Tokens> {
        let mut types = Vec::new();
        for (name, def) in &schema.definitions {
            types.push(self.expand_schema(name, def));
        }
        types
    }

    pub fn expand_schema(&mut self, name: &str, schema: &Schema) -> Tokens {
        let (fields, default) = {
            let mut field_expander = FieldExpander {
                default: true,
                expander: self,
            };
            let fields = field_expander.expand_fields(name, schema);
            (fields, field_expander.default)
        };

        let name = Ident(name);
        if !fields.is_empty() {
            if default {
                quote! {
                    #[derive(Default, Deserialize, Serialize)]
                    pub struct #name {
                        #(#fields),*
                    }
                }
            } else {
                quote! {
                    #[derive(Deserialize, Serialize)]
                    pub struct #name {
                        #(#fields),*
                    }
                }
            }
        } else if !schema.enum_.is_empty() {
            let variants = schema.enum_.iter().map(|v| {
                rename_keyword("", v).unwrap_or_else(|| {
                    let v = Ident(v);
                    quote!(#v)
                })
            });
            quote! {
                #[derive(Deserialize, Serialize)]
                pub enum #name {
                    #(#variants),*
                }
            }
        } else {
            let typ = Ident(self.expand_type("", true, schema).typ);
            quote! {
                pub type #name = #typ;
            }
        }
    }

    pub fn expand(&mut self, schema: &Schema) -> Tokens {
        let mut types = self.expand_definitions(schema);
        if let Some(name) = self.root_name {
            types.push(self.expand_schema(name, schema));
        }

        let one_or_many = Ident(if self.needs_one_or_many {
            ONE_OR_MANY
        } else {
            ""
        });

        quote! {
            #one_or_many
            
            #( #types )*
        }
    }
}

pub fn generate(root_name: Option<&str>, s: &str) -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let schema = serde_json::from_str(s).unwrap();
    let mut expander = Expander::new(root_name, &schema);
    let output = expander.expand(&schema).to_string();
    let mut child =
        try!(Command::new("rustfmt").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn());
    try!(child.stdin.as_mut().expect("stdin").write_all(output.as_bytes()));
    let output = try!(child.wait_with_output());
    assert!(output.status.success());
    Ok(try!(String::from_utf8(output.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    #[test]
    fn generate_schema() {
        let s = include_str!("../../json-schema/tests/schema.json");

        let s = generate(Some("Schema"), s).unwrap().to_string();

        verify_compile("schema", &s);

        assert!(s.contains("pub struct Schema"), "{}", s);
        assert!(s.contains("pub type positiveInteger = i64"));
        assert!(s.contains("pub type_: Option<OneOrMany<simpleTypes>>"));

        let result = Command::new("rustc")
            .args(&["-L",
                    "target/debug/deps/",
                    "-o",
                    "target/debug/schema-test",
                    "tests/support/schema-test.rs"])
            .status()
            .unwrap();

        assert!(result.success());
        let result = Command::new("./target/debug/schema-test")
            .status()
            .unwrap();

        assert!(result.success());
    }

    fn verify_compile(name: &str, s: &str) {

        let mut filename = PathBuf::from("target/debug");
        filename.push(&format!("{}.rs", name));
        {
            let mut file = File::create(&filename).unwrap();
            let header = r#"
            #![feature(proc_macro)]
            
            extern crate serde;
            #[macro_use]
            extern crate serde_derive;
            extern crate serde_json;
            "#;
            file.write_all(header.as_bytes()).unwrap();
            file.write_all(s.as_bytes()).unwrap();
        }
        println!("{}", filename.display());
        let child = Command::new("rustc")
            .args(&["-L",
                    "target/debug/deps/",
                    "--crate-type=rlib",
                    "-o",
                    &format!("target/debug/deps/lib{}.rlib", name),
                    filename.to_str().unwrap()])
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();


        let output = child.wait_with_output().unwrap();
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(output.status.success(), "{}", error);
    }

    #[test]
    fn builds_with_rustc() {
        let s = include_str!("../../json-schema/tests/debugserver-schema.json");

        let s = generate(None, s).unwrap().to_string();

        verify_compile("debug-server", &s)
    }
}

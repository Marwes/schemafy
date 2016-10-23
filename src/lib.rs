extern crate json_schema;
extern crate serde_json;

#[macro_use]
extern crate quote;

use std::borrow::Cow;
use std::error::Error;

use json_schema::{Schema, Type};

use quote::{Tokens, ToTokens};

struct Ident<S>(S);

impl<S: AsRef<str>> ToTokens for Ident<S> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(self.0.as_ref())
    }
}

fn field(s: &str) -> Ident<Cow<str>> {
    Ident(if ["type", "struct", "enum"].iter().any(|&keyword| keyword == s) {
        format!("{}_", s).into()
    } else {
        s.into()
    })
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

struct Expander<'r> {
    root: &'r Schema,
}

impl<'r> Expander<'r> {
    fn type_ref(&self, s: &str) -> String {
        s.split('/').last().expect("Component").into()
    }

    fn schema(&self, s: &'r Schema) -> &'r Schema {
        if let Some(ref ref_) = s.ref_ {
            self.schema_ref(ref_)
        } else {
            s
        }
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

    fn expand_type(&mut self, typ: &Schema) -> String {
        if let Some(ref ref_) = typ.ref_ {
            self.type_ref(ref_)
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
                Type::Object => "serde_json::Value".into(),
                Type::Array => {
                    let item_schema =
                        typ.items.as_ref().expect("Array type must have items schema");
                    format!("Vec<{}>", self.expand_type(item_schema))
                }
                _ => panic!("Type"),
            }
        } else {
            "serde_json::Value".into()
        }
    }

    fn expand_fields(&mut self, schema: &Schema) -> Vec<Tokens> {
        if let Some(ref ref_) = schema.ref_ {
            let schema = self.schema_ref(ref_);
            self.expand_fields(schema)
        } else if !schema.allOf.is_empty() {
            let first = schema.allOf.first().unwrap().clone();
            let result = schema.allOf
                .iter()
                .skip(1)
                .fold(first, |mut result, def| {
                    merge(&mut result, self.schema(def));
                    result
                });
            self.expand_fields(&result)
        } else {
            schema.properties
                .iter()
                .map(|(key, value)| {
                    let key = field(key);
                    let typ = Ident(self.expand_type(value));
                    quote!( pub #key : #typ )
                })
                .collect()
        }
    }

    pub fn expand_schema(&mut self, schema: &Schema) -> Tokens {
        let mut types = Vec::new();
        for (name, def) in &schema.definitions {
            let fields = self.expand_fields(def);
            let name = Ident(name);
            let tokens = quote! {
                #[derive(Deserialize, Serialize)]
                pub struct #name {
                    #(#fields),*
                }
            };
            types.push(tokens);
        }
        quote! { #(
            #types
            )*
        }
    }
}

pub fn generate(s: &str) -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let schema = serde_json::from_str(s).unwrap();
    let mut expander = Expander { root: &schema };
    let output = expander.expand_schema(&schema).to_string();
    let mut child =
        try!(Command::new("rustfmt").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn());
    try!(child.stdin.as_mut().expect("stdin").write_all(output.as_bytes()));
    let output = try!(child.wait_with_output());
    Ok(try!(String::from_utf8(output.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attempt() {
        let s = include_str!("../../json-schema/tests/debugserver-schema.json");

        let s = generate(s).unwrap().to_string();
        println!("{}", s);
        assert!(false);
    }
}

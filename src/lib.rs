extern crate json_schema;
extern crate serde_json;

#[macro_use]
extern crate quote;

use std::error::Error;

use json_schema::{Schema, Type};

use quote::{Tokens, ToTokens};

struct Ident<S>(S);

impl<S: AsRef<str>> ToTokens for Ident<S> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(self.0.as_ref())
    }
}

fn field(s: &str) -> Tokens {
    if ["type", "struct", "enum"].iter().any(|&keyword| keyword == s) {
        let n = Ident(format!("{}_", s));
        quote!( #n )
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

struct Expander<'r> {
    root_name: Option<&'r str>,
    root: &'r Schema,
}

impl<'r> Expander<'r> {
    fn type_ref(&self, s: &str) -> String {
        if s == "#" {
            self.root_name.expect("Root name").into()
        } else {
            s.split('/').last().expect("Component").into()
        }
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

    fn expand_type(&mut self, type_name: &str, typ: &Schema) -> String {
        let result = self.expand_type_(typ);
        if type_name == result {
            format!("Box<{}>", result)
        } else {
            result
        }
    }
    fn expand_type_(&mut self, typ: &Schema) -> String {
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
                    let item_type =
                        typ.items.as_ref().map_or("serde_json::Value".into(),
                                                  |item_schema| self.expand_type_(item_schema));
                    format!("Vec<{}>", item_type)
                }
                _ => panic!("Type"),
            }
        } else {
            "serde_json::Value".into()
        }
    }

    fn expand_fields(&mut self, type_name: &str, schema: &Schema) -> Vec<Tokens> {
        if let Some(ref ref_) = schema.ref_ {
            let schema = self.schema_ref(ref_);
            self.expand_fields(type_name, schema)
        } else if !schema.allOf.is_empty() {
            let first = schema.allOf.first().unwrap().clone();
            let result = schema.allOf
                .iter()
                .skip(1)
                .fold(first, |mut result, def| {
                    merge(&mut result, self.schema(def));
                    result
                });
            self.expand_fields(type_name, &result)
        } else {
            schema.properties
                .iter()
                .map(|(key, value)| {
                    let key = field(key);
                    let typ = Ident(self.expand_type(type_name, value));
                    quote!( #key : #typ )
                })
                .collect()
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
        let fields = self.expand_fields(name, schema);
        let name = Ident(name);
        quote! {
            #[derive(Deserialize, Serialize)]
            pub struct #name {
                #(#fields),*
            }
        }
    }

    pub fn expand(&mut self, schema: &Schema) -> Tokens {
        let mut types = self.expand_definitions(schema);
        if let Some(name) = self.root_name {
            types.push(self.expand_schema(name, schema));
        }

        quote! { #(
            #types
            )*
        }
    }
}

pub fn generate(root_name: Option<&str>, s: &str) -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let schema = serde_json::from_str(s).unwrap();
    let mut expander = Expander {
        root_name: root_name,
        root: &schema,
    };
    let output = expander.expand(&schema).to_string();
    let mut child =
        try!(Command::new("rustfmt").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn());
    try!(child.stdin.as_mut().expect("stdin").write_all(output.as_bytes()));
    let output = try!(child.wait_with_output());
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

        assert!(s.contains("pub struct Schema"), "{}", s);

        verify_compile("schema.rs", &s);
    }

    fn verify_compile(name: &str, s: &str) {

        let mut filename = PathBuf::from("target/debug");
        filename.push(name);
        {
            let mut file = File::create(&filename).unwrap();
            let header = r#"
            #![feature(proc_macro)]
            
            #[macro_use]
            extern crate serde_derive;
            extern crate serde_json;
            "#;
            file.write_all(header.as_bytes()).unwrap();
            file.write_all(s.as_bytes()).unwrap();
        }

        let child = Command::new("rustc")
            .args(&["-L", "target/debug/deps/", "--crate-type=rlib", filename.to_str().unwrap()])
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

        verify_compile("debug-server.rs", &s)
    }
}

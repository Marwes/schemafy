extern crate json_schema;

#[macro_use]
extern crate quote;

use std::borrow::Cow;

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

struct Expander;

impl Expander {
    fn type_ref(&self, s: &str) -> String {
        s.split('/').last().expect("Component").into()
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
                Type::Object => "obj".into(),
                Type::Array => "array".into(),
                _ => panic!("Type"),
            }
        } else {
            "serde_json::Value".into()
        }
    }

    pub fn expand_schema(&mut self, schema: &Schema) -> Tokens {
        let mut types = Vec::new();
        for (name, def) in &schema.definitions {
            let fields = def.properties.iter().map(|(key, value)| {
                let key = field(key);
                let typ = Ident(self.expand_type(value));
                quote!( #key : #typ )
            });
            let name = Ident(name);
            let tokens = quote! {
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


#[cfg(test)]
mod tests {
    use super::*;
    use super::Expander;
    extern crate serde_json;

    #[test]
    fn attempt() {
        let s = include_str!("../../json-schema/tests/debugserver-schema.json");
        let schema = serde_json::from_str(s).unwrap();

        let mut expander = Expander;
        let s = expander.expand_schema(&schema).to_string();
        println!("`{}`", s);
        assert!(false);
    }
}

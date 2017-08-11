extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate quote;
extern crate inflector;
extern crate itertools;

pub mod one_or_many;
pub mod schema;

use self as schemafy;

use std::borrow::Cow;
use std::error::Error;

use inflector::Inflector;

use serde_json::Value;

use schema::{Schema, SimpleTypes};

use itertools::Itertools;

use quote::{Tokens, ToTokens};

struct Ident<S>(S);

impl<S: AsRef<str>> ToTokens for Ident<S> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(self.0.as_ref())
    }
}

fn rename_keyword(prefix: &str, s: &str) -> Option<Tokens> {
    if ["type", "struct", "enum"]
           .iter()
           .any(|&keyword| keyword == s) {
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
        let snake = s.to_snake_case();
        if snake != s || snake.contains(|c: char| c == '$' || c == '#') {
            let field = if snake == "$ref" {
                Ident("ref_".into())
            } else {
                Ident(snake.replace('$', "").replace('#', ""))
            };

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

fn merge_option<T, F>(mut result: &mut Option<T>, r: &Option<T>, f: F)
    where F: FnOnce(&mut T, &T),
          T: Clone
{
    *result = match (&mut result, r) {
        (&mut &mut Some(ref mut result), &Some(ref r)) => return f(result, r),
        (&mut &mut None, &Some(ref r)) => Some(r.clone()),
        _ => return (),
    };
}

fn merge_all_of(result: &mut Schema, r: &Schema) {
    use std::collections::btree_map::Entry;

    for (k, v) in &r.properties {
        match result.properties.entry(k.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(v.clone());
            }
            Entry::Occupied(mut entry) => merge_all_of(entry.get_mut(), v),
        }
    }

    if let Some(ref ref_) = r.ref_ {
        result.ref_ = Some(ref_.clone());
    }

    if let Some(ref description) = r.description {
        result.description = Some(description.clone());
    }

    merge_option(&mut result.required,
                 &r.required,
                 |required, r_required| { required.extend(r_required.iter().cloned()); });

    result.type_.retain(|e| r.type_.contains(e));
}

const LINE_LENGTH: usize = 100;
const INDENT_LENGTH: usize = 4;

fn make_doc_comment(mut comment: &str, remaining_line: usize) -> String {
    let mut out_comment = String::new();
    out_comment.push_str("/// ");
    let mut length = 4;
    while let Some(word) = comment.split(char::is_whitespace).next() {
        if comment.is_empty() {
            break;
        }
        comment = &comment[word.len()..];
        if length + word.len() >= remaining_line {
            out_comment.push_str("\n/// ");
            length = 4;
        }
        out_comment.push_str(word);
        length += word.len();
        let mut n = comment.chars();
        match n.next() {
            Some('\n') => {
                out_comment.push_str("\n");
                out_comment.push_str("/// ");
                length = 4;
            }
            Some(_) => {
                out_comment.push_str(" ");
                length += 1;
            }
            None => (),
        }
        comment = n.as_str();
    }
    if out_comment.ends_with(' ') {
        out_comment.pop();
    }
    out_comment.push_str("\n");
    out_comment
}

struct FieldExpander<'a, 'r: 'a> {
    default: bool,
    expander: &'a mut Expander<'r>,
}

impl<'a, 'r> FieldExpander<'a, 'r> {
    fn expand_fields(&mut self, type_name: &str, schema: &Schema) -> Vec<Tokens> {
        let schema = self.expander.schema(schema);
        schema
            .properties
            .iter()
            .map(|(field_name, value)| {
                self.expander.current_field.clone_from(field_name);
                let key = field(field_name);
                let required = schema
                    .required
                    .iter()
                    .flat_map(|a| a.iter())
                    .any(|req| req == field_name);
                let field_type = self.expander.expand_type(type_name, required, value);
                if !field_type.typ.starts_with("Option<") {
                    self.default = false;
                }
                let typ = Ident(field_type.typ);

                let default = if field_type.default {
                    Some(Ident("#[serde(default)]"))
                } else {
                    None
                };
                let attributes = if field_type.attributes.is_empty() {
                    None
                } else {
                    Some(Ident(format!("#[serde({})]", field_type.attributes.iter().format(", "))))
                };
                let comment =
                    value
                        .description
                        .as_ref()
                        .map(|comment| {
                                 Ident(make_doc_comment(comment, LINE_LENGTH - INDENT_LENGTH))
                             });
                quote!{
                    #comment
                    #default
                    #attributes
                    #key : #typ 
                }
            })
            .collect()
    }
}

struct Expander<'r> {
    root_name: Option<&'r str>,
    root: &'r Schema,
    current_type: String,
    current_field: String,
    types: Vec<(String, Tokens)>,
}

struct FieldType {
    typ: String,
    attributes: Vec<String>,
    default: bool,
}

impl<S> From<S> for FieldType
    where S: Into<String>
{
    fn from(s: S) -> FieldType {
        FieldType {
            typ: s.into(),
            attributes: Vec::new(),
            default: false,
        }
    }
}

impl<'r> Expander<'r> {
    fn new(root_name: Option<&'r str>, root: &'r Schema) -> Expander<'r> {
        Expander {
            root_name: root_name,
            root: root,
            current_field: "".into(),
            current_type: "".into(),
            types: Vec::new(),
        }
    }

    fn type_ref(&self, s: &str) -> String {
        if s == "#" {
            self.root_name.expect("Root name").to_pascal_case()
        } else {
            s.split('/')
                .last()
                .expect("Component")
                .to_pascal_case()
        }
    }

    fn schema(&self, schema: &'r Schema) -> Cow<'r, Schema> {
        let schema = match schema.ref_ {
            Some(ref ref_) => self.schema_ref(ref_),
            None => schema,
        };
        match schema.all_of {
            Some(ref all_of) if !all_of.is_empty() => {
                all_of
                    .iter()
                    .skip(1)
                    .fold(self.schema(&all_of[0]).clone(), |mut result, def| {
                        merge_all_of(result.to_mut(), &self.schema(def));
                        result
                    })
            }
            _ => Cow::Borrowed(schema),
        }
    }

    fn schema_ref(&self, s: &str) -> &'r Schema {
        s.split('/')
            .fold(self.root, |schema, comp| if comp == "#" {
                self.root
            } else if comp == "definitions" {
                schema
            } else {
                schema
                    .definitions
                    .get(comp)
                    .unwrap_or_else(|| panic!("Expected definition: `{}` {}", s, comp))
            })
    }

    fn expand_type(&mut self, type_name: &str, required: bool, typ: &Schema) -> FieldType {
        if type_name == "StoppedEvent" {
            println!("{:#?}", typ);
        }
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
        } else if typ.any_of.as_ref().map_or(false, |a| a.len() == 2) {
            let any_of = typ.any_of.as_ref().unwrap();
            let simple = self.schema(&any_of[0]);
            let array = self.schema(&any_of[1]);
            if let SimpleTypes::Array = array.type_[0] {
                if simple == self.schema(&array.items[0]) {
                    return FieldType {
                               typ: format!("Vec<{}>", self.expand_type_(&any_of[0]).typ),
                               attributes: vec![r#"serialize_with="::schemafy::one_or_many::serialize""#
                                                    .into(),
                                                r#"deserialize_with="schemafy::one_or_many::deserialize""#
                                                    .into()],
                               default: true,
                           };
                }
            }
            return "serde_json::Value".into();
        } else if typ.type_.len() == 1 {
            match typ.type_[0] {
                SimpleTypes::String => {
                    if typ.enum_.as_ref().map_or(false, |e| e.is_empty()) {
                        "serde_json::Value".into()
                    } else {
                        "String".into()
                    }
                }
                SimpleTypes::Integer => "i64".into(),
                SimpleTypes::Boolean => "bool".into(),
                SimpleTypes::Number => "f64".into(),
                SimpleTypes::Object if typ.additional_properties.is_some() => {
                    let prop = serde_json::from_value(typ.additional_properties.clone().unwrap())
                        .unwrap();
                    let result = format!("::std::collections::BTreeMap<String, {}>",
                                         self.expand_type_(&prop).typ);
                    FieldType {
                        typ: result,
                        attributes: Vec::new(),
                        default: typ.default == Some(Value::Object(Default::default())),
                    }
                }
                // Handle objects defined inline
                SimpleTypes::Object if !typ.properties.is_empty() => {
                    let name = format!("{}{}",
                                       self.current_type.to_pascal_case(),
                                       self.current_field.to_pascal_case());
                    let tokens = self.expand_schema(&name, typ);
                    self.types.push((name.clone(), tokens));
                    name.into()
                }
                SimpleTypes::Array => {
                    let item_type = typ.items
                        .get(0)
                        .map_or("serde_json::Value".into(),
                                |item| self.expand_type_(item).typ);
                    format!("Vec<{}>", item_type).into()
                }
                _ => "serde_json::Value".into(),
            }
        } else {
            "serde_json::Value".into()
        }
    }

    pub fn expand_definitions(&mut self, schema: &Schema) {
        for (name, def) in &schema.definitions {
            let type_decl = self.expand_schema(name, def);
            let definition_tokens = match def.description {
                Some(ref comment) => {
                    let t = Ident(make_doc_comment(comment, LINE_LENGTH));
                    quote! {
                        #t
                        #type_decl
                    }
                }
                None => type_decl,
            };
            self.types.push((name.to_string(), definition_tokens));
        }
    }

    pub fn expand_schema(&mut self, original_name: &str, schema: &Schema) -> Tokens {
        self.expand_definitions(schema);

        let pascal_case_name = original_name.to_pascal_case();
        self.current_type.clone_from(&pascal_case_name);
        let (fields, default) = {
            let mut field_expander = FieldExpander {
                default: true,
                expander: self,
            };
            let fields = field_expander.expand_fields(original_name, schema);
            (fields, field_expander.default)
        };
        let name = Ident(pascal_case_name);
        let type_decl = if !fields.is_empty() {
            if default {
                quote! {
                    #[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
                    pub struct #name {
                        #(#fields),*
                    }
                }
            } else {
                quote! {
                    #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
                    pub struct #name {
                        #(#fields),*
                    }
                }
            }
        } else if schema.enum_.as_ref().map_or(false, |e| !e.is_empty()) {
            let variants = schema
                .enum_
                .as_ref()
                .map_or(&[][..], |v| v)
                .iter()
                .map(|v| match *v {
                         Value::String(ref v) => {
                    let pascal_case_variant = v.to_pascal_case();
                    let variant_name = rename_keyword("", &pascal_case_variant)
                        .unwrap_or_else(|| {
                                            let v = Ident(&pascal_case_variant);
                                            quote!(#v)
                                        });
                    if pascal_case_variant == *v {
                        variant_name
                    } else {
                        quote! {
                                #[serde(rename = #v)]
                                #variant_name
                            }
                    }
                }
                         _ => panic!("Expected string"),
                     });
            quote! {
                #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
                pub enum #name {
                    #(#variants),*
                }
            }
        } else {
            let typ = Ident(self.expand_type("", true, schema).typ);
            return quote! {
                pub type #name = #typ;
            };
        };
        if original_name == name.0 {
            type_decl
        } else {
            quote! {
                #[serde(rename = #original_name)]
                #type_decl
            }
        }
    }

    pub fn expand(&mut self, schema: &Schema) -> Tokens {
        match self.root_name {
            Some(name) => {
                let schema = self.expand_schema(name, schema);
                self.types.push((name.to_string(), schema));
            }
            None => self.expand_definitions(schema),
        }

        let types = self.types.iter().map(|t| &t.1);

        quote! {
            #( #types )*
        }
    }
}

fn format(output: &str) -> Result<String, Box<Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = try!(Command::new("rustfmt")
                             .stdin(Stdio::piped())
                             .stdout(Stdio::piped())
                             .spawn());
    try!(child
             .stdin
             .as_mut()
             .expect("stdin")
             .write_all(output.as_bytes()));
    let output = try!(child.wait_with_output());
    if !output.status.success() {
        return Err("rustfmt returned an error".into())
    }
    Ok(try!(String::from_utf8(output.stdout)))
}

pub fn generate(root_name: Option<&str>, s: &str) -> Result<String, Box<Error>> {
    let schema = serde_json::from_str(s).unwrap();
    let mut expander = Expander::new(root_name, &schema);
    let output = expander.expand(&schema).to_string();

    Ok(format(&output).unwrap_or_else(|_| output))
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
        let s = include_str!("schema.json");

        let s = generate(Some("Schema"), s).unwrap().to_string();
        let s = s.replace("\r\n", "\n");

        verify_compile("schema", &s);

        assert!(s.contains("pub struct Schema"), "{}", s);
        assert!(s.contains("pub type PositiveInteger = i64"), s);
        assert!(s.contains("pub type_: Vec<SimpleTypes>"), s);
        assert!(s.contains("pub enum SimpleTypes {\n    #[serde(rename = \"array\")]"),
                s);

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
            extern crate serde;
            #[macro_use]
            extern crate serde_derive;
            extern crate serde_json;
            extern crate schemafy;
            "#;
            file.write_all(header.as_bytes()).unwrap();
            file.write_all(s.as_bytes()).unwrap();
        }

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
    fn debugserver_types() {
        let s = include_str!("../tests/debugserver-schema.json");

        let s = generate(None, s).unwrap().to_string();

        verify_compile("debug-server", &s);

        assert!(s.contains("pub arguments: SourceArguments,"));
    }

    #[test]
    fn nested_definition() {
        let s = include_str!("../tests/nested.json");

        let s = generate(None, s).unwrap().to_string();

        verify_compile("debug-server", &s);

        assert!(s.contains("pub struct Defnested"));
    }
}

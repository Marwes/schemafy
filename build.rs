extern crate schemafy_snapshot;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let schema = "src/schema.json";
    println!("cargo:rerun-if-changed={}", schema);
    let src = Path::new(schema);

    let mut file = File::open(src).unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();

    let output = schemafy_snapshot::generate(Some("Schema"), &input).unwrap();
    let dst = Path::new("src/schema.rs");

    let mut file = File::create(dst).unwrap();
    file.write_all(br#"
    use one_or_many;
    use serde_json;
    "#)
        .unwrap();
    file.write_all(output.as_bytes()).unwrap();
}

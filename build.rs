extern crate json_schema_snapshot;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let src = Path::new("src/schema.json");

    let mut file = File::open(src).unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();

    let output = json_schema_snapshot::generate(Some("Schema"), &input).unwrap();
    let dst = Path::new("src/schema.rs");

    let mut file = File::create(dst).unwrap();
    file.write_all(br#"
    use serde_json;
    use serde;
    "#);
    file.write_all(output.as_bytes()).unwrap();
}

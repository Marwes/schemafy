use schemafy_lib::Expander;
use std::path::PathBuf;

#[test]
fn schema() {
    let json = std::fs::read_to_string("src/schema.json").expect("Read schema JSON file");

    let schema = serde_json::from_str(&json).unwrap_or_else(|err| panic!("{}", err));
    let mut expander = Expander::new(Some("Schema"), "UNUSED", &schema, PathBuf::from("src"));

    expander.expand(&schema);
}

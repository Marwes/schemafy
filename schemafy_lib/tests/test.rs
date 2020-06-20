use schemafy_lib::Expander;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[test]
fn schema() {
    let json = std::fs::read_to_string("src/schema.json").expect("Read schema JSON file");

    let schema = Rc::new(RefCell::new(
        serde_json::from_str(&json).unwrap_or_else(|err| panic!("{}", err)),
    ));
    let mut expander = Expander::new(
        Some("Schema"),
        "UNUSED",
        schema.clone(),
        PathBuf::from("src"),
    );

    let schema_ref = &schema.borrow();

    expander.expand(schema_ref);
}
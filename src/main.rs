use schemafy_helper;


use serde::{Deserialize, Serialize};

schemafy::regenerate!(
    root: Schema
    "src/schema.json"
);

fn main() {}

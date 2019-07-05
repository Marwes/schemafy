extern crate schemafy_helper;
extern crate serde;

use serde::{Deserialize, Serialize};

schemafy::regenerate!(
    root: Schema
    "src/schema.json"
);

fn main() {}

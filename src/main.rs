use schemafy_core;

use serde::{Deserialize, Serialize};

schemafy::regenerate!(
    root: Schema
    "src/schema.json"
);

fn main() {}

//! Generate test cases from the JSON Schema Test Suite.

use inflector::Inflector;
use serde::{Deserialize, Serialize};
use std::{error::Error, ffi::OsStr, fs, path::PathBuf};

// Each test has a description, schema, and a list of tests. Each of
// those tests has a description, some data, and a `valid` field which
// indicates whether that data should validate against the schema.
schemafy::schemafy!(
    root: TestSchema
    "tests/JSON-Schema-Test-Suite/test-schema.json"
);

fn main() -> Result<(), Box<dyn Error>> {
    let test_suite_dir = PathBuf::from("tests/test_suite");
    let schemas_dir = test_suite_dir.join("schemas");
    if test_suite_dir.exists() {
        fs::remove_dir_all("tests/test_suite")?;
    }
    fs::create_dir(&test_suite_dir)?;
    fs::create_dir(&schemas_dir)?;

    let mut test_modules = vec![];
    let mut blacklist_count = 0;

    for path in fs::read_dir("tests/JSON-Schema-Test-Suite/tests/draft4")?
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension() == Some(OsStr::new("json")))
    {
        let buffer = fs::read_to_string(&path)?;
        let test_schema: TestSchema = serde_json::from_str(&buffer)?;
        println!("{} ==> {} tests", path.display(), test_schema.len());

        let module_name = path.file_stem().unwrap().to_str().unwrap().to_snake_case();

        let mut test_file: String = format!(
            r#"//! Automatically generated from {}
"#,
            path.display()
        );

        for (i, test_group) in test_schema.iter().enumerate() {
            if is_blacklisted(&module_name, i) {
                blacklist_count += 1;
                println!(" !! skipping test group: {}", test_group.description);
                continue;
            }

            let schema_name = format!("{}_{}.json", module_name, i);
            let schema = serde_json::to_string(&test_group.schema)?;
            fs::write(schemas_dir.join(&schema_name), schema)?;

            test_file.push_str(&format!(
                r#"
mod {} {{
    #[allow(unused_imports)]
    use serde::{{Deserialize, Serialize}};

    schemafy::schemafy!(root: Schema "tests/test_suite/schemas/{}");
"#,
                test_group.description.to_snake_case(),
                schema_name
            ));
            for test in &test_group.tests {
                let test_name = {
                    // Prefix the name with an underscore if it starts
                    // with a number.
                    let root = test.description.to_snake_case();
                    let prefix = match root.chars().next().unwrap().is_numeric() {
                        true => "_",
                        false => "",
                    };
                    format!("{}{}", prefix, root)
                };

                // For the positive test cases, unwrapping the result
                // gives better error messages than simply asserting
                // on .is_ok(). For the negative test cases, a simple
                // assert is the best we can do.
                let assertion = match test.valid {
                    true => "let _: Schema = serde_json::from_str(&data).unwrap();",
                    false => "assert!(serde_json::from_str::<Schema>(&data).is_err());",
                };

                test_file.push_str(&format!(
                    r##"
    #[test]
    fn r#{}() {{
        let data = r#"{}"#;
        {}
    }}
"##,
                    test_name, test.data, assertion
                ));
            }
            test_file.push_str("}\n");
        }

        fs::write(
            test_suite_dir.join(format!("{}.rs", module_name)),
            test_file,
        )?;
        test_modules.push(module_name);
    }

    // Generate a root module that declares all the above files.
    let mut tests: String = r#"//! Automatically generated
"#
    .into();
    for module in &test_modules {
        tests.push_str(&format!("mod r#{};\n", module));
    }
    fs::write(test_suite_dir.join("mod.rs"), tests)?;

    if blacklist_count > 0 {
        println!("\nSkipped {} test schemas\n", blacklist_count);
    }

    Ok(())
}

/// To allow for gradual progress, this function determines whether a
/// test should be skipped.
fn is_blacklisted(test_group: &str, index: usize) -> bool {
    match test_group {
        "additional_items" if index == 0 || index == 2 => true,

        "additional_properties"
        | "all_of"
        | "any_of"
        | "definitions"
        | "dependencies"
        | "enum"
        | "items"
        | "max_items"
        | "max_length"
        | "max_properties"
        | "maximum"
        | "min_items"
        | "min_length"
        | "min_properties"
        | "minimum"
        | "multiple_of"
        | "not"
        | "one_of"
        | "pattern"
        | "pattern_properties"
        | "properties"
        | "ref"
        | "ref_remote"
        | "required"
        | "type"
        | "unique_items" => true,

        _ => false,
    }
}

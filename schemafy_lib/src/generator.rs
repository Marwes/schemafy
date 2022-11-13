use crate::{Expander, ExpanderOptions};
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
pub struct GeneratorOptions {
    pub deny_unknown_fields: bool,
}

/// A configurable builder for generating Rust types from a JSON
/// schema.
///
/// The default options are usually fine. In that case, you can use
/// the [`generate()`](fn.generate.html) convenience method instead.
#[derive(Debug, PartialEq)]
#[must_use]
pub struct Generator<'a, 'b> {
    /// The name of the root type defined by the schema. If the schema
    /// does not define a root type (some schemas are simply a
    /// collection of definitions) then simply pass `None`.
    pub root_name: Option<String>,
    /// The module path to this crate. Some generated code may make
    /// use of types defined in this crate. Unless you have
    /// re-exported this crate or imported it under a different name,
    /// the default should be fine.
    pub schemafy_path: &'a str,
    /// The JSON schema file to read
    pub input_file: &'b Path,

    pub generator_options: &'a GeneratorOptions,
}

impl<'a, 'b> Generator<'a, 'b> {
    /// Get a builder for the Generator
    pub fn builder() -> GeneratorBuilder<'a, 'b> {
        GeneratorBuilder::default()
    }

    pub fn generate(&self) -> proc_macro2::TokenStream {
        let input_file = if self.input_file.is_relative() {
            let crate_root = get_crate_root().unwrap();
            crate_root.join(self.input_file)
        } else {
            PathBuf::from(self.input_file)
        };

        let json = std::fs::read_to_string(&input_file).unwrap_or_else(|err| {
            panic!("Unable to read `{}`: {}", input_file.to_string_lossy(), err)
        });

        let schema = serde_json::from_str(&json).unwrap_or_else(|err| {
            panic!(
                "Cannot parse `{}` as JSON: {}",
                input_file.to_string_lossy(),
                err
            )
        });
        let expander_options = ExpanderOptions {
            deny_unknown_fields: self.generator_options.deny_unknown_fields,
        };
        let mut expander = Expander::new(
            self.root_name.as_deref(),
            self.schemafy_path,
            &schema,
            &expander_options,
        );
        expander.expand(&schema)
    }

    pub fn generate_to_file<P: ?Sized + AsRef<Path>>(&self, output_file: &'b P) -> io::Result<()> {
        use std::process::Command;
        let tokens = self.generate();
        let out = tokens.to_string();
        std::fs::write(output_file, &out)?;
        Command::new("rustfmt")
            .arg(output_file.as_ref().as_os_str())
            .output()?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
#[must_use]
pub struct GeneratorBuilder<'a, 'b> {
    inner: Generator<'a, 'b>,
}

impl<'a, 'b> Default for GeneratorBuilder<'a, 'b> {
    fn default() -> Self {
        Self {
            inner: Generator {
                root_name: None,
                schemafy_path: "::schemafy_core::",
                input_file: Path::new("schema.json"),
                generator_options: &GeneratorOptions {
                    deny_unknown_fields: false,
                },
            },
        }
    }
}

impl<'a, 'b> GeneratorBuilder<'a, 'b> {
    pub fn with_root_name(mut self, root_name: Option<String>) -> Self {
        self.inner.root_name = root_name;
        self
    }
    pub fn with_root_name_str(mut self, root_name: &str) -> Self {
        self.inner.root_name = Some(root_name.to_string());
        self
    }
    pub fn with_input_file<P: ?Sized + AsRef<Path>>(mut self, input_file: &'b P) -> Self {
        self.inner.input_file = input_file.as_ref();
        self
    }
    pub fn with_schemafy_path(mut self, schemafy_path: &'a str) -> Self {
        self.inner.schemafy_path = schemafy_path;
        self
    }
    pub fn with_options(mut self, generator_options: &'a GeneratorOptions) -> Self {
        self.inner.generator_options = generator_options;
        self
    }
    pub fn build(self) -> Generator<'a, 'b> {
        self.inner
    }
}

fn get_crate_root() -> std::io::Result<PathBuf> {
    if let Ok(path) = std::env::var("CARGO_MANIFEST_DIR") {
        return Ok(PathBuf::from(path));
    }

    let current_dir = std::env::current_dir()?;

    for p in current_dir.ancestors() {
        if std::fs::read_dir(p)?
            .into_iter()
            .filter_map(Result::ok)
            .any(|p| p.file_name().eq("Cargo.toml"))
        {
            return Ok(PathBuf::from(p));
        }
    }

    Ok(current_dir)
}

use std::io::copy;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};
use schemafy_lib::{Generator, GeneratorOptions};
use structopt::StructOpt;
use tempfile::NamedTempFile;

/// Generate Rust structs from a JSON schema
#[derive(Debug, StructOpt)]
struct Opts {
    /// Name for the root structure
    #[structopt(short, long, value_name = "NAME", default_value = "Schema")]
    root: String,
    /// Output file [default: stdout]
    #[structopt(short, long, value_name = "PATH")]
    output: Option<String>,
    /// JSON schema file
    schema_path: String,
    /// Flag to allow new fields from the server.
    #[structopt(long)]
    allow_unknown_fields: bool,
}

pub fn main() -> Result<()> {
    let opts = Opts::from_args();
    let generator_options = GeneratorOptions {
        deny_unknown_fields: !opts.allow_unknown_fields,
    };
    // generate the Rust code
    let mut generated_file = NamedTempFile::new()?;
    Generator::builder()
        .with_root_name_str(&opts.root)
        .with_input_file(&opts.schema_path)
        .with_options(&generator_options)
        .build()
        .generate_to_file(
            &generated_file
                .path()
                .to_str()
                .ok_or_else(|| anyhow!("converting output path"))?,
        )?;

    // run it through rustfmt and write it out
    let (output_file, output_path) = NamedTempFile::new_in(
        opts.output
            .as_ref()
            .map(|p| Path::new(p).parent())
            .flatten()
            .unwrap_or(&std::env::temp_dir()),
    )
    .context("creating temporary output file")?
    .into_parts();
    let mut formatter = Command::new("rustfmt")
        .args(&["--edition", "2018"])
        .stdin(Stdio::piped())
        .stdout(
            opts.output
                .as_ref()
                .and(Some(output_file.into()))
                .unwrap_or(Stdio::inherit()),
        )
        .spawn()
        .context("running rustfmt")?;
    copy(
        generated_file.as_file_mut(),
        formatter.stdin.as_mut().expect("stdin"),
    )?;
    let result = formatter.wait()?;
    if !result.success() {
        bail!("rustfmt failed");
    }
    if let Some(path) = &opts.output {
        output_path.persist(path)?;
    }

    Ok(())
}

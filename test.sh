#!/bin/sh
cargo build --features internal-regenerate

cargo run --bin generate-tests --features="generate-tests"
cargo fmt --all
cargo test --all 

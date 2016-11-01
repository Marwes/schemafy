# schemafy

This is a Rust crate which can take a [json schema](http://json-schema.org/) and generate Rust types which are serializable with [serde](https://serde.rs/). No checking such as `min_value` are done but instead only the structure of the schema is followed as closely as possible.

As a schema could be arbitrarily complex this crate makes no guarantee that it can generate good types or even any types at all for a given schema but the crate does manage to bootstrap itself which is kind of cool.

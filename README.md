# schemafy

[![Build Status](https://travis-ci.org/Marwes/schemafy.svg?branch=master)](https://travis-ci.org/Marwes/schemafy)

This is a Rust crate which can take a [json schema (draft 4)](http://json-schema.org/) and generate Rust types which are serializable with [serde](https://serde.rs/). No checking such as `min_value` are done but instead only the structure of the schema is followed as closely as possible.

As a schema could be arbitrarily complex this crate makes no guarantee that it can generate good types or even any types at all for a given schema but the crate does manage to bootstrap itself which is kind of cool.

## Example

Generated types for VS Codes [debug server protocol][]: https://docs.rs/debugserver-types

[debug server protocol]:https://code.visualstudio.com/docs/extensions/example-debuggers

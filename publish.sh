#!/bin/sh
(cd schemafy_snapshot && cargo publish) &&
    cargo release $@

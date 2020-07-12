#!/bin/sh

(./version.sh $@ &&
    (cd schemafy_core && cargo publish) &&
    cargo publish)

#!/bin/sh

(./version.sh $@ &&
    (cd schemafy_core && cargo publish) &&
    sleep 25 &&
    (cd schemafy_lib && cargo publish) &&
    sleep 25 &&
    cargo publish)

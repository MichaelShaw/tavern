#!/bin/sh
RUST_TEST_THREADS=1 cargo test $1 -- --nocapture

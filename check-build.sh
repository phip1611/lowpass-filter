#!/bin/sh

echo "checks that this builds on std+no_std + that all tests run"

cargo build --all-targets # build works
cargo test --all-targets # tests work
rustup target add thumbv7em-none-eabihf
cargo check --target thumbv7em-none-eabihf # test no_std-build

cargo doc
cargo fmt -- --check
cargo clippy --all-targets

# run examples
cargo run --example lpf-example-minimal

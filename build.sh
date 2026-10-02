#!/usr/bin/env bash
# Builds the Rust engine to WebAssembly into web/src/lib/pkg/, where the site imports it from.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
# Keep the build machine's paths out of the published file (they'd show up in error messages).
# The encoded form allows spaces in paths; its flags are separated by \x1f.
sep=$'\x1f'
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$HOME/.cargo/registry/src=/cargo${sep}--remap-path-prefix=$PWD=/unpacked${sep}--remap-path-prefix=$HOME=/home"

cargo build --manifest-path engine/Cargo.toml --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir web/src/lib/pkg \
  engine/target/wasm32-unknown-unknown/release/engine.wasm
ls -lh web/src/lib/pkg

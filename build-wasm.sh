#! /bin/bash
wasm-pack build --target web --no-typescript --no-pack --release --out-dir ./wasm
rm ./wasm/.gitignore
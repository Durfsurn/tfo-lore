#! /bin/bash
wasm-pack build --target web --no-typescript --no-pack --release --out-dir ./docs --no-opt
rm ./docs/.gitignore
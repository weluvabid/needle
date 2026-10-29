# needle-wasm

Thin `wasm-bindgen` adapter for `needle-core`. It contains no language or execution semantics.

Build from the repository root:

```sh
wasm-pack build crates/needle-wasm --target web --out-dir ../../web/src/wasm --out-name needle_wasm --release
```

The generated binding files are build artifacts and are not committed.

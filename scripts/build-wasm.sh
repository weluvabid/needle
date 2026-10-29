#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
wasm-pack build "$ROOT/crates/needle-wasm" \
  --target web \
  --out-dir "$ROOT/web/src/wasm" \
  --out-name needle_wasm \
  --release

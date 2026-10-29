#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

cargo fmt --all -- --check
cargo clippy -p needle-core --all-targets -- -D warnings
cargo test -p needle-core

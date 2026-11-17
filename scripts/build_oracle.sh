#!/usr/bin/env bash
# Build the pinned provari oracle in /tmp/470M1-oracle. Never commits it.
set -euo pipefail

PIN=bceb1ee1faf7d9cf2422eb89c931be69ae30619d
ROOT=${ORACLE_ROOT:-/tmp/470M1-oracle}
SRC="$ROOT/provari"

mkdir -p "$ROOT"
if [ ! -d "$SRC/.git" ]; then
  git clone https://github.com/aiorla/provari.git "$SRC"
fi
git -C "$SRC" fetch --quiet origin "$PIN"
git -C "$SRC" checkout --quiet "$PIN"
test "$(git -C "$SRC" rev-parse HEAD)" = "$PIN"

cargo build --release --manifest-path "$SRC/Cargo.toml"
"$SRC/target/release/provari" --version | tee "$ROOT/version.txt"

#!/usr/bin/env bash
# Build the pinned provari oracle in /tmp/470M1-oracle. Never commits it.
set -euo pipefail

PIN=b87d179c01ef8eae92a8ae93e4ca8b3df924dd71
VERSION="provari 0.5.0, with its network and model built in"
ROOT=${ORACLE_ROOT:-/tmp/470M1-oracle}
SRC="$ROOT/provari"

mkdir -p "$ROOT"
if [ ! -d "$SRC/.git" ]; then
  git clone https://github.com/aiorla/provari.git "$SRC"
fi
git -C "$SRC" fetch --quiet origin "$PIN"
git -C "$SRC" checkout --quiet "$PIN"
test "$(git -C "$SRC" rev-parse HEAD)" = "$PIN"

# The bundled build carries data/network.nnue and data/model.json.
(cd "$SRC" && ./data/fetch.sh)
cargo build --release --features bundled --manifest-path "$SRC/Cargo.toml"

BIN="$SRC/target/release/provari"
got=$("$BIN" --version)
test "$got" = "$VERSION" || { echo "provari version mismatch: $got" >&2; exit 1; }

{
  echo "$got"
  echo "commit $PIN"
  echo "binary $(sha256sum "$BIN" | cut -d' ' -f1)"
  echo "network $(sha256sum "$SRC/data/network.nnue" | cut -d' ' -f1)"
  echo "model $(sha256sum "$SRC/data/model.json" | cut -d' ' -f1)"
} | tee "$ROOT/version.txt"

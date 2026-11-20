#!/bin/sh
# Guard for the `unsafe-optimizations` dependency feature: run the pinned
# crate's own test suite in the shipped shape, from the registry copy that
# Cargo actually builds. Read-only for this repository.
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
src=$(cd "$root" && cargo metadata --format-version 1 --locked --offline |
	python3 -c 'import json,sys; d=json.load(sys.stdin); print(next(p["manifest_path"] for p in d["packages"] if p["name"]=="atomic-movegen" and p["version"]=="2.11.0").rsplit("/",1)[0])')
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cp -a "$src/." "$tmp/"
cd "$tmp"
cargo test --offline --locked --features unsafe-optimizations --quiet

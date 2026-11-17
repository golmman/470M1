#!/usr/bin/env bash
# Every dependency must be GPL-3.0-or-later compatible. The licence gate is
# never weakened: an unknown or incompatible expression fails.
set -euo pipefail
cd "$(dirname "$0")/.."

metadata=$(mktemp)
trap 'rm -f "$metadata"' EXIT
cargo metadata --format-version 1 >"$metadata"

python3 - "$metadata" <<'PY'
import json, re, sys
allowed = (
    "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib",
    "Unicode-3.0", "Unicode-DFS-2016", "Unlicense", "CC0-1.0", "GPL-2.0-or-later",
    "GPL-3.0-or-later", "LGPL-2.1-or-later", "LGPL-3.0-or-later",
)
with open(sys.argv[1]) as file:
    data = json.load(file)
root = data["resolve"]["root"]
bad = []
for package in data["packages"]:
    if package["id"] == root:
        continue
    expression = package.get("license") or ""
    if not expression:
        bad.append((package["name"], "<no license field>"))
        continue
    tokens = [token for token in re.split(r"[()\s]+|AND|OR", expression) if token]
    if not tokens or any(token not in allowed for token in tokens):
        bad.append((package["name"], expression))
if bad:
    for name, expression in bad:
        print(f"licence gate: {name}: {expression}", file=sys.stderr)
    raise SystemExit(1)
print("licence gate: all dependencies are GPL-3.0-or-later compatible")
PY

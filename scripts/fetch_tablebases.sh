#!/usr/bin/env bash
# Fetch and verify Lichess's Atomic 3-4-5 win tables. Never commits them.
#
# The manifest's own digest and every file's md5 are checked; any mismatch
# aborts, so a row cannot be measured against a partial or damaged set.
set -euo pipefail

BASE=${TABLEBASE_BASE:-https://tablebase.lichess.ovh/tables/atomic/3-4-5}
ROOT=${ORACLE_ROOT:-/tmp/470M1-oracle}
DIR=${TABLEBASES_DIR:-$ROOT/tablebases/3-4-5}
MANIFEST=3-4-5.atbw.md5
MANIFEST_SHA256=a0c7a69bc720dae8c77a776810d2f33486c153655cebdef229e27a40fdeefc37

mkdir -p "$DIR"
cd "$DIR"

if [ ! -f "$MANIFEST" ] || [ "$(sha256sum "$MANIFEST" | cut -d' ' -f1)" != "$MANIFEST_SHA256" ]; then
  curl -fsSL -o "$MANIFEST.part" "$BASE/$MANIFEST"
  mv "$MANIFEST.part" "$MANIFEST"
fi
test "$(sha256sum "$MANIFEST" | cut -d' ' -f1)" = "$MANIFEST_SHA256"

while read -r md5 name; do
  if [ ! -f "$name" ]; then
    curl -fsSL -o "$name.part" "$BASE/$name"
    mv "$name.part" "$name"
  fi
  echo "$md5  $name" | md5sum -c --quiet - || {
    echo "md5 mismatch: $name" >&2
    exit 1
  }
done < "$MANIFEST"

echo "tablebases: $(wc -l < "$MANIFEST") files in $DIR"
echo "manifest sha256: $MANIFEST_SHA256"

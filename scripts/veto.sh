#!/usr/bin/env bash
# Our checker and the pinned provari agree on the fixtures and controls.
set -euo pipefail
cd "$(dirname "$0")/.."

ORACLE=${PROVARI_ORACLE:-/tmp/470M1-oracle/provari/target/release/provari}
test -x "$ORACLE" || { echo "missing oracle: run make oracle" >&2; exit 1; }

cargo build --quiet
OURS=target/debug/470m1

for proof in tests/fixtures/proofs/*.pvr; do
  echo "== accept $proof"
  ours=$($OURS check "$proof")
  theirs=$("$ORACLE" check "$proof")
  python3 - "$ours" "$theirs" "$proof" <<'PY'
import json, sys
ours, theirs, path = json.loads(sys.argv[1]), json.loads(sys.argv[2]), sys.argv[3]
assert ours["valid"] is True, (path, ours)
assert theirs["valid"] is True, (path, theirs)
PY
done

for control in tests/fixtures/controls/*.pvr; do
  echo "== reject $control"
  ours=$($OURS check "$control" || true)
  theirs=$("$ORACLE" check "$control" || true)
  python3 - "$ours" "$theirs" "$control" <<'PY'
import json, sys
ours, theirs, path = json.loads(sys.argv[1]), json.loads(sys.argv[2]), sys.argv[3]
assert ours["valid"] is False, (path, ours)
assert theirs["valid"] is False, (path, theirs)
PY
done

echo "veto: all fixtures accepted, all controls rejected"

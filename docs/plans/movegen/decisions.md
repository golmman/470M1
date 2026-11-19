# Decisions — `movegen`

Full text for the decisions summarised in [`initiative.md`](initiative.md).
A decision is changed by a new dated row, never by editing history silently.
The `MD` prefix keeps these distinct from the parity `D1`–`D12` records and
from the bench diagnostic labelled `D14`.

## MD1 — Definition of done

**Resolution (2026-11-19).** `movegen` is done only when the **prover** no
longer uses shakmaty: FEN parsing, identity/hash, move generation, effects,
legality, terminality, proof writing/reading and line display all run on
`atomic-movegen`. The independent checker `src/check.rs` is the one exception:
it stays shakmaty-only and shares no code with the prover, so the two rules
implementations remain independent (`D4`, `D11`). "Only `atomic-movegen`"
therefore describes the prover, not the whole crate.

The performance gate is **no regression** against the current 10 s bench
baseline, with both checkers accepting every claim. The stretch goal is a
measured improvement over the shakmaty-based prover. A stage that regresses is
a finding, not an accepted trade.

**Rationale.** Complete removal is the owner's goal; keeping the parser or the
display path on shakmaty would leave the replacement partial. The checker is
the external veto and must not share the prover's rules implementation.

**Consequences.** Parser removal (W2) and display/proof-path removal are in
scope, not later cleanups. The crate will still depend on shakmaty for the
checker even after the prover is clean. A missing display API (for example SAN)
becomes a wishlist item, not a reason to keep shakmaty in the prover.

## MD2 — Key strategy

**Resolution (2026-11-19).** Use `atomic-movegen`'s current keys by default
and measure. If they are not slower, keep them and drop numeric
shakmaty-compatible key parity as a gate; semantic key agreement and the
existing differential remain. If they are slower and the key is the cause,
formulate the W5 wishlist so `atomic-movegen` can produce shakmaty-compatible
keys, then adopt it by the normal owner-triggered plan/report and pin update.
The end state has only `atomic-movegen` in the prover and must be faster than
the shakmaty-based baseline.

**Rationale.** Keys only index in-memory tables; proofs and the checker do not
contain them. Native keys are the simplest and likely fastest path, but the
owner will not accept a key-driven regression.

**Consequences.** Plan 3 must record a native-key bench before the key
question is closed. W5 is evidence-triggered. A built-in shakmaty table in the
MIT upstream crate needs an owner licensing decision; the preferred W5 shape
is an injectable/compat table hook so the GPL table stays in this GPL
repository.

## MD3 — EP-blind `threats`

**Resolution (2026-11-19).** Option B: do not change `threats` now. Add a
read-only census over the corpus and play-outs for positions where an
en-passant explosion exists and `threats` would differ. If the census finds a
material difference, a later plan makes `threats` EP-aware and measures it; if
not, close the item with the recorded count.

**Rationale.** This is a score-only undercount, not a soundness gap. The
repo's rule is to measure before changing search or evaluation.

**Consequences.** Plan 3 records the census result but does not change eval.
A future EP-aware change needs its own bench evidence.

## MD4 — Product name

**Resolution (2026-11-19).** The final product name is **`470MI`** (capital
`M`, capital `I`), replacing the working name `470m1` (digit one). The
mechanical mapping is: package and library `atomic470mi`, binary and
product-facing name `470MI`. The rename is applied before the first adapter
bench so artifacts are not mixed. Current docs, scripts, `Makefile`, tests and
CLI text use the new name; historical plans, reports and measurement files are
not rewritten.

**Rationale.** The owner has made the name final. A single mechanical rename
before new measurements avoids a second baseline and mixed binary paths.

**Consequences.** `Cargo.toml`, `Cargo.lock`, `src/`, `tests/`, `scripts/`,
`bench/`, `Makefile` and current docs need the rename. The repository
directory name is not part of the build and is left to the owner.

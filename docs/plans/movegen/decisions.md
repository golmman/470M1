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

**Reaffirmed (2026-11-19).** A measured rollback to the provari-derived hot path
(plan 2's binary reads 0.987 in the plan 4 session) was proposed and **rejected**:
the crate is the rules layer even when that is slower today. The regression is
closed by making the crate fast, not by keeping a second implementation.

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

## MD5 — Hot-path boundary

**Resolution (2026-11-19).** The prover's rules hot path calls `atomic-movegen`
directly. There is no translation layer: a per-candidate conversion, a
rebuilt `Setup`, a cached board copy or a second legality implementation in
the prover is a defect, not a convenience. Boundary conversions are allowed
and expected only at the edges — FEN parsing (until `M8`), `to_shakmaty` for
the checker and proof I/O, and the `.pvr` move bits in `format.rs`. If a
conversion or a re-derivation is needed on the hot path, the fix is a
`wishlistN.md` request for a leaner crate API.

**Rationale.** The measured per-node regression is the translator: plan 2's
shape and the crate's shape differ in representation, move encoding, node
state and key derivation. Keeping the translator and optimising inside it was
measured at ~5%, while every candidate change inside the translator (a cached
`StateInfo`, `EffectContext` for effect/legality) regressed
([`hot-path-audit.md`](hot-path-audit.md)).

**Consequences.** `src/board.rs`/`src/moves.rs` shrink to direct calls plus
the boundary; `M2` moves the prover's move storage to the crate's stable `u16`
encoding; `M8` removes the parser and `to_shakmaty` from the prover path.

## MD6 — What "no regression" means

**Resolution (2026-11-19).** "No regression" is a same-session comparison: the
candidate's 10 s geomean against the plan 2 reference binary (`470m1`) measured
in the same session with the same driver and protocol, requiring **3/3 cells
and 9/9 accepted by both checkers**, with the reference reading within noise of
its recorded **0.972**. The recorded 0.972 is an anchor, not a target, and
`1.0` against provari is not the bar.

**Rationale.** Host drift is larger than the difference between 0.972 and the
0.987 re-read of the same old binary in the plan 4 session. An absolute number
measured in another session would pass or fail on noise.

**Consequences.** Every adoption plan runs the reference and the candidate
interleaved in one session and records both in `measurements/planN/`. A pass
that beats `1.0` but loses to the same-session reference does not pass.

## MD7 — Rules residual accepted

**Resolution (2026-11-19).** The `MD6` miss is accepted as a dated miss, not
carried into a fourth `atomic-movegen` release. On `core` the adopted tree is
**7.8162e9** instructions against plan 2's **6.5694e9** (1.190×), a gap of
**1.2468e9**: rules **0.617e9** (49.5%), node geometry **0.15–0.36e9**
depending on how `attackers_to`'s plan-2 inlining is aligned, generation
**0.1255e9**, tree **0.1249e9** and search/table **0.2505e9**. The audit's
crate-led ceiling is ≈**1.12×**; a perfect rules close still lands near 1.035
candidate/old against a reference reading 0.987.

Plan 8 measured the one cheap rules lever that 2.10.0 already contains: the
`FastLegality` ladder. Exposed and wired into `Mover::legal`'s fallback in a
scratch tree, it is correct — identical counters, mate and `.pvr` — but the
best of three shapes costs **+0.1641e9** instructions on `core`, not a fall.
The coverage census explains why: 82.0% of the 10.01M fallback candidates are
in check, where the ladder can only test its trivial rung and defer to the
oracle, and it decides only 1.56M calls; the pre-registered projection is at
most **0.125e9**, below the 0.15e9 stop threshold. `M12` is the last measured
prover-side gain. `M1`'s performance part and `M6` close as a documented miss;
`M8` (complete shakmaty removal) is the only open item.

**Rationale.** A release that can only narrow a miss already bounded by the
audit, and is measured slower on the only available shape, costs more than it
buys. The goal remains complete prover replacement; the performance claim is
now a recorded residual rather than an open promise.

**Consequences.** No `wishlist3.md`, no pin or `PROVENANCE.md` change, no
ledger row. A future performance attempt needs a new plan and a measured lever
outside the rules layer, or a new crate shape that first passes the 0.31e9
whole-binary rule. The checker stays shakmaty and the differential remains the
correctness gate.

## MD8 — Evasion mask reopens the rules lever

**Resolution (2026-11-20).** `MD7` is reopened for one lever only: an
evasion-aware legality screen. The plan 9 prototype rejects a quiet
non-commoner move that cannot answer a single-slider check, and rejects
castling in check; all other candidates keep the existing oracle. On the bench
host it measures **−724,605,146** instructions on `core` (7,816,219,508 →
7,091,614,362, **9.271%**), −84,243,461 on `easy-win` (5.134%) and
−5,295,132,123 on `deep-win` (6.400%), against the pre-registered 0.31e9 rule.
Counters and mate are identical in every round on all three cells, the `.pvr`
is byte-identical, our checker and `provari check` accept all 9 proofs, and
the full differential reports `mismatches: 0`, `key_mismatches: 0`, EP 87/0/0.

The census behind it: of 8,220,516 in-check oracle calls on `core`, 6,128,196
are quiet non-king and **5,882,026 are rejected by the mask, none of them
legal**; `easy-win` rejects 761,648 of 1,038,152 and `deep-win` 45,072,205 of
61,347,947. On `deep-win` 21 castling/EP calls in check are legal, so en
passant stays with the oracle.

**Rationale.** `MD7` was decided on the only cheap lever then available, the
exposed `FastLegality` ladder, which costs +0.1641e9. The evasion mask attacks
the same 82% in-check population from the other side: it rejects what the
ladder could only defer, and the measured fall is 2.3× the 0.31e9 rule. It is
crate-side legality, so `MD5` forbids a prover patch; the lever is an upstream
ask ([`wishlist3.md`](wishlist3.md) `W10`), not an adoption.

**Consequences.** Adoption needs an owner-triggered release plan (plan 10):
implement `W10` upstream, then pin, `PROVENANCE.md`, differential, gates, veto
and a **same-session `MD6` row**. No pin, no `PROVENANCE.md` change, no ledger
row and no `src/` edit follow from this report. `MD7`'s text is unchanged; the
remaining rules residual and the other `MD7` buckets are unaffected.

## MD9 — `unsafe-optimizations` adopted on the audited pin

**Resolution (2026-11-20).** On `atomic-movegen` 2.11.0 the safe-Rust
alternative `sm` — masking the `ZOBRIST.piece` indices and the `ENTRIES[sq]`
loads with `& 63` / `& 15` — was built and measured against the default
(`base`) and the feature (`uo`). It is **not** a substitute: `sm` is slightly
slower than `base` on every cell (`core` +18,103,780 instructions, +0.253%;
`easy-win` +3,716,666, +0.238%; `deep-win` +190,478,555, +0.245%), so it
recovers −20.1% / −18.1% / −19.5% of `uo`'s fall against the 70% line. `uo`
falls 90,264,864 (1.263%), 20,497,022 (1.313%) and 974,758,363 (1.253%) on
the same three cells, with counters/mate identical, byte-identical `.pvr`,
both checkers 9/9 and differential 0/0 (EP 87/0/0). The pre-registered rule
therefore selects **adopt**, not `W11` and not close.

The audit closes the call sites. The three unsafe blocks are exactly the
census: `piece_key_raw`, `bishop_attacks` and `rook_attacks`. In 2.11.0 the
`BISHOP_ENTRIES[sq]` / `ROOK_ENTRIES[sq]` loads preceding the unsafe table
loads are **safe array indexing**, so `Square::NONE` still panics before the
unsafe access; the plan's panic→UB delta does not hold for the magic paths.
Only `piece_key_raw` could read out of bounds on `NONE`, and the prover never
sends it: moves decode `bits & 63`, bitboard iterations yield real squares,
`attackers_to` receives move endpoints, iterated targets, `Some(king)` or the
`maybe_ep_square()` value, and `checked_after` is unused. Our `src/moves.rs`
`attacks::bishop_attacks` / `rook_attacks` calls are **shakmaty's**, not the
crate's; the crate's magic is reached through `Board` internals.

**Guard.** Upstream `0aaf958` has no `.github` directory and no CI job.
`measurements/plan11/upstream-ci.patch` adds a workflow that tests the feature;
the enforceable guard in this repository is `make unsafe-feature`, a script
that runs the pinned crate's own test suite with
`--features unsafe-optimizations` from the registry copy Cargo builds, wired
into `make test-full`. Landing the upstream workflow is an owner action and
does not change the pin.

**Consequences.** `Cargo.toml` enables the feature on `=2.11.0`; the pin and
`PROVENANCE.md` row are unchanged except for the audit note. The same-session
`MD6` row reads 1.093 / 1.066 / 0.867 (geomean **1.003**) against the plan 2
binary's 1.002 / 1.015 / 0.957 (geomean 0.991), so candidate/old is
1.091 / 1.050 / 0.906, geomean **1.012**: `MD6` is still **not held**,
improved from 1.028. `core` is **7.0573e9**, **1.074×** plan 2, from 1.088×.
`M15` closes; the product win is 1.3% and a cleaner `M8` baseline, not parity.

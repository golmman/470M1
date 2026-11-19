# Plan 8 measurements — rules endgame: the legality ladder

**Date:** 2026-11-19. **Host:** `c90face26537` (x86_64, Linux 7.2.13, 4 vCPU).
**Pin:** `atomic-movegen` 2.10.0 (`5083647`), shakmaty 0.30.1. **Tree:**
`470MI@bb0d125` plus this plan's docs only; `src/`, `Cargo.toml`,
`Cargo.lock`, `PROVENANCE.md` and `check.rs` are unchanged.
**Outcome:** **acceptance**. The exposed `FastLegality` ladder is correct but
**slower** on `core` in every tested shape: the best variant costs
**+0.1641e9** instructions, not the pre-registered 0.20–0.60e9 fall. The
census projection is at most **0.125e9**, below the 0.15e9 stop threshold and
far below the 0.31e9 release threshold. No fourth release is requested.

## Binaries and scratch trees

| Side | Path | sha256 |
| --- | --- | --- |
| unchanged candidate | `target/release/470MI` | `b7c16c94…` |
| plan 2 reference | `target/release/470m1` | `71fff646…` |
| V1 prototypes | `/tmp/plan8-proto-tree/target/release/470MI` | not committed |

`/tmp/plan8-crate` is the scratch 2.10.0 crate with the diagnostic exposure
(`w10.patch`); `/tmp/plan8-census-crate` adds the stage trace; `/tmp/plan8-probe`
is the per-call screen; `/tmp/plan8-census-tree` and `/tmp/plan8-proto-tree`
are scratch copies of this repository. No binary, `perf.data` or transcript is
committed.

## Phase A — baseline

`core-perf.txt` starts with three interleaved `perf stat -e instructions:u`
runs, `prove FEN --seconds 10 --memory 192 --seed 0 --json`, no proof export.
The unchanged tree is **7.8162193e9** (spread 668) and plan 2 is
**6.5693771e9** (spread 323); the gap is **1.2468422e9**, the plan's fixed
input. Counters are `child_evals` 3,896,430, `expansions` 191,557, `nodes`
350,430, mate 14.

## Phase A — per-call screen

`probe/` is the ten-FEN, 2,000-pass harness (496,000 calls per operation; the
plan's 49,600 is the per-FEN mean, the plan-5 precedent reports the total).
`probe/results.json`:

| Operation | Instructions/call |
| --- | ---: |
| `V0`: `attackers_to` + `by_role` | **65.7** |
| `VB`: `LegalityContext::legal` (oracle) | **152.3** |
| `V1`: `FastLegality` ladder | **72.0** |
| `V3`: shipped `ChildEffect` | **213.8** |

`V1` is only **6.3** above `V0`, so the cheap-stop screen condition
`V1 > V0 + 10` did **not** fire; the whole-binary prototype was therefore
run. `V2` (a plan-2-occupancy legality entry point) was not built: `V0` is its
floor, so it could not undercut the ladder by more than that 6.3 margin. `V3`
is 148.1 above `V0`, and the shipped `ChildEffect` already is the
parent-bitboard shape; no further V3 API was available to test, so V3 was not
selected. The probe is a screen only; the decision uses whole-binary numbers.

## Phase A — coverage census

`census.json` is the `core` run (no proof export) of the scratch tree with
`Mover::legal`'s fallback wired to `FastLegality::legal_traced`. Counters and
mate are identical to the unchanged tree; a separate export run also produced
a byte-identical `.pvr`, so the ladder is oracle-equivalent by contract. The
counts below are the no-export run; proof export walks the winning line and
adds about 157k `Mover::legal` calls, so it is not the census denominator.

| Quantity | Count |
| --- | ---: |
| `Mover::legal` calls | 15,171,649 (matches plan 6's entry count) |
| prover's own quiet fast path | 5,159,465 |
| fallback candidates | **10,012,184** |
| — in check | **8,220,516** (82.0%) |
| — capture | 764,200 |
| — king | 915,488 |
| — quiet (pinned) | 111,980 |
| ladder trivial | 0 |
| widened `Some(true)` | 1,449,797 |
| widened `Some(false)` | 106,811 |
| widened `None` → oracle | 235,060 |
| in check → oracle | 8,220,516 |

The ladder decides **1,556,608** of 10.01M fallback calls (15.5%). In check,
82% of the fallback, the ladder can only test the trivial rung and defer. The
pre-registered projection `Σ count × (cost_oracle − cost_rung)`, using the
screen's ladder average as the rung cost, gives `1,556,608 × (152.3 − 72.0)` =
**0.125e9**; charging the failed widened attempts against it gives **0.108e9**,
and scaling the oracle to the whole-binary 120.9 gives **0.099e9**. These are
optimistic (the ladder average is measured on the full candidate stream, not
the fallback population) and all are below 0.15e9; Phase B is the decision
input.

## Phase B — whole-binary prototypes

`v1-hybrid.patch` is the prover wiring; `w10.patch` is the scratch crate
exposure. Three V1 shapes were measured on `core`, three interleaved pairs
each, all against the unchanged candidate. The first two use the ladder's
built-in oracle; the third keeps the prover's `LegalityContext` as the oracle
and calls only `decide_or_none`, which is the census's `None (oracle)` shape.

| V1 shape | Median instructions | Delta vs unchanged |
| --- | ---: | ---: |
| ladder, full `StateInfo` built-in oracle | 8,034,301,477 | **+218,082,159** |
| ladder, counts-only built-in oracle | 8,106,264,837 | **+290,045,519** |
| ladder `decide_or_none` + `LegalityContext` oracle | 7,980,337,054 | **+164,117,736** |

The best variant is a **+0.1641e9 regression**, not a fall. Every run has
identical counters and mate, and the `.pvr` is byte-identical
(`714723211c7a0246…` on `core`). The census projection was optimistic because
it charges the ladder only on decided calls; the whole binary also pays the
trivial-rung test and the failed widened attempt on the 8.46M oracle calls,
and it pays the larger `Mover` per node. `core-perf.txt` holds all pairs.

## Phase C — decision

Pre-registered rule: release only at a measured `core` fall **≥ 0.31e9** with
identical counters, mate and `.pvr`. The best measured fall is **−0.1641e9**,
so the branch is **acceptance**. The dated `MD7` row is in `decisions.md`; no
`wishlist3.md` was written. The tree is unchanged, so no pin, no ledger row
and no bench geomean is claimed. V3 was not added: the shipped `ChildEffect`
is already the parent-bitboard query, and a further node snapshot is the
prover-side `NodeFacts` shape plan 6 falsified at +0.038e9; it could not move
the decision inside this session.

## What this instrument cannot see

`perf stat` sees the same search and binary; it does not attribute the
+0.164e9 between the extra ladder work and the larger `Mover`. The census and
the three shapes together bound that: 82% of fallback calls are in check, so
the ladder's ceiling is small and its fixed cost is paid on every fallback
call. The bench cannot separate a rules-layer regression from guide ordering;
it would only repeat that the tree, counters and proof bytes did not move.

## Files

- `core-perf.txt` — baseline and every prototype `perf stat` pair.
- `probe/` — screen source, driver and `results.json`.
- `census.json` — stage/site counts and the projection.
- `results.json` — parsed baseline, screen and prototype numbers.
- `env.json`, `decision.json` — environment and branch record.
- `w10.patch`, `v1-hybrid.patch`, `census.patch`, `census-crate.patch` —
  diagnostic scratch patches, not requests.

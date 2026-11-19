# Plan 6 — the non-rules residual

**Item:** new `M11` (non-rules hot path: node geometry, generation adapter,
tree, `null`). **Decisions:** `MD1`, `MD5`, `MD6`; no new decision row.
**Session:** execution. **Predecessors:** [`report5.md`](report5.md),
[`hot-path-audit.md`](hot-path-audit.md). **Report:** `report6.md`.
**Budget:** one sitting (~6 h).

## Goal

Remove the largest measured *non-rules* site from the adopted 2.10.0 hot path,
with the same tree and proofs, and re-run the `MD6` gate: a formulation change
on the prover's node geometry, not a third crate micro-optimisation round and
not a search-tuning session.

## Fixed inputs (do not re-litigate)

Rules cost is off-limits: `W7` 121.9 against ≤100, `W8` 215.2 against ≤120, the
plan-5 rule stands (no third crate round, no `wishlist3`). `core` is 8.004e9
instructions against plan 2's 6.569e9; plan 5 attributed ~0.84e9 to rules and
~0.60e9 to the non-rules residual. Both checkers, `D14` and `.pvr` identity
are the only correctness contract; the owner funds the non-rules residual.

## Pre-measurement — read-only probe this session

`perf record -e instructions:u` of the unchanged binaries on `core` (seed 0,
10 s, no export): 8.001e9 against 6.565e9, report 5 to the digit, counters
matching (`child_evals` 3,896,430 / 3,823,720, mate 14 / 17). Exclusive symbol
share, grouped and normalised per node (tree +1.9% evals ≈ 0.12e9):

| Site group (symbols) | 2.10.0 | plan 2 | Per-node Δ |
| --- | ---: | ---: | ---: |
| rules path (`legal_in`, `child_effect`, blast, rights) | 25.6% = 2.05e9 | 1.36e9 | **+0.63e9** |
| node geometry (`exposes`, `plain`, `terms`, `gives_check`, `threats`, `Mover::new`, `Board::attackers_to`, `shakmaty::attacks`, …) | 29.4% = 2.35e9 | 1.86e9 | **+0.45e9** |
| generation adapter (`Position::pseudo_legal` + closure/push) | 8.9% = 0.71e9 | 0.50e9 | **+0.20e9** |
| tree (+1.90% `child_evals`, +1.03% expansions) | — | — | **+0.12e9** |
| `null` (`replay_threat` 0.69% vs 1.12%) | 0.055e9 | 0.074e9 | −0.02e9 |
| `play` + `__memmove` | 0.325e9 | 0.352e9 | −0.03e9 |
| search/table (`mid`, `children`, `evaluate`, store, filter, proofs) | 33.0% = 2.64e9 | 39.7% = 2.61e9 | −0.02e9 |

Sum 1.33e9 against 1.436e9; ~0.11e9 sits under the 0.15% cut and in inlining
attribution. **Provisional finding:** node geometry dominates, then the
generation adapter, then tree; `null` and `play` are *negative*. The plan-5
`0.84 / 0.60` split is not confirmed: by symbol rules are ~0.63e9, non-rules
~0.70e9. Input, not evidence, until Phase A reproduces it with call counts.

## Phase A — the decisive experiment: instruction split by call site

Two instruments, one budget, no committed source change:

1. **Counted build** (temporary, uncommitted). Relaxed `AtomicU64` counters
   plus a `--counts` JSON field at 12 sites: `Position::legal`,
   `effect_with_occupancy`, `pseudo_legal`, `play`, `null`, `Mover::new`,
   `plain`, `exposes`, `gives_check`, `threats`, `eval::terms`,
   `replay_threat`. Counts only — timing must not be read from it. The
   counters (`child_evals`, `expansions`, `nodes`, mate) must be identical with
   and without it: the instrument must not move the search.
2. **`perf record -e instructions:u -c 500000`** per binary (≥ 8k samples;
   the default 2k is too coarse), `--sort symbol`, then the bucket map above
   applied mechanically to every symbol.

Deliverable: `split.json` — per site `calls`, `instructions`, `per_call`,
`share`, the bucket roll-up for both binaries, and a reconciliation: Σ site
budgets within ±3% of `perf stat` (8.004e9), the plan-2 replay within ±5% of
6.569e9. If it misses, report the split unresolved; stop before Phase B.

## Phase B — one formulation change: `NodeFacts`

**Decision rule (pre-registered).** Adopt `NodeFacts` only if Phase A puts the
node-geometry bucket at ≥ 0.30e9 **and** ≥ 50% of the non-rules residual. If
generation dominates, stop and report: that is a crate API shape
(`for_each_*` closure + caller buffer), a wishlist item this plan forbids. No
substitute change is made.

**C1 — the node's geometry becomes a per-node value.** Today `Position` is the
geometry source: `exposes`, `threats`, `gives_check`, `blasted_cheaply`,
`might_end_material`, `plain` and `eval::terms` re-enter it (and the crate's
`PositionView`) per candidate and per piece — ~21 accessor sites in
`src/moves.rs` plus `role_at` per piece in `eval::terms`. Build one
`NodeFacts` (occupied, both colour masks, both king bitboards, six by-role
masks) per expanded node from the crate's public bitboards, pass it by
reference to `Mover` and `eval::terms`, and let the candidate queries read it.
Legality and effect stay direct crate calls (`LegalityContext::legal`,
`child_effect_with_occupancy`): no `StateInfo`, no `Board` copy, no second
legality, no conversion — `MD5` holds and the audit's negative
`StateInfo`-in-`Position` result is not reintroduced (facts are built once per
expanded node, never carried into children). It is a formulation change, not a
micro-optimisation: it moves where the node's rules geometry is owned, touching
~29% of instructions rather than ~2%.

**Expected gain:** 0.30e9 ± 0.15e9 (≈3.7% of instructions, ≈2–3% wall),
geomean **1.094 → ~1.06** — still above the `MD6` bar. C1 narrows the
residual; it does not close it.

**Cheapest falsifier:** build C1, run `perf stat -e instructions` on `core`
(seed 0, 10 s, no export) twice, interleaved with the unmodified binary. If
instructions fall by < 0.15e9, or `child_evals` / `expansions` / `nodes` / mate
move on any cell, C1 is dropped in-session (working-tree revert, reported as a
negative result — not a decision-row revert) and the session ends at the split.
Cost ~15 minutes, before any gate or bench.

## Phase C — gates and the `MD6` row

Untouched unless Phase A names an ordering change (which would need its own
justification as a formulation change): rules layer, `.pvr` format and order,
`check.rs`, search/eval/threat ordering, seeds, counters. C1 computes the same
values, so counters, mate and the nine fixtures must be identical at seed 0.
Gates: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`,
`cargo test`, `cargo doc --no-deps`, `make test`, `make test-full`, `make veto`,
`cargo test --release --test differential rules_differential_full` (plan 5's
counts: 84/11, 27,088 positions, 540,383 moves/keys, 0/0, `MD3` 87/0/0).

**Gate `MD6`:** same-session plan 2 binary (`470m1`, rebuilt from `2f629fd`),
plan 3's driver, protocol `a`, seed 0, 10 s, 192 MB, 3 repeats: **3/3 cells,
9/9 accepted by both checkers, `D14` identical, geomean ≤ the same-session
plan 2 factor** with that reference near its 0.972 anchor. Beating `1.0`
against provari while losing to the reference is not a pass.

## Stop rules

- No tuning to hide a rules cost: no search, eval, threat or order change to
  recover wall time; that would be its own formulation change.
- No proof movement: a `.pvr`, mate or counter change caused by C1 is a stop.
- A non-EP rules mismatch or a checker disagreement is a veto.
- Reverting the adoption needs a new dated decision row; none is written here.
- Stop if the split misses reconciliation, C1 misses its falsifier, or C1
  needs a crate API — a wishlist item, and no `wishlist3` is opened.
- No second formulation change in the same session.

## Artefacts, ledger and budget

`measurements/plan6/`: `README.md`; `split/` (bucket map, `counts.patch`, the
uncommitted instrument diff, `run_split.py`, `split.json`);
`bench/{results,old,env}.json`; `core-perf.txt`. No raw transcripts,
`perf.data`, oracle artifacts, `__pycache__` or binaries. Exactly one
`movegen plan6` row in [`../../bench.md`](../../bench.md) **when the tree
changes**; otherwise the report records the split and no row (plan-4
precedent). The execution session adds `M11`, its status and the history line
to [`initiative.md`](initiative.md).

Budget: Phase A 1.5 h, C1 2 h, Phase C 1 h, bench 1 h, docs 0.5 h. Deliver
`report6.md` with the ranked budget and reconciliation, the C1 verdict (kept /
falsified / not justified), the `MD6` table and the distance to the goal.

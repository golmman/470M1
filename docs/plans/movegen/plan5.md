# Plan 5 — the `M1` decision: does the adopted hot path hold the bar?

**Items:** `M1` (adapter hot path), `M2`, `M6`; decisions `MD1`, `MD5`, `MD6`.
**Session:** execution (measurement and decision). **Predecessor:**
[`plan4.md`](plan4.md) (adopt the next `atomic-movegen` release) and
[`report3.md`](report3.md); see [`hot-path-audit.md`](hot-path-audit.md).
**Report:** `report5.md`.

## Goal

Decide, on measured data, whether the `atomic-movegen` hot path holds
"no regression" against the plan 2 prover. The rollback to the provari-derived
path is **rejected** (owner, 2026-11-19): complete replacement stays the
definition of done (`MD1`), the prover keeps no translator on the rules path
(`MD5`), and the bar is the plan 2 reference binary measured in the same
session (`MD6`). This plan measures and records; it does not tune the solver.

**Not here:** no search/eval/threats change, no `.pvr` change, no checker
change, no path dependency, no second upstream release in this plan.

## Bar

Run plan 3's driver, one session, protocol `a`, 10 s, 192 MB, 3 repeats, seed
0, with the plan 2 binary (`target/release/470m1`, never committed) as the
same-session reference. Pass requires **3/3 cells, 9/9 accepted by both
checkers, and geomean ≤ the same-session plan 2 factor**, with that reference
reading within noise of its recorded **0.972**. A candidate that beats `1.0`
against provari but loses to the same-session reference does **not** pass.

## Diagnosis to carry forward

Plan 3 measured a per-node regression, not a tree regression (tree +2.4%,
instructions +44%, same-session old binary 0.987). The audit turns that into a
call budget: the rules layer is ~31% of all instructions and the
movegen-minus-plan2 part of it is ~1.50e9 of the 2.46e9 gap, split roughly
legality 1.21e9 · effect+key 0.56e9 · pseudo+conversion 0.79e9 · state 0.05e9
· play 0.18e9. The rest is the larger tree, `Mover`'s accessor traffic,
`null`, and residual adapter code.

Three designs were measured and rejected, so they are not candidates:

| Variant | `core` instructions | Wall | vs HEAD |
| --- | ---: | ---: | ---: |
| plan 3 HEAD | 9.026e9 | 639 ms | — |
| `EffectContext` for effect + key | 10.637e9 | 729 ms | +17.9% |
| node `StateInfo` cached in `Position` | 10.521e9 | 728 ms | +16.6% |
| both, plus `EffectContext::is_legal` | 12.794e9 | 908 ms | +41.7% |
| earlier bundle (`/tmp/probe4`) | 8.527e9 | 608 ms | −5.5% |
| plan 2 binary (the bar) | **6.569e9** | **494 ms** | −27.2% |

The optimistic ceiling for a crate-led fix is ~7.35e9 (≈1.12× plan 2), so the
expected outcome of `plan4` is a **finding**, not necessarily a pass. That
expectation is recorded before the measurement.

## Phases

### A — same-session reference and candidate (~1 h)

Rebuild plan 2's binary from `2f629fd` into `target/release/470m1` (never
committed). Build the adopted candidate. Run both through plan 3's driver on the
same host, interleaved, protocol `a`, three repeats, seed 0; also record
`perf stat` instructions for `core` on both. Two identical runs of each.

### B — decision table (~30 min)

Classify the row:

1. **Pass** — 3/3, 9/9, geomean ≤ same-session reference. Record it, close
   `M1`'s performance part, proceed to `M8`.
2. **Near miss, rules-caused** — the residual is in a named wishlist item. File
   the measured item in `wishlist3.md`; the next release plan targets it. Do
   not tune search/eval.
3. **Near miss, non-rules** — name the cause (tree size, `Mover` traffic,
   `null`, table). `M6`'s no-regression gate stays open; the next plan is a
   measurement plan for that cause, not a rules change.
4. **Regression** — stop and report; the adoption is reverted only by a new
   dated decision row, never silently.

### C — residual write-up (~30 min)

Per-call checks against the wishlist's acceptance numbers (legality ≤100,
effect+key ≤120, no conversion, counts-only state), the residual instruction
count by cause, and the tree comparison. This is what `report5.md` reports.

### D — docs (~30 min)

`measurements/plan5/` (driver, `results.json`, `old.json`, `env.json`,
README; no raw transcripts), one ledger row in [`../../bench.md`](../../bench.md),
`report5.md`, and the one-line status in `initiative.md`.

## Risks and stop rules

- **A cross-session comparison is not a factor.** Only same-session ratios
  decide; the recorded 0.972 is an anchor, not a target.
- **No tuning to hide a rules cost.** A regression is a finding. Changing
  search, evaluation or move order to recover wall time is out of scope and
  would be reported as a scope change.
- **No proof movement.** A `.pvr` change caused by a non-key, non-order cause
  is a stop.
- **Stop** on a non-EP rules mismatch, a checker disagreement, a lost cell, or
  a candidate slower than the same-session reference on two of three cells.
- **No third optimisation round.** Two measured bundles are enough; the next
  move is a formulation decision, not another micro-optimisation.

## Cost and deliverables

Measurement 1.5 h, decision 0.5 h, write-up 0.5 h, docs 0.5 h. Deliver:
`measurements/plan5/`, one ledger row, `report5.md`, the decision (pass / near
miss with cause / regression) and, if a pass, the handoff to `M8`. The decisive
experiment is the same-session `core` comparison; the stored plan 2 binary
bounds it.

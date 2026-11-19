# Plan 7 — eval geometry: one per-node pass instead of one per piece

**Item:** new `M12` (eval geometry formulation; the last prover-side lever named
by `report6`). **Decisions:** `MD1`, `MD5`, `MD6`; no new decision row.
**Session:** execution. **Predecessors:**
[`report6.md`](report6.md), [`report5.md`](report5.md),
[`measurements/plan6/`](measurements/plan6/README.md),
[`hot-path-audit.md`](hot-path-audit.md). **Report:** `report7.md`.
**Budget:** one sitting (~5 h): A 1 h, B 1.5 h, C 1 h, D 1.5 h.

## Goal

Move `eval::terms` from a per-piece board walk to a per-role pass that reuses
the node's already-computed check masks, with identical scores, tree, counters
and `.pvr`. This is `report6` option 1 and the only measured prover-owned lever
left after `C1` was falsified. Expected **0.05–0.10e9**; it narrows the gap, it
does **not** hold `MD6`.

## Fixed inputs (do not re-litigate)

- Rules stay off-limits: no fourth release, no `wishlist3`, no crate patch, no
  legality/effect change. `MD1`'s adoption stands.
- `core` is 8.004e9 instructions against plan 2's 6.569e9; `MD6` needs ~0.78e9.
  This plan's ceiling is the absolute cost of the eval geometry: `eval::terms`
  **355.0M** (926.6 per call, 383,114 calls) plus eval's share of
  `shakmaty::attacks::attacks` **533.5M** (shared with `moves.rs`).
- Two corrections the plan session established, to keep the estimate honest:
  - `eval::terms` is in the **geometry** bucket; its *delta* is only **+46.5M**
    (355.0 vs 308.5M). The `+0.1045e9` in `report6` is `Dfpn::evaluate`
    (420.5 vs 316.0M), a search symbol that never calls `eval`. "Composes with
    the `evaluate` growth" is a conflation; the prize here is beating plan 2's
    absolute eval cost, not closing an eval delta.
  - `shakmaty::attacks::attacks` is called from six sites, not only `eval`. The
    measured share, not the whole symbol, is the budget.
- Identical scores are the contract. Any change in a `Terms` value, in the tree
  or in a proof is a stop, not a tuning opportunity.

## Phase A — pre-registered budget, read-only (~1 h)

1. **Session baseline.** Rebuild the unchanged tree; three interleaved
   `perf stat -e instructions:u` runs of the candidate and the plan 2 binary on
   `core` (seed 0, 10 s, 192 MB, no export). Record the pair and both spreads.
2. **Counted build** (temporary, uncommitted; plan-6 `split/counts.patch`
   precedent) counting, per `terms` call: pieces per role per side (`N`, `R`)
   and the `attacks::attacks` calls made inside `eval`. Counters must be
   unchanged (`child_evals` 3,896,430, `expansions` 191,557, `nodes` 350,430,
   mate 14). This fixes the expected call reduction at **`N − R` per `terms`
   call** — one mask call per role present replaces one per piece, minus the
   masks `Mover` already holds — and pre-registers the expected fall as
   `(N − R) × per-call cost` with `N`, `R` read from the run, not assumed.
3. **Boundary check, 20-minute cap.** `perf annotate` of `Dfpn::evaluate` in
   both binaries: its +25 per child on an unchanged tree may be crate-inlined
   cost of the same kind as the `attackers_to` inlining boundary. Recorded for
   the next decision only; it does not change this plan.

**Gate:** if the counted build shows `N − R ≤ 1` on the median `terms` call,
stop and report that the mask hoist is not worth a plan (the fork is then
`report6` option 2, not option 3).

## Phase B — `M12`, the change (~1.5 h)

`evaluate(pos, mover)` replaces `terms(pos, us)` + `terms(pos, !us)`:

1. read `occupied`, both colour masks, and the `Mover`'s `check_squares` — it
   is already one mask per role (`src/moves.rs:89-93`) and already in scope at
   the `step` call (`src/dfpn.rs:676`, `:717`);
2. compute the defender's per-role table **once** (≤5 `attacks::attacks`
   calls), only when the defender's king exists and the kings do not touch;
3. iterate each side's pieces **by role** over `by_piece`, so the role is known:
   material is `popcount × VALUES[role]`, the check mask is indexed by role, and
   `role_at`/`piece_at` leave the loop;
4. keep the early returns exactly: no king → material only; touching kings →
   material only.

The old formulation stays as `#[cfg(test)] terms_reference`; a unit test asserts
`Terms` (then `evaluate`) equality on every differential-corpus position and the
seeded play-outs. Identity is structural — the only order-dependent operations
are union, count and sum — so the test is the witness, not the search.

**Expected:** 0.07e9 ± 0.04e9 (≈0.9% instructions, ≈0.6% wall), geomean
**1.094 → ~1.085**. The measured number, not a pass, is the deliverable.

## Phase C — falsifier (~1 h)

Pre-registered: adopt only if three interleaved `perf stat -e instructions:u`
pairs show a negative delta outside both spreads **and ≥ 0.03e9**. Below that,
or on any counter, mate or `.pvr` movement, revert in the working tree and
report the negative (plan-6 `C1` precedent: no decision row, no bench row).

Why 0.03e9 and not `report6`'s 0.05e9: that cliff sits at the bottom of the
plausible range, and `C1` cost +0.038e9 with identical counters — the instrument
resolves 0.03e9, so a real 0.04e9 win must not be dropped for being small.

## Phase D — gates and the `MD6` row (~1.5 h)

Rules layer, `.pvr` format and order, `check.rs`, eval **values**, threat
ordering, seeds untouched. Gates: `cargo fmt --check`,
`cargo clippy --all-targets -D warnings`, `cargo test`, `cargo doc --no-deps`,
`make test`, `make test-full`, `make veto`,
`cargo test --release --test differential rules_differential_full -- --ignored`
(plan 5's counts: 84/11, 27,088 positions, 540,383 moves/keys, 0/0, `MD3`
87/0/0).

**Gate `MD6`:** same-session plan 2 binary (`470m1`, rebuilt from `2f629fd`),
plan 3's driver, protocol `a`, seed 0, 10 s, 192 MB, 3 repeats: **3/3 cells,
9/9 accepted by both checkers, `D14` identical, geomean ≤ the same-session
reference** with that reference near its 0.972 anchor.

## Stop rules

- A different `Terms`/`evaluate` value is a stop — score tuning is a different
  initiative.
- `child_evals`, `expansions`, `nodes`, mate or `.pvr` movement at seed 0 is a
  stop.
- No rules, format, checker or search change; no crate API; no `wishlist3`.
- One formulation change this session. If `M12` is falsified, the session ends
  at the measurement and the report states the fork: `report6` option 2 (dated
  acceptance) or plan 8 (the plan-2-shape crate prototype, gated at ≥0.4e9).
- A checker disagreement or a non-EP rules mismatch is a veto.

## Artefacts, ledger and budget

`measurements/plan7/`: `README.md`; `counts.patch` and the counts JSON;
`bench/{results,old,env}.json`; `core-perf.txt`; the `perf annotate` excerpts.
No raw transcripts, `perf.data`, oracle artifacts or binaries. Exactly one
`movegen plan7` row in [`../../bench.md`](../../bench.md) **when the tree
changes**; otherwise no row. The execution session adds `M12`, its status and
the history line to [`initiative.md`](initiative.md), and writes the
pre-registered vs measured call/instruction table into `report7.md`.

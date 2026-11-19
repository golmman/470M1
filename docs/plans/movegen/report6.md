# Report 6 — the non-rules residual

**Plan:** [`plan6.md`](plan6.md). **Item:** `M11` (new). **Decisions:**
`MD1`, `MD5`, `MD6`; no new decision row. **Date:** 2026-11-19.
**Session:** execution, measurement, negative result. **Tree:**
`470MI@1aba046`, `atomic-movegen` 2.10.0 (`5083647`), unchanged at the end.
**Outcome:** the split is resolved — rules **+0.62e9**, node geometry
**+0.36e9**, generation **+0.13e9**, tree **+0.12e9** of the 1.435e9 `core`
gap. `NodeFacts` (`C1`) passed the pre-registered adoption rule and then
failed its falsifier at **+0.038e9** instructions with identical counters, so
it was dropped and reverted. No tree change, no bench, no ledger row.
Measurements: [`measurements/plan6/`](measurements/plan6/README.md).

## Phase A — the split, resolved

Two instruments, no committed source change. The **counted build**
(`measurements/plan6/split/counts.patch`, built in a scratch copy) counts the
12 pre-registered sites plus three supplementary entry points with relaxed
`AtomicU64`s and prints them as a `--counts` field. On `core` (seed 0, 10 s,
no export) it reports the same search as the product binary —
`child_evals` 3,896,430, `expansions` 191,557, `nodes` 350,430, mate 14 — so
the instrument does not move the search. The **profile** is
`perf record -e instructions:u -c 500000` per binary, `--sort symbol
--show-total-period`, bucketed by `split/bucket_map.py`.

Reconciliation: symbol totals are 7.9965e9 and 6.5640e9, i.e. **−0.09%** and
**−0.08%** of the session's `perf stat` (8.0041e9 / 6.5694e9) — far inside the
plan's ±3% / ±5%. Call counts and per-call costs:

| Site | Calls | Instructions | Per call |
| --- | ---: | ---: | ---: |
| legality (`context_legal`, rules) | 10,012,184 | 1.2100e9 | **120.9** |
| effect (`child_effect` + blast + rights) | 4,779,358 | 0.6325e9 | **132.3** |
| `pseudo_legal` (generation) | 578,396 | 0.6620e9 | **1144.5** |
| `Mover::new` (+ `populate_state`) | 578,396 | 0.3230e9 | **558.4** |
| `terms` | 383,114 | 0.3550e9 | **926.6** |
| `exposes` | 4,745,979 | 0.3205e9 | 67.5 |
| `plain` | 6,630,358 | 0.2820e9 | 42.5 |
| `gives_check` | 3,358,938 | 0.1645e9 | 49.0 |
| `threats` | 987,717 | 0.0595e9 | 60.2 |
| `play` | 949,651 | 0.1585e9 | 166.9 |
| `replay_threat` (`null`) | 44,734 | 0.0655e9 | — |
| `attackers_to` | 2,933,157 | 0.2120e9 | 72.3 |

**The ledger of the gap.** Total 1.4347e9; tree volume (+1.9% `child_evals`)
0.1249e9; rules **+0.6170e9**; **non-rules residual 0.6928e9**, split as
geometry **+0.3585e9**, generation +0.1255e9, search/table +0.2505e9
(`evaluate` +0.1045, `prefetch_child` +0.0375, `read_filter` +0.0265,
`store` +0.0125, proofs +0.0115), play + memmove +0.0575e9, `null` +0.004e9.
Σ bucket deltas = 1.4325e9 against 1.4347e9 (−0.15%).

Two findings the plan's provisional table could not see:

- **Plan 5's legality upper bound is now a call count.** 10.01M
  `context_legal` calls at **120.9** each — plan 5's probe said 121.9 — so
  `W7` is confirmed from a second instrument and the rules delta is 0.617e9,
  not the 0.84e9 that the uncounted budget allowed. The effect path is 132.3
  per call here (98.1 for the crate call plus the `blast_casualties` and
  `castling_rights_after` internals) against the probe's 215.2; the probe
  measures a different call shape, and the binary figure is the one the gap is
  made of.
- **A third of the geometry delta is an inlining boundary.**
  `Board::attackers_to` is its own 0.2120e9 symbol only in 2.10.0; in plan 2
  its counterpart sits inside `checked_after`/`king_attacked`, which are rules
  symbols. `split.json` carries both readings: **primary** (the plan's label)
  geometry +0.3585e9 = 51.7% of the non-rules residual, rule open;
  **aligned** geometry +0.1465e9 = 30.5%, rule shut. `evaluate` and the
  per-call shapes of `plain`/`exposes`/`gives_check` show no accessor-driven
  growth at all, which is what `C1` was betting on.

## Phase B — `C1` `NodeFacts`, falsified

The plan's rule opened on the primary reading, so `C1` was built as specified:
one `NodeFacts` (occupied, two colour masks, two kings, six role masks) per
expanded node from `Board`'s public bitboards, passed by reference to `Mover`
and `eval::terms`, with the candidate queries reading it; legality and effect
stay direct crate calls, so `MD5` holds and no `StateInfo` or board copy is
introduced. It compiled clean, kept every counter identical and cost
**+0.0382e9** instructions (three interleaved runs each side,
`measurements/plan6/core-perf.txt`: 8.0423e9 vs 8.0041e9, spreads 275 and 429
instructions).

The reason is the one the split already implied: the accessors `C1` replaces
are `#[inline(always)]` field reads, while building an 88-byte value at each of
578,396 `Mover::new` calls is a real cost. **C1 is dropped** — a
working-tree revert, a negative result, not a decision-row revert.
`target/release/470MI` is bit-identical to plan 5's (`14093d41…`), so plan 5's
wall row still stands for this tree. The attempt is kept as
`measurements/plan6/c1/attempt.patch`.

Phase C's gates and the `MD6` row were therefore not run: there is no changed
candidate to gate or bench, and the plan says a tree that does not change gets
no ledger row (plan 4 precedent). The gates were re-run on the reverted tree
anyway, all green: `cargo fmt --check`, `cargo clippy --all-targets -D
warnings`, `cargo test` (48 unit + 12 integration, 2 ignored slow), `cargo doc --no-deps`,
`make licence`, `make test-full` (slow cells and veto), `make veto` (all
fixtures accepted, all controls rejected) and
`cargo test --release --test differential rules_differential_full --
--ignored`, which reproduces plan 5's counts exactly: 84/11 FEN, 27,088
positions, 540,383 legal moves and child keys, 72,131 explosions, 642
material exemptions, 392 games / 26,612 plies, **0** mismatches, **0** key
mismatches, `MD3` 87/0/0.

## Distance to the goal

Unchanged from plan 5: **8.004e9 vs 6.569e9 instructions (1.218×)** on
`core`, wall geomean **1.094** against the same-session plan 2 binary's
**0.997**; `MD6` is not held. But the residual is now a budget, not a fog:

- **0.62e9 (43%) rules**, owned upstream: legality 120.9 per call against the
  ≤100 target (`W7`), effect 132.3 against ≤120 (`W8`), plus the state and
  blast/rights internals. A third crate round is forbidden by the plan.
- **0.25e9 search/table**, of which `evaluate` is +0.1045e9: +25 instructions
  per child evaluation on the same tree. This is prover code, not crate code.
- **0.15e9 geometry** once `attackers_to` is not counted twice: `terms` 926.6
  per expanded node, built from two `attacks::attacks` per piece, and
  `Mover::new` 558.4.
- **0.13e9 generation**, whose fix is a crate API shape (`for_each_*` with a
  caller buffer) — a wishlist item, which this plan forbids opening.
- 0.06e9 play/memmove, 0.12e9 tree volume.

## Problems, scope changes and what was not done

- No scope change. No rules, format, checker, search, evaluation, threat or
  ordering change was kept; `C1` is the only change attempted and it is
  reverted.
- The plan's Phase A wording asks for a "per site `instructions`" figure for
  all 12 sites. `Mover::legal` and `Position::null` do not survive as symbols
  in 2.10.0 (inlined into `mid` and `replay_threat`), so their cost is read at
  the entry point that remains (`context_legal`, `replay_threat`) and the
  supplementary counters are marked as such in `split.json`.
- The `perf` bucket map moves `attackers_to` between rules and geometry
  depending on which binary's inlining you respect. The split reports both;
  `C1` was decided by measurement, not by the map.

## Next steps and options

1. **Formulation change on evaluation (recommended).** `eval::terms` asks
   `attacks::attacks(their_king, role.of(!color), occupied)` once per piece,
   though the answer depends only on the role: six values per `terms` call
   instead of one per piece. Estimated **0.10e9 ± 0.05e9** (1.2% instructions, ~0.8% wall), and it
   composes with the `evaluate` growth. Cheapest decisive experiment: build
   it, two interleaved `perf stat -e instructions` runs on `core`; drop it
   below −0.05e9. It is a new item with its own plan, not this session's.
2. **Accept the regression as a dated row.** Write the `MD1`/`MD6` acceptance
   with the split as evidence that no non-rules change of the pre-registered
   shape closes a 1.218× gap.
3. **A rule-layer change is still the only 0.62e9 lever.** That means a fourth
   `atomic-movegen` release with a legality/effect shape at plan 2's cost; the
   audit's ceiling says even that lands near 1.1×, so it does not reach the bar
   alone either.

Option 1's kickoff prompt:

> Read `docs/plans/movegen/initiative.md`, `report5.md`, `report6.md` and
> `measurements/plan6/`. Plan 7: change the formulation of `eval::terms` so
> the check-square mask is computed once per role per node instead of once per
> piece, keeping the tree, counters, `.pvr` and search order identical. Falsify
> with two interleaved `perf stat -e instructions:u` runs on `core` (seed 0,
> 10 s, no export); drop the change below a 0.05e9 fall. If it holds, run the
> gates and the `MD6` bench row.

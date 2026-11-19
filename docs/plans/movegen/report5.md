# Report 5 — the `M1` decision

**Plan:** [`plan5.md`](plan5.md). **Items:** `M1`, `M2`, `M6`; decisions `MD1`,
`MD5`, `MD6`. **Date:** 2026-11-19. **Session:** execution, measurement and
decision. **Tree:** `470MI@a28fb33`, `atomic-movegen` 2.10.0 (`5083647`).
**Outcome:** 3/3 cells, 9/9 accepted by both checkers, `D14` identical.
Candidate geomean **1.094**; same-session plan 2 binary **0.997**;
candidate/old geomean **1.107**. The `MD6` bar is **not held**. This is
classification **4 (regression against the bar)**, and its wall row sits at the
pre-recorded crate-led ceiling (~1.12×), so it is the expected finding. The
adoption stays; no source, search, evaluation, proof or checker change.
Measurements: [`measurements/plan5/`](measurements/plan5/README.md).

## Phase A — same-session reference and candidate

The reference was rebuilt, not trusted: `git archive 2f629fd` into
`/tmp/movegen-plan2`, `cargo build --release`. The binary is **bit-identical**
(`cmp`) to the stored `target/release/470m1` (sha256 `71fff646…`), so the
stored plan 2 binary is the `2f629fd` tree. Candidate `target/release/470MI`
(sha256 `14093d41…`) is the unchanged plan-4 tree at HEAD `a28fb33`.

Both sides ran through plan 3's driver, copied to
`measurements/plan5/run_bench.py`, interleaved, protocol `a`, seed 0, 10 s,
192 MB, three repeats, `--d14`. The old-binary pass ran immediately after with
the same driver and host.

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.525 s | 0.634 s | **1.208** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.133 s | 0.154 s | **1.158** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.587 s | 6.165 s | **0.936** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.094** | | | **9/9** |

| Cell | Old reference | Old candidate | Old factor | Old mate / ref | Old `child_evals` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.518 s | 0.525 s | **1.014** | 17 / 17 | 3,823,720 |
| `easy-win` | 0.132 s | 0.134 s | **1.015** | 15 / 15 | 897,937 |
| `deep-win` | 6.549 s | 6.303 s | **0.962** | 19 / 22 | 54,633,842 |
| **Geomean** | | | **0.997** | | | |

Candidate/old is **1.208 / 1.149 / 0.978**, geomean **1.107**. The plan 2
reference reads 0.997 against its 0.972 anchor (+2.6%), at the top of the
0.972–0.997 band seen in earlier sessions. `MD6` needs the reference near
0.972 and the candidate at or below it; the candidate is 9.4% above 1.0 and
9.7% above the reference on the factor geomean (10.7% above the old binary on
median walls), so no reading of the anchor rescues a pass.

Two identical `perf stat` runs of each binary on `core` (10 s, seed 0, no proof
export) agree on every counter:

| Binary | Instructions | Wall | Mate | `child_evals` |
| --- | ---: | ---: | ---: | ---: |
| plan 2 `470m1` | **6.569e9** | 494 ms | 17 | 3,823,720 |
| 2.10.0 `470MI` | **8.004e9** | 568 ms | 14 | 3,896,430 |

The instruction gap is **1.435e9 (1.218×)**; per `child_evals` it is 1718 vs
2054 (+19.6%). Plan 4's `core` numbers reproduce to the digit, as they should
for the unchanged tree.

## Phase B — decision

| Criterion | Bar | Measured | Verdict |
| --- | --- | --- | --- |
| Coverage | 3/3 | 3/3 | pass |
| Checkers | 9/9 accepted by both | 9/9, `D14` identical, old also 9/9 | pass |
| Reference | within noise of 0.972 | 0.997 (+2.6%) | accepted as noise, does not matter |
| Geomean | ≤ same-session reference | 1.094 vs 0.997 | **fail** |
| Cells | not slower on 2 of 3 | slower on `core` and `easy-win` | **fail** |

**Classification: regression against the `MD6` bar.** It is not a near miss in
the plan's sense: the loss is 9.7–10.7% against the same-session reference,
five times the ±2% session noise, and the candidate is slower on two of three
cells. The pre-recorded crate-led ceiling was ~7.35e9 instructions (~1.12×
plan 2). The measured 8.004e9 is 9% above that instruction ceiling, while the
wall result is at it — candidate/old geomean 1.107, bare `core` 1.150 — because
the 2.10.0 binary retires instructions slightly faster (14.1e9/s vs 13.3e9/s).
Either way the result is the audit's expected finding, and a third round of the
same per-call shape cannot be expected to reach the bar. Per `MD1` and the
owner's 2026-11-19 rollback rejection, the adoption **stays**; reverting would
need a new dated decision row, which this plan does not write.

There is no `M8` handoff: the plan makes a pass the precondition, and the pass
did not happen. `M1`'s performance part and the `M6` no-regression gate stay
open.

## Phase C — residual by cause

Per-call probe, median of three `perf stat` runs on the ten-FEN harness
(`measurements/plan5/probe/`), reproducing plan 4 exactly:

| Operation | 2.10.0 | Target | Verdict |
| --- | ---: | ---: | --- |
| `LegalityContext::legal` (`W7`) | **121.9** | ≤100 | miss |
| `child_effect_with_occupancy(Legal)` (`W8`) | **215.2** | ≤120 | miss |
| `Board::legal` + full state (host anchor) | 168.7 | — | host ~1.17× audit |
| `counts` / `populate_counts` (`W6`) | 49.9 / 68.3 | counts-only | used |
| `null_for_advanced` (`W9`) | 35.9 | no rebuild | pass |
| native `Move` round-trip | 18.6 | 0 on hot path | pass |

Normalised to the audit's host, `W7` is near 104 and `W8` near 184. `W6` is
counts-only on the per-`Position::legal` path, but `Mover::new` still calls the
full `populate_state` (189.7 here) because `tested` reads `checkers`/`pinned`.

Call budget for `core`, with the audit's call counts per run (8.4M legality,
2.6M effect, 0.36M pseudo, 0.33M state, 0.77M play, ~0.4M `in_check`) and this
session's per-call costs:

| Call | 2.10.0 | Total | Plan 2 shape | Total | Delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| legality | 121.9 | 1.02e9 | 83 | 0.70e9 | +0.33e9 |
| effect + key | 215.2 | 0.56e9 | 60 | 0.16e9 | +0.40e9 |
| pseudo generation | 833 | 0.30e9 | 964 | 0.35e9 | −0.05e9 |
| `populate_state` | 189.7 | 0.06e9 | 0 | 0 | +0.06e9 |
| `play` | 230 | 0.18e9 | 158 | 0.12e9 | +0.06e9 |
| `in_check` | 108 | 0.04e9 | 0 | 0 | +0.04e9 |
| **Rules total** | | **2.16e9** | | **1.33e9** | **+0.84e9** |

So **0.84e9 of the 1.435e9 gap (59%) is rules cost**, dominated by `W7`
(+0.33e9) and `W8` (+0.40e9); the native move storage already gained back
pseudo-generation and conversion. The remaining **0.60e9** is non-rules: the
slightly larger `core` tree (+1.9% `child_evals`, ~0.12e9), `Mover`'s
accessor/geometry traffic through the crate board, `null`, and residual adapter
code. The legality figure is an upper bound: `Mover::legal` returns before the
context for quiet untested moves, and this session has no per-call-site
counter. That missing instrument is named here so the next agenda does not
treat 0.84e9 as exact.

Tree comparison (candidate vs same-session old):

| Cell | Mate | `child_evals` | Evals ratio | Wall ratio | Per-node ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 14 / 17 | 3,896,430 / 3,823,720 | 1.019 | 1.208 | 1.186 |
| `easy-win` | 18 / 15 | 904,368 / 897,937 | 1.007 | 1.149 | 1.141 |
| `deep-win` | 20 / 19 | 44,828,946 / 54,633,842 | 0.821 | 0.978 | 1.191 |

The per-node ratio is ~1.14–1.19 on all three cells; the per-cell wall outcome
is the tree, not a cell-specific cost. `deep-win` wins only because its tree is
18% smaller; `core` and `easy-win` lose because their trees are 0.7–1.9%
bigger on top of the same per-node penalty.

## Phase D — gates and docs

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | green |
| `cargo doc --no-deps` | green |
| `make licence` | green |
| `make veto` | green: all fixtures accepted, all controls rejected |
| `make test-full` | green (fast tests, ignored slow cells, veto) |
| `cargo test --release --test differential rules_differential_full` | green, counts identical to plan 4 |

Differential: 84/11 FEN accepted/rejected, 27,088 positions, 540,383 legal
moves and child keys, 72,131 immediate explosions, 6,684 decided children
skipped, 642 material exemptions, 392 games / 26,612 plies, **0** mismatches,
**0** key mismatches, `MD3` 87 / 0 / 0. No rules surface changed.

`measurements/plan5/` holds the driver, `results.json`, `old.json`, `env.json`,
`env-old.json`, `core-perf.txt` and `probe/` (harness, `run_probe.py`,
`results.json`). One `movegen plan5` ledger row is appended in
[`../../bench.md`](../../bench.md); the initiative status gets its one-line
entry.

## Problems and scope changes

- The driver's `env.json` write is shared by every `--out`, so the old pass
  overwrote the candidate's environment file. Plan 5's copy now writes
  `env-<stem>.json` for a non-`results` output; `env-old.json` records the old
  run. No measurement changed.
- `W6` is only partly realised on the hot path: `Mover::new` still populates
  the full state. That is the audit's negative `StateInfo`-in-`Position`
  result, not a new miss; it is counted in the 0.06e9 state row.
- No scope change: no search, evaluation, threat, proof, checker or rules code
  was touched, and no second optimisation round was attempted.

## Open parts, missing tests and next steps

- **Distance to the goal:** 1.107× the same-session plan 2 binary, 1.435e9
  extra instructions on `core`, both `W7`/`W8` per-call targets missed. The
  crate-led channel is exhausted at the pre-recorded 1.12× ceiling. `M2`
  (native move storage) is complete and is why pseudo-generation and
  conversion no longer appear as a positive residual; it needs no further
  work.
- `M1`'s performance part and `M6` stay open; `MD6` is not held; the adoption
  stays under `MD1`/`MD5`. No `M8` handoff.
- Existing tests cover the adopted surface (native-encoding sweep,
  differential, EP regressions, fixtures, veto, counter pins). What is missing
  is a per-call-site counter in the prover and a standing per-node instruction
  budget; both are measurement instruments, not correctness tests.
- **Next is a formulation decision, not a third micro-optimisation round.**
  The owner options: (a) accept the measured regression as a new dated
  `MD1`/`MD6` row, keeping the crate as the rules layer; (b) fund a non-rules
  formulation change (`Mover`/tree/table) under a new item with its own bench
  evidence; (c) revert the hot path, which `MD1`'s 2026-11-19 reaffirmation
  rejects. The cheapest decisive experiment for (b) is a whole-binary
  instruction split by call site on `core`, not another per-call probe.

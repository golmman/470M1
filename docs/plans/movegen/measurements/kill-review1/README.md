# Kill review 1 measurements — the proof-export path

**Date:** 2026-11-20. **Host:** bench host (`c90face26537`; container UTS
`36f35886a6b3`, platform `Linux-7.2.13-200.fc44.x86_64`). **Pins:** the plan
12 product (`470MI@e3c237e` working tree, `8d807f21…`), the plan 2 reference
(`470m1@2f629fd` rebuilt in `/tmp/plan12`, `71fff646…`), provari `v0.4.0`
(`bceb1ee`). **Not a ledger row:** this is a decomposition probe for
[`kill-review1.md`](../kill-review1.md), not a new stack and not a new cap
(`docs/bench.md` rules 1 and 6). No source, pin or checker change.

## Files

- `run_killreview1.py` — five paired, interleaved rounds per cell of
  {product, plan 2} × {with `--proofs`, without}, then three `perf stat -e
  instructions:u` rounds per combination, then a symbol-level delta of the
  with/without runs on `core` (`perf record -c 100000`). Writes
  `results.json`.
- `run_provari_export.py` — the same five rounds for the pinned oracle with
  and without `--proofs`, so the protocol `a` export asymmetry has a number.
  Writes `provari_export.json`.
- `results.json`, `provari_export.json` — the parsed numbers and `env`.

Reproduce from a clean `/tmp/plan12` (the plan 12 driver builds it) and the
plan 12 product binary; both hashes are asserted in `env`.

## Result

| Cell | with proof | search only | export cand | export old |
| --- | ---: | ---: | ---: | ---: |
| `core` | 1.110 | 1.084 | 51 ms / 0.770e9 | 34 ms / 0.486e9 |
| `easy-win` | 1.075 | 1.051 | 9 ms / 0.117e9 | 5 ms / 0.068e9 |
| `deep-win` | 0.899 | 0.877 | 2.490e9 | 1.565e9 |
| **Geomean** | **1.024** | **1.000** | | |

Instruction ratios `core`: 1.110 with the export, 1.075 search-only; the
export delta (0.284e9) is 58% of the 0.492e9 gap. The with/without symbol
delta on `core` is in `results.json` (`symbol_delta_core`): `legal_in` +203M,
`is_pseudo_legal` +92M, `checkers` +67M, `play` +64M, `pseudo_legal` +63M,
`memmove` +54M, `format::*` +56M. The oracle's own export costs +4.7 ms
(`core`), −4.6 (`easy-win`) and −22.9 (`deep-win`), i.e. inside noise, for a
1021-position proof.

## What this cannot see

Wall is whole-process, one `perf_counter` bracket, five rounds; the row's
driver uses three. `easy-win` is a 0.13 s cell and `deep-win` a 5.4–6.2 s
cell, so the wall ratios carry a few percent of spread and the deep-win
export wall is not used (instructions are). The symbol delta is percentile ×
total instructions with inlining, so it names the rules functions the export
pays, not the call sites. No cycle, cache or branch counters were taken; no
function-level call counts were instrumented. The oracle comparison measures
the same cells on the same host but not the same session as the plan 12 row.

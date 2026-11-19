# Plan 2 measurements — en-passant immediate explosions

**Date:** 2026-11-18. **Host:** `f043976fff92` (x86_64, Linux 7.2.13).
**Harness:** `tests/differential.rs`, pin `atomic-movegen` `2.9.0`
(`af3faf0`), shakmaty `0.30.1`. **Bench:** product default (seed 0), 10 s,
192 MB, protocol `a`, three repeats. **Outcome:** full sweep green; 3/3 cells,
geomean **0.972**, 9/9 accepted, `D14` counters identical.

`results.json` is the claimed row. `prefix.json` is the pre-fix binary measured
after it in the same session; the prefix binary was built before the patch.
`env.json` records both candidates; the driver is plan5's, header only.

## Commands

```
cargo test --test differential fen_acceptance_and_rules_smoke -- --nocapture
cargo test --test differential rules_differential_full -- --ignored --nocapture
cargo test --test differential en_passant_explosion_regression -- --nocapture
```

```
python3 docs/plans/movegen/measurements/plan2/run_bench.py \
  --provari /tmp/470M1-oracle/provari/target/release/provari \
  --candidate target/release/470m1 \
  --out docs/plans/movegen/measurements/plan2/results.json \
  --seconds 10 --reference-seconds 10 --memory 192 \
  --repeats 3 --seeds 0 --protocols a --d14
```

The prefix run was the same command with
`--candidate target/release/470m1-prefix --out .../prefix.json`, run after the
claim. No raw transcripts or oracle artifacts are committed.

## Full sweep

3 seeds x 128 games x <= 200 plies, plus the fixture/corpus FENs, generated
acceptance mutations and the five en-passant fixtures. Two runs after the fix
gave identical counts; elapsed 1.47 s and 1.50 s.

| Category | Before | After |
| --- | ---: | ---: |
| FEN accepted / rejected | 79 / 11 | 84 / 11 |
| Positions compared | 26,869 | 27,088 |
| Legal moves / child keys compared | 538,453 | 540,383 |
| Play-out games / plies | 384 / 26,406 | 392 / 26,612 |
| Immediate explosions present | 71,971 | 72,131 |
| Decided children skipped (explosion is moot) | 6,644 | 6,684 |
| Material exemptions | 640 | 642 |
| Numeric key-table differences | 26,869 | 27,088 (recorded, not asserted) |
| Immediate-explosion mismatches | 86, all EP | **0** |
| `Mover::exposes` mismatches | 81, all EP | **0** |
| Node-level explosion mismatches | 5, all EP | **0** |
| Other rules mismatches | 0 | 0 |

The three EP counters are all zero and `mismatches: 0`. The numeric Zobrist
tables still differ on every paired position; that recorded finding is plan 3's
key decision, not a rules disagreement.

## Bench — protocol `a`, same session

| Cell | Reference wall | Candidate wall | Factor | Mate / ref mate | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.523 s | 0.513 s | **0.981** | 17 / 17 | 3,823,720 | 3/3 |
| `easy-win` | 0.136 s | 0.133 s | **0.978** | 15 / 15 | 897,937 | 3/3 |
| `deep-win` | 6.415 s | 6.147 s | **0.958** | 19 / 22 | 54,633,842 | 3/3 |
| **Geomean** | | | **0.972** | | | **9/9** |

Coverage 3/3. `D14`: the untimed export repeated the timed counters for all
three cells, and all three proofs were accepted by `470m1 check` and
`provari check`.

The pre-fix diagnostic (`prefix.json`) reads 1.016 / 0.992 / 1.025, geomean
**1.011**, 3/3 and 9/9 accepted, with `deep-win` at mate **22** and
`child_evals` 58,229,418. The fix therefore moved the tree as well as the wall
on that cell: the corrected EP explosion is seen as a winning resource, and the
19-move proof is accepted by both checkers. `core` and `easy-win` are within
the reference spread (0.5-0.53 s and 0.13-0.136 s).

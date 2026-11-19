# Plan 4 measurements — adopt `atomic-movegen` 2.10.0

**Date:** 2026-11-19. **Host:** `c90face26537` (x86_64, Linux 7.2.13).
**Pin:** `atomic-movegen` 2.10.0 (`5083647`), shakmaty 0.30.1.
**Bench:** product default (seed 0), 10 s, 192 MB, protocol `a`, three repeats.
**Outcome:** all four wishlist-2 surfaces landed and are adopted; the
differential and both checkers are green; the wall row is **1.099** against a
same-session plan 2 binary at **0.988**. The per-call targets are missed on
this host and are reported, not tuned.

`results.json` is the 2.10.0 row. `old.json` is the same-session plan 2 binary
(`target/release/470m1`, never committed) run with the same driver and
protocol. `env.json` belongs to `results.json`; the old run shares its host and
build settings. `core-perf.txt` holds the raw `perf stat` lines behind the
instruction table. `probe/` is the ten-FEN per-call harness and
`probe/results.json` its numbers.

## Commands

```
cargo test --release --test differential rules_differential_full -- --ignored --nocapture
cargo build --release
python3 docs/plans/movegen/measurements/plan4/run_bench.py \
  --provari /tmp/470M1-oracle/provari/target/release/provari \
  --candidate target/release/470MI \
  --out docs/plans/movegen/measurements/plan4/results.json \
  --seconds 10 --reference-seconds 10 --memory 192 \
  --repeats 3 --seeds 0 --protocols a --d14
```

The old diagnostic is the same command with
`--candidate target/release/470m1 --out .../old.json`. It was run after the
candidate and does not overwrite `results.json`.

```
cd docs/plans/movegen/measurements/plan4/probe
cargo build --release
./target/release/hotpath-probe legal_context 2000
perf stat -e instructions ./target/release/hotpath-probe legal_context 2000
```

The probe prints `ops` and a checksum; each row is the median of three
`instructions / ops` values. No raw transcripts are committed.

## Per-call probe (instructions per call)

| Operation | 2.10.0 | Target | Verdict |
| --- | ---: | ---: | --- |
| `populate_state` | 189.7 | — | old full state |
| `counts` | 49.9 | ~18 | measured on this host |
| `populate_counts` | 68.3 | — | counts-only state |
| `Board::legal` + full state | 168.7 | — | old legality |
| `LegalityContext::legal` | **121.9** | **≤100** | **miss** |
| adapter key path (`hash_and_rule50_after` + double-push `Legal`) | 243.8 | — | old effect+key |
| `child_effect_with_occupancy(Legal)` | **215.2** | **≤120** | **miss** |
| `null_for_advanced` | 35.9 | no rebuild | pass |
| native `Move::from_bits`/`to_bits` | 18.6 | 0 in prover | pass (native storage) |

The host is ~1.17× the audit's per-call scale (`Board::legal` here 168.7 vs
144 there), but even after normalising, `LegalityContext` lands near 104 and
`ChildEffect` near 184: `W7` is a small miss and `W8` is a large one. The
prover stores the native move and calls these entry points directly; no
per-candidate conversion remains. `Mover` still populates the full state
because it reads `checkers`/`pinned`; the counts-only path is used where no
such query follows.

## `core` instructions (seed 0, 10 s, no proof export)

| Binary | Instructions | Wall | Mate | `child_evals` |
| --- | ---: | ---: | ---: | ---: |
| plan 2 `470m1` | **6.569e9** | 480 ms | 17 | 3,823,720 |
| 2.10.0 `470MI` | **8.004e9** | 571 ms | 14 | 3,896,430 |
| plan 3 `470MI` (recorded) | 9.026e9 | 639 ms | 14 | 3,896,430 |

The adoption removes 1.02e9 instructions from plan 3 (−11.3%) and leaves
1.44e9 against plan 2 (1.219×). The audit's optimistic crate-led ceiling was
~7.35e9 (1.12×); the misses above are the named residual.

## Bench — protocol `a`, same session

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.506 s | 0.620 s | **1.225** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.131 s | 0.152 s | **1.160** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.500 s | 6.078 s | **0.935** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.099** | | | **9/9** |

Old-binary diagnostic, same driver and protocol:

| Cell | Reference | Old candidate | Old factor | Old mate / ref | Old `child_evals` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.519 s | 0.517 s | **0.996** | 17 / 17 | 3,823,720 |
| `easy-win` | 0.132 s | 0.134 s | **1.015** | 15 / 15 | 897,937 |
| `deep-win` | 6.487 s | 6.194 s | **0.955** | 19 / 22 | 54,633,842 |
| **Geomean** | | | **0.988** | | | |

Coverage 3/3. `D14`: the untimed export repeated the timed counters for all
three cells, and both checkers accepted every proof. The candidate/old ratio
is 1.230 / 1.134 / 0.981 by cell, geomean **1.113** — close to the audit's
1.12 ceiling and still short of the `MD6` bar. This row is input to plan 5,
not a closure.

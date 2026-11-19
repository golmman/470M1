# Plan 5 measurements — the `M1` decision

**Date:** 2026-11-19. **Host:** `c90face26537` (x86_64, Linux 7.2.13, 4 vCPU).
**Pin:** `atomic-movegen` 2.10.0 (`5083647`), shakmaty 0.30.1. **Tree:**
`470MI@a28fb33` (`atomic470mi` 0.1.0). **Bench:** product default (seed 0),
10 s, 192 MB, protocol `a`, three repeats.
**Outcome:** 3/3 cells, 9/9 accepted, `D14` identical. Candidate geomean
**1.094**; same-session plan 2 binary **0.997**. The `MD6` bar is **not held**
(candidate/old geomean 1.107), and the result is at the pre-recorded
crate-led ceiling (~1.12×), i.e. the expected finding.

`results.json` is the adopted 2.10.0 tree. `old.json` is the same-session plan 2
reference (`target/release/470m1`) run with the same driver and protocol.
`env.json` records both binaries, their sha256 and the rebuild. `core-perf.txt`
holds the raw `perf stat` lines. `probe/` is the per-call harness,
`probe/run_probe.py` its driver and `probe/results.json` its numbers.

## Binaries

| Side | Path | sha256 | Note |
| --- | --- | --- | --- |
| candidate | `target/release/470MI` | `14093d41…` | HEAD `a28fb33`, 2.10.0 |
| reference | `target/release/470m1` | `71fff646…` | rebuilt from `2f629fd` |

The reference was rebuilt this session: `git archive 2f629fd` into
`/tmp/movegen-plan2`, `cargo build --release`. The result is **bit-identical**
(`cmp`) to the stored `target/release/470m1`, so the stored binary is the plan 2
tree and did not drift. Neither binary is committed.

## Commands

```
cargo build --release
python3 docs/plans/movegen/measurements/plan5/run_bench.py \
  --provari /tmp/470M1-oracle/provari/target/release/provari \
  --candidate target/release/470MI \
  --out docs/plans/movegen/measurements/plan5/results.json \
  --seconds 10 --reference-seconds 10 --memory 192 \
  --repeats 3 --seeds 0 --protocols a --d14
```

The old-binary diagnostic is the same command with
`--candidate target/release/470m1 --out .../old.json`, run immediately after.
`core-perf.txt` is two interleaved `perf stat -e instructions,cycles,task-clock`
runs of each binary on the `core` FEN, 10 s, seed 0, no proof export.

```
cd docs/plans/movegen/measurements/plan5/probe
cargo build --release
python3 run_probe.py     # 3 x perf stat per op, writes results.json
```

No raw transcripts, oracle artifacts or `__pycache__` are committed.

## Bench — protocol `a`, same session

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.525 s | 0.634 s | **1.208** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.133 s | 0.154 s | **1.158** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.587 s | 6.165 s | **0.936** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.094** | | | **9/9** |

Reference wall spread per cell: 0.515–0.526 / 0.133–0.136 / 6.506–6.597.

Same-session plan 2 binary, same driver and protocol:

| Cell | Reference | Old candidate | Old factor | Old mate / ref | Old `child_evals` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.518 s | 0.525 s | **1.014** | 17 / 17 | 3,823,720 |
| `easy-win` | 0.132 s | 0.134 s | **1.015** | 15 / 15 | 897,937 |
| `deep-win` | 6.549 s | 6.303 s | **0.962** | 19 / 22 | 54,633,842 |
| **Geomean** | | | **0.997** | | | |

The reference reads 0.997 against its recorded 0.972 anchor (+2.6%), at the top
of the 0.972–0.997 band observed across sessions; the decision does not hinge
on it: the candidate is 10.7% slower on median walls (geomean 1.107).

Candidate / old per cell: **1.208 / 1.149 / 0.978**, geomean **1.107**. The
per-node ratio (`wall ratio / child_evals ratio`) is 1.186 / 1.141 / 1.191, so
the wall loss is a per-node cost on all three cells and the `deep-win` gain is
its 18% smaller tree, not cheaper nodes.

Coverage 3/3. `D14`: the untimed export repeated the timed counters for all
three cells; both checkers accepted every proof (9/9), old and new.

## `core` instructions (seed 0, 10 s, no proof export)

Two runs each, interleaved, `perf stat`:

| Binary | Instructions | Wall | Mate | `child_evals` |
| --- | ---: | ---: | ---: | ---: |
| plan 2 `470m1` | **6.569e9** | 494 ms | 17 | 3,823,720 |
| 2.10.0 `470MI` | **8.004e9** | 568 ms | 14 | 3,896,430 |

The gap is **1.435e9 (1.218×)**; instructions per `child_evals` are 1718 vs
2054 (+19.6%). Counters are identical across the two runs of each binary.

## Per-call probe (instructions per call, median of three)

| Operation | 2.10.0 | Target | Verdict |
| --- | ---: | ---: | --- |
| `populate_state` (full) | 189.7 | — | still used at `Mover::new` |
| `counts` | 49.9 | — | counts-only |
| `populate_counts` | 68.3 | — | counts-only |
| `Board::legal` + full state | 168.7 | — | host anchor |
| `LegalityContext::legal` (`W7`) | **121.9** | **≤100** | **miss** |
| adapter effect path (`hash_and_rule50_after` + double push) | 243.8 | — | replaced |
| `child_effect_with_occupancy(Legal)` (`W8`) | **215.2** | **≤120** | **miss** |
| `null_for_advanced` (`W9`) | 35.9 | no rebuild | pass |
| native `Move` round-trip | 18.6 | 0 in prover | pass (native storage) |

The numbers reproduce plan 4 to the digit: no release or API change since, so
the per-call residual is unchanged. The host is ~1.17× the audit's scale
(`Board::legal` 168.7 here vs 144 there); normalised, `W7` is near 104 and
`W8` near 184. `W6` is used for the per-`Position::legal` path, but the
candidate loop's `Mover::new` still populates the full state because `tested`
reads `checkers`/`pinned`.

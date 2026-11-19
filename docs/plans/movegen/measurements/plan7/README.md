# Plan 7 measurements — eval geometry (`M12`)

**Date:** 2026-11-19. **Host:** `c90face26537` (x86_64, Linux 7.2.13, 4 vCPU).
**Pin:** `atomic-movegen` 2.10.0 (`5083647`), shakmaty 0.30.1. **Tree:**
`470MI@e4588cb` + this plan's working tree (`atomic470mi` 0.1.0).
**Outcome:** `M12` adopted. `core` falls **−0.1879e9** instructions (−2.35%)
with identical counters, mate and `.pvr`; 10 s geomean **1.056** against a
same-session plan 2 binary at **0.987** (candidate/old 1.070, down from 1.107).
`MD6` is still not held, as the plan pre-registered.

## Binaries

| Side | Path | sha256 | Note |
| --- | --- | --- | --- |
| candidate (M12) | `target/release/470MI` | `b7c16c94…` | this plan's tree |
| unchanged HEAD | `/tmp/plan7-base-470MI` | `14093d41…` | bit-identical to plan 5/6 |
| plan 2 reference | `target/release/470m1` | `71fff646…` | rebuilt from `2f629fd`; bit-identical to `/tmp/movegen-plan2` |

Neither binary nor a `perf.data` file is committed.

## Phase A — pre-registered budget

`core-perf.txt` is A.1 (unchanged tree vs plan 2, three interleaved
`perf stat -e instructions:u` runs): plan 2 **6.5693769e9** (spread 100),
unchanged HEAD **8.0040859e9** (spread 703). `perf-annotate.txt` is A.3, the
`Dfpn::evaluate` disassembly excerpts in both binaries, recorded for the next
decision only.

`counts.patch` is the temporary counted build (applied in the working tree,
never committed) and `counts.json` its `core` output. It counts, per `terms`
call, pieces and roles present per side; per `evaluate`, total non-king pieces
and the defender's distinct roles; and every `attacks::attacks` call inside
`eval`. Search counters are unchanged: `child_evals` 3,896,430, `expansions`
191,557, `nodes` 350,430, mate 14.

| Quantity | Value |
| --- | ---: |
| `terms` calls | 383,114 |
| `evaluate` calls | 191,557 |
| Pieces over all `evaluate` calls | 4,386,775 (mean **22.90**) |
| Defender role masks over all `evaluate` calls | 831,332 (mean **4.34**) |
| Median pieces / roles / `N − R` per `evaluate` | 23 / 4 / **19** |
| Median pieces / roles / `N − R` per `terms` call | 12 / 4 / **7** |
| `attacks::attacks` calls inside `eval` (old) | 8,773,550 |
| Old check-mask calls (one per piece, both sides) | 4,386,775 |
| New check-mask calls (one per defender role) | 831,332 |
| Removed check-mask calls | **3,555,443** |

The gate (`N − R > 1` on the median `terms` call) passes at 7; on the
`evaluate` formulation it is 19.

## Phase B/C — `M12` and its falsifier

`eval::terms` now walks each side by role and reuses the `Mover`'s
`check_squares` for the side to move; only the defender's per-role masks are
built, once, and only when both kings exist and do not touch. The
pre-formulation is `#[cfg(test)] terms_reference`, and two unit tests compare
`Terms` and `evaluate` over the differential corpus and seeded play-outs.

`core-perf.txt` holds the three interleaved falsifier pairs: unchanged HEAD
**8.0040859e9** (spread 437) against M12 **7.8162191e9** (spread 299), deltas
187,866,613 / 187,866,675 / 187,867,020 instructions. Median fall
**0.1879e9**, i.e. **52.8** instructions per removed check-mask call. The
pre-registered estimate was 0.07e9 ± 0.04e9; the measured fall is larger, not
smaller, and is outside both spreads. Median wall 588 -> 564 ms. The `.pvr`
bytes are identical on all three cells.

## Bench — protocol `a`, same session

Driver: plan 3's `measurements/plan3/run_bench.py`, unchanged; command with
`--seconds 10 --reference-seconds 10 --memory 192 --repeats 3 --seeds 0
--protocols a --d14`. `bench/results.json` is M12, `bench/old.json` the plan 2
binary measured immediately after; `env.json` / `env-old.json` record both.

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.516 s | 0.619 s | **1.200** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.136 s | 0.149 s | **1.096** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.585 s | 5.896 s | **0.895** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.056** | | | **9/9** |

Same-session plan 2 binary, same driver and protocol:

| Cell | Reference | Old candidate | Old factor | Old mate / ref | Old `child_evals` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.514 s | 0.530 s | **1.031** | 17 / 17 | 3,823,720 |
| `easy-win` | 0.137 s | 0.134 s | **0.978** | 15 / 15 | 897,937 |
| `deep-win` | 6.565 s | 6.267 s | **0.955** | 19 / 22 | 54,633,842 |
| **Geomean** | | | **0.987** | | | |

Coverage 3/3; every proof accepted by our checker and `provari check` (9/9);
`D14` counters identical for every cell. Candidate/old is **1.070** (was
1.107 in plan 5), so `MD6` is still not held. The reference reads 0.987
against its 0.972 anchor.

## What this instrument cannot see

`perf stat` and the counted build see the same search and the same binary, but
the counted build is not used for timing. The bench cannot separate `M12`'s
instruction fall from the guide's unchanged ordering; it sees only that the
tree, counters and proof bytes did not move.

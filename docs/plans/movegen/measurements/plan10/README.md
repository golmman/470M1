# Plan 10 measurements — adopt `atomic-movegen` 2.11.0 (`W10`)

**Date:** 2026-11-20. **Host:** bench host (`c90face26537`; this container's
UTS name is `36f35886a6b3`, platform `Linux-7.2.13-200.fc44.x86_64`, the same
string as plan 7/9). **Pins:** `atomic-movegen` 2.11.0 (`0aaf958`, MIT),
checksum `46b5519…`; provari 0.4.0 (`bceb1ee`). **Tree:** `470MI@a79f68f` +
the plan's working tree. **Outcome:** `W10` adopted; `MD6` row measured and
still missed.

## Files

- `run_plan10.py` — the driver. It builds `base` (`git archive HEAD`, 2.10.0)
  and `w10` (`w10.patch`: pin, lock and the one call) under `--scratch`
  (`/tmp/plan10`), runs the ignored full differential in `w10`, then three
  interleaved rounds of `perf stat -e instructions:u` on `core`, `easy-win`
  and `deep-win`, order `base`, `w10`. Each variant exports one `.pvr` per
  cell for sha256 and both checkers. It writes `perf.txt`, `results.json`,
  `env.json` and `decision.json`. `--allow-dirty` is required when the working
  tree holds the same change, because the driver archives HEAD, never the
  working tree.
- `w10.patch` — `Cargo.toml` pin, `Cargo.lock` 2.11.0 entry and
  `legality_context_with_state(counts, &state)`; no test, docs or search edit.
- `bench/` — the plan 3 driver's 10 s protocol-`a` row: `results.json`
  (product 2.11.0), `old.json` (plan 2 binary rebuilt from `2f629fd`) and
  `env.json` (both sides; the old pass overwrote the driver's `env.json`, so
  the candidate fields and both hashes were reconstructed from the parsed
  result and the binaries).
- `core-perf.txt` — three interleaved `instructions:u` pairs, product vs plan 2.
- `perf.txt`, `results.json`, `env.json`, `decision.json` — Phase C evidence.

No binaries, `.pvr`, `perf.data`, raw transcripts or oracle artifacts are
committed.

## Phase C — `W10` fall (base vs patched 2.11.0)

```sh
python3 docs/plans/movegen/measurements/plan10/run_plan10.py --allow-dirty
```

`base` is sha256 `b7c16c94…`, the same binary plan 8/9 measured; `w10` is
`62986012…`. Three rounds, pair falls `base_r − w10_r`:

| Cell | base median | `w10` median | fall | fall % | pair falls | wall (ms) |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| `core` | 7,816,219,420 | 7,147,608,959 | **668,610,461** | 8.554 | 668,610,558 / 668,610,461 / 668,610,217 | 559 → 529 |
| `easy-win` | 1,640,875,516 | 1,560,746,666 | 80,128,850 | 4.883 | 80,128,550 / 80,128,668 / 80,129,044 | 136 → 132 |
| `deep-win` | 82,738,904,016 | 77,783,667,935 | 4,955,236,081 | 5.989 | 4,955,236,036 / 4,955,236,315 / 4,955,236,106 | 5690 → 5482 |

The rule is `core` median fall ≥ **0.31e9**; the measured 0.6686e9 clears it.
The plan's expectation was 0.5–0.72e9, so the crate-side mask sits below plan
9's 0.7246e9 prototype and inside the range.

Correctness, all cells: counters and mate identical in **every** round; the
`.pvr` is byte-identical (`core` `714723211c7a0246…`, `easy-win`
`32ffb30d96ccb172…`, `deep-win` `50f062de2a67b1dd…`); our checker and
`provari check` accept **9/9**; the full differential reports
`mismatches: 0`, `key_mismatches: 0`, EP 87/0/0 (27,088 positions, 540,383
moves/keys). `decision.json` branches `w10` to **release**.

## `MD6` row — 10 s, protocol `a`, seed 0, 192 MB, three repeats

Driver: plan 3's `run_bench.py`, unchanged. `results.json` is the product
2.11.0 binary (`2eb218a6…`; the C-phase `w10` binary differs by the board
comment's line metadata and the `#[cfg(test)]` test, not by the release code
path); `old.json` is the plan 2 binary (`71fff646…`, rebuilt bit-identical
from `2f629fd`). The two passes ran back to back.

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.511 s | 0.578 s | **1.131** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.132 s | 0.143 s | **1.083** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.527 s | 5.627 s | **0.862** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.018** | | | **9/9** |

Same-session plan 2 binary, same driver and protocol:

| Cell | Reference | Old candidate | Old factor | Old mate / ref | Old `child_evals` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.519 s | 0.518 s | **0.998** | 17 / 17 | 3,823,720 |
| `easy-win` | 0.133 s | 0.135 s | **1.015** | 15 / 15 | 897,937 |
| `deep-win` | 6.517 s | 6.255 s | **0.960** | 19 / 22 | 54,633,842 |
| **Geomean** | | | **0.991** | | | **9/9** |

Coverage 3/3, 9/9 accepted by both checkers, `D14` counters identical for
every cell. Candidate/old is **1.133 / 1.067 / 0.898**, geomean **1.028** (was
1.070 in plan 7), so `MD6` is still **not held**; the reference reads 0.991
against its 0.972 anchor. `core` instructions are **7.1476e9** vs plan 2's
**6.5694e9**, a **1.088×** gap (was 1.190×); the `W10` fall over 2.10.0 is
0.6686e9.

## What this instrument cannot see

The C-phase pair measures the crate release against the unchanged 2.10.0 tree
with the same compiler and flags, so it isolates the constructor call. The
`MD6` row sees the whole product but not the host's physical label; host
identity rests on the base binary sha256, the platform string and the base
`core` reading. The differential counts a rules disagreement but cannot prove
the mask's rejection set is complete beyond the crate's own oracle tests; the
crate's release notes carry that evidence.

# Plan 3 measurements — the 10 s parity gate

**Date:** 2026-11-17. **Host:** `8f5ca2f2750a` (aarch64, Linux 7.2.7).
**Oracle:** provari `0.4.0` at `bceb1ee`, built in `/tmp/470M1-oracle`.
**Candidate:** `target/release/470m1` from the plan2 tree (no source change).
**Corpus:** `corpus-v1`, sha256
`9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`.
`run_bench.py` is the driver (`env-10s.json`, `env-120s.json` the pins).

## Protocol

`run_bench.py` runs the three fixed cells, one process at a time, in **three
protocol variants inside one interleaved pass** so that warm-up and drift hit
both sides alike:

| Variant | Reference side | Candidate side |
| --- | --- | --- |
| **a** (ledger protocol) | `provari prove --guide hand --seconds 10 --memory 192` | `470m1 prove --seconds 10 --memory 192 --seed S --proofs TMP` |
| **b** (symmetric end to end) | as a, plus `--proofs` | as a |
| **c** (search wall only) | as a | as a, without `--proofs` |

Three repeats per cell; the candidate runs seeds 0–2; the reference has no
seed. Every candidate proof is checked by `470m1 check` and `provari check`;
only both-accepted runs carry a factor. Each repeat rotates the run order by
one place, so the first-run warm-up lands on a different task every time. In
variant **c**, a run is claimed when the identical run with the proof (same
cell, seed and repeat) was accepted: the search is deterministic.

`results-10s*.json` are four independent repeats of the 10 s pass;
`results-120s.json` is one pass with the candidate at a 120 s cap and the
reference still at 10 s.

## 10 s matrix, protocol a (final pass, `results-10s.json`)

| Cell | Ref wall | Cand wall | Factor | Ref mate | Cand mate[^mate] | Child evals | Expansions | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.389 s | 0.413 s | 1.062 | 17 | 15 | 4,160,024 | 204,264 | 9/9 |
| `easy-win` | 0.086 s | 0.085 s | 0.988 | 15 | 20 | 906,543 | 52,030 | 9/9 |
| `deep-win` | 5.265 s | 4.498 s | 0.854 | 22 | 20 | 56,461,476 | 3,115,411 | 9/9 |

**Geomean 0.964, coverage 3/3.** Every proof accepted by both checkers.

[^mate]: Candidate wall is the median over nine runs (three seeds × three
    repeats); the mate, `child_evals` and `expansions` are those of the run at
    that median (seed 1, seed 1, seed 0). Seed 0 — provari's own tree — found
    17 / 15 / 22, and seeds 1–2 found shorter proofs at larger trees on
    `core` (15 and 14).

### The same pass under the other protocols

| Variant | `core` | `easy-win` | `deep-win` | Geomean |
| --- | ---: | ---: | ---: | ---: |
| a | 1.062 | 0.988 | 0.854 | 0.964 |
| b | 1.087 | 1.000 | 0.887 | 0.988 |
| c | 0.985 | 0.930 | 0.845 | 0.918 |

### Repeatability, four 10 s passes (protocol a)

| Pass | `core` | `easy-win` | `deep-win` | Geomean |
| --- | ---: | ---: | ---: | ---: |
| `results-10s-run1.json` | 1.053 | 1.027 | 0.903 | 0.992 |
| `results-10s-run2.json` | 1.089 | 1.026 | 0.929 | 1.013 |
| `results-10s-run3.json` | 1.023 | 1.013 | 0.892 | 0.974 |
| `results-10s.json` | 1.062 | 0.988 | 0.854 | 0.964 |

Files `run1`/`run2` come from earlier driver revisions (no rotation of the
run order, and coarser `protocol` labels on their `rows`); their `summary`
sections are the ones quoted. `run3` and `results-10s.json` are the final
driver. Session median **0.983**, range 0.964–1.013. The reference wall of `core`
moved 0.350 → 0.389 s and of `easy-win` 0.075 → 0.086 s across passes: the
host drifted ~10% while the ratio moved ~5%. **The 10 s gate is at the edge of
this instrument's resolution**: a 2% margin is inside the noise band.

## Per-node and tree terms (protocol a, seed 0 = provari's tree)

At seed 0 the candidate's node count equals the reference's on all three
cells (`core` 342,289, `easy-win` 72,523, `deep-win` 4,574,912), so the
walls are a per-node comparison on an identical tree.

| Cell | Ref µs/node | Cand µs/node (a) | Cand µs/node (c) | Tree |
| --- | ---: | ---: | ---: | --- |
| `core` | 1.136 | 1.122 | 1.028 | identical |
| `easy-win` | 1.186 | 1.172 | 1.103 | identical |
| `deep-win` | 1.151 | 1.002 | 1.014 | identical |

The candidate's **per-node cost is at or below provari's** — 8–10% below once
the proof export is taken out (variant c, `core` 0.352 s vs 0.389 s). The gap
is not per-node cost.

The tree is the gap on `core`: seeds 1 and 2 inflate it by 10% and 17%
(376,527 and 401,891 nodes) and only their walls set the 9-run median
(`core` 0.384 / 0.413 / 0.436 s for seeds 0/1/2 against a 0.389 s reference).
`deep-win` is the mirror: its median run is *shorter* than provari's tree.

## Product overheads (temporary local instrumentation, reverted)

| Item | `core` | `easy-win` | `deep-win` |
| --- | ---: | ---: | ---: |
| Proof export (`format::encode`, wall) | 26 ms | 0.9 ms | 26 ms |
| … of which `play` + `in_check` per legal move | ~12 ms | — | — |
| … of which `legal_moves` (45k calls) | ~9 ms | — | — |
| … of which `wins_in_two` (14k calls) | ~9 ms | — | — |
| Memory zero-fill at `--memory 192` (vs 8) | ~4.5 ms | ~4.5 ms | ~4.5 ms |
| Process start (`--version`) | ~1.0 ms | ~1.0 ms | ~1.0 ms |
| provari's memory zero-fill / start | ~3 ms / 0.4 ms | same | same |

`--proofs` costs the candidate 8% of `core`'s wall; provari's reference pays
nothing under protocol a. Under protocol b both pay their own export.

## Lever evidence (none adopted)

**HOTPATH-V2 / NULL-PIPE** — the atomi levers replace per-node work. The
measurement above says per-node is already at or below provari's on an
identical tree, and a `perf` comparison of the two binaries on `core` shows
the same profile to within a point (`pseudo_legal` 8.2/8.4%, `mid` 7.9/9.0%,
`play` 7.5/7.7%, `evaluate` 8.3/7.4%, `checked_after` 7.3/7.2%). No measured
gap justifies a port. Closed with this evidence.

**Proof-retention share** — `deep-win` finds a 19-move proof (38.2 M child
evals) when the proof table has 144 MB and a 22-move proof (58.2 M) with the
10 s split's 64 MB. A sweep at 192 MB, seeds 0–2, two repeats, temporary
`SPLIT` hook, reverted:

| Proof MB | 64 | 96 | 112 | 120 | 128 | 136 | 144 | 152 | 160 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Median wall [s] | 4.380 | 4.126 | 4.226 | 4.114 | 4.210 | 4.285 | 3.997 | 4.736 | 4.251 |
| Median mate | 22 | 20 | 22 | 20 | 22 | 19 | 19 | 20 | 20 |

The proofs **never fill** (no `drop_mates`, no `proofs_full`) at any split,
and the wall is not monotone in the proof share: the tree moves by up to 35%
with the table's size alone. The 144 MB point is the 120 s cap's split, not a
rule. Adopting it would fit one bench cell's trajectory lottery, so it is
**not adopted**; the 120 s row's 0.831 on `deep-win` is this effect, not a
mechanism.

**Guide/EF-PN tuning** — not attempted: no held-out set in this plan.

## 120 s candidate cap (`results-120s.json`, protocol a)

| Cell | Ref wall | Cand wall | Factor | Ref mate | Cand mate | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.383 s | 0.422 s | 1.102 | 17 | 15 | 9/9 |
| `easy-win` | 0.084 s | 0.085 s | 1.012 | 15 | 20 | 9/9 |
| `deep-win` | 5.291 s | 4.396 s | 0.831 | 22 | 19 | 9/9 |

Geomean 0.975, coverage 3/3. The cap changes `proofs_needed` and so the
memory split (48 MB table / 144 MB proofs); the same tree then costs one node
more or fewer than at 10 s (`core` 342,290 vs 342,289), which is the split,
not the clock.

## D14 — timed and untimed counters

For every cell and seed the search was repeated with `--proofs` and the same
`--seconds` and flags: `mate`, `nodes`, `child_evals` and `expansions` are
identical to all three timed repeats, and both checkers accept the proof
(`results-10s-run1.json`, `results-120s.json`, section `d14`). Exporting does
not touch the trajectory.

## What this instrument cannot see

- The reference is a single (seedless) tree per repeat; the candidate's median
  is over three perturbed trees. The seed's effect on the tree (10–17% on
  `core`) is larger than the reported margins.
- Wall at 70–400 ms includes ~1 ms of process start and ~4.5 ms of memory
  zero-fill; neither is search work, and both differ between the binaries.
- `child_evals` is ours only; provari's `nodes` is not the same unit.
- No profile of `deep-win` (the guide's effect there may differ), no held-out
  set, no cross-host check.

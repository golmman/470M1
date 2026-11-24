# Plan 1 measurements

Session 2026-11-24, one host (`f54c552f3ece`, platform
`Linux-7.3.4-401.asahi.fc44.aarch64+16k-aarch64-with-glibc2.43`, `nproc` 4,
rustc 1.100.0), one process at a time. Preflight `make test-full` passed
before any measurement.

## Instrument

| Item | Value |
| --- | --- |
| Reference | provari `v0.5.0` `b87d179c…`, `--release --features bundled` |
| Reference binary sha256 | `3cd1a4ecf4aed82ca3b37f1e4a5af55d9457a0b1f4b44dfb8cb5abebe7118842` |
| Network / model | `3d76258c…5c40b1` / `51be8f47…31882b` |
| Tablebases | Lichess 3-4-5, manifest `a0c7a69b…fdeefc37`, 145 files, all md5 verified |
| Candidate | `target/release/470MI`, sha256 `25cc08d869699f2ec061afabc8a3539c37dce53f3e8fe8e959f1500bcf427fee` |
| Cap / memory | 10 s / 192 MB, three repeats, interleaved `ref,cand` |

The driver gates the corpus, `high200`, the set file and the tablebase
manifest digests and the reference version string; `env.json` in each
directory records both binary sha256s, the data digests, `N`, cap and memory.

## Calibration (`calibration/`)

`P = nproc = 4`, `Nmax = 4`; `high200` 1-based lines 3, 23, …, 143.
`r(1) = 10.0240`, `r(2) = 10.0237`, `r(4) = 10.0250` s → **N = 1**. No
calibration cell proves at 10 s, so each wall is the cap and the ratio is
1.0; the rule degenerates on this host (report finding). `runs.json` holds
all 72 runs, `env.json` the choice.

## Selection (`select-matched/`, `select-goal/`)

Universe 201 (`core` + 200 `high200`), 8 calibration cells excluded → 193
eligible. Reference only, `--guide model --seed 0`, one repeat; fast win
(under 9.0 s) joins 1/1, slow/failed is retried, a cell joins iff proved in
2 of 3, two failures eliminate. Both columns: **25 members**, strong 20,
held-out 5, identical membership (`at N = 1` the tablebases added no cell).

| Set | sha256 | Members | Strong | Held-out |
| --- | --- | ---: | ---: | ---: |
| `bench/sets/matched-v1.json` | `810289f0…d83d1` | 25 | 20 | 5 |
| `bench/sets/goal-v1.json` | `0129ff7c…516e9f` | 25 | 20 | 5 |
| `bench/sets/guard.json` | `199c03ab…60bfb` | 3 | 3 | 0 |

Raw attempts are in each `selection.json`.

## Rows

| Directory | Column | Coverage | Geomean | Notes |
| --- | --- | ---: | ---: | --- |
| `matched/` | `matched-v1` | 9/20 | 3.104 | 8 censored (bounds 1.12–6.35), 3 unstable |
| `goal/` | `goal-v1` (reference only) | 20/20 | — | 20/20 reference claims accepted |
| `guard/` | `guard` | 3/3 | 1.023 | both sides hand; claims 3/3 both checkers |
| `distance.json` | goal distance | 9/20 | 2.971 | candidate walls reused from `matched/` |

`matched/README.md`, `goal/README.md` and `guard/README.md` are the
driver-written per-cell tables; `summary.json` has medians, spreads,
factors, bounds and mates; `results.json` has every run with its command.

**Claims.** `matched/claims.json`: 9/9 candidate proofs accepted by
`470MI check` and `provari check`; 9/9 reference proofs by `provari check`.
`goal/claims.json`: 20/20 reference proofs by `provari check --tablebases`.
`guard/claims.json`: 3/3 by both. The `.pvr` files are kept under each
`proofs/` directory so the verdicts can be re-run.

**Determinism.** For every claimed cell the three candidate runs have
identical `nodes`, `child_evals` and `mate`. The 8 censored cells differ only
in the counters at the 10 s wall cut; a fixed-`--evals` probe
(`1051b3d28a27`, 6,000,000 evals) repeated identically (494,712 nodes,
6,000,021 evals, root `[518,340106]`), so the trajectory is deterministic and
the difference is the deadline.

## Reproduce

```
make oracle tablebases
make bench-calibrate            # writes calibration/, prints r(N), N
make bench-sets N=1             # writes bench/sets/{matched,goal}-v1.json
make bench N=1                  # matched row + claims
make bench-guard                # guard row + claims
# The goal reference (the first run reused the matched candidate walls):
python3 bench/run_bench.py --mode row --set bench/sets/goal-v1.json --side ref \
    --guide model --threads 1 --tablebases /tmp/470M1-oracle/tablebases/3-4-5 \
    --claim --out docs/plans/foundation/measurements/plan1/goal
```

`make bench-goal N=1` is the full both-sides form; the first goal run was
`--side ref` and its candidate walls came from `matched/` (same command,
same session), combined in `distance.json`.

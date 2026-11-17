# Plan 4 measurements — corrected-seed 10 s regression

**Date:** 2026-11-17. **Host:** `8f5ca2f2750a` (aarch64, Linux 7.2.7).
**Oracle:** provari `0.4.0` at `bceb1ee`, built in `/tmp/470M1-oracle`.
**Candidate:** `target/release/470m1`, sha256
`b430c34ab2547f0c94f7432c5b4cad4bb7ee2c2b271f5f9a91bc0175028e06e6`,
built after the D8 fix. **Corpus:** `corpus-v1`, sha256
`9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`.

`run_bench.py` is plan3's driver copied unchanged apart from its header. It
runs the three fixed cells in one interleaved session, protocol a only
(candidate exports, reference does not), seeds 0–2, three repeats, one process
at a time. `env.json` is the pin record; `results.json` has the raw parsed
rows, the per-seed summary and the `D14` checks.

## Command

```
python3 docs/plans/parity/measurements/plan4/run_bench.py \
  --provari /tmp/470M1-oracle/provari/target/release/provari \
  --candidate target/release/470m1 \
  --out docs/plans/parity/measurements/plan4/results.json \
  --seconds 10 --reference-seconds 10 --memory 192 \
  --repeats 3 --seeds 0,1,2 --protocols a --d14
```

## Result

| Cell | Ref wall | Cand wall | Factor | Ref mate | Cand mate | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.400 s | 0.422 s | 1.055 | 17 | 17 | 9/9 |
| `easy-win` | 0.085 s | 0.094 s | 1.106 | 15 | 15 | 9/9 |
| `deep-win` | 5.225 s | 4.701 s | 0.900 | 22 | 22 | 9/9 |

**Coverage 3/3, geomean 1.016, cap 10 s.** This is a regression row, not a
speed claim. Every candidate proof was accepted by both checkers; `D14`
timed and untimed counters are identical for all 9 cell-seed pairs.

## Per seed

| Cell | Seed 0 factor | Seed 1 factor | Seed 2 factor |
| --- | ---: | ---: | ---: |
| `core` | 0.965 | 1.147 | 1.055 |
| `easy-win` | 1.012 | 1.106 | 1.118 |
| `deep-win` | 0.931 | 0.900 | 0.864 |
| **geomean** | **0.965** | — | — |

The D8 fix removed the guide-step magnitude perturbation. The seed now picks
only among children with equal deciding number and equal history, so seed 0 is
the product search and seeds 1–2 differ only where such a tie exists. That is
still enough to reorder `core`'s tree (seed 0 342,289 nodes; seeds 1–2
406,879 / 369,860), so the three-seed median remains above the product
default. No cap was changed and no cell was dropped.

## `D14` counters

All nine cell-seed pairs match the timed run exactly:

| Cell | Nodes (seed 0 / 1 / 2) | Child evals (0 / 1 / 2) |
| --- | --- | --- |
| `core` | 342,289 / 406,879 / 369,860 | 3,823,765 / 4,619,153 / 4,241,810 |
| `easy-win` | 72,523 / 79,633 / 82,964 | 897,937 / 981,405 / 971,340 |
| `deep-win` | 4,574,912 / 4,438,637 / 4,238,713 | 58,229,418 / 56,145,712 / 54,066,365 |

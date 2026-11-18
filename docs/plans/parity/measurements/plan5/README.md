# Plan 5 measurements — `P5` closure, product default

**Date:** 2026-11-17. **Host:** `8f5ca2f2750a` (aarch64, Linux 7.2.7).
**Oracle:** provari `0.4.0` at `bceb1ee`, built in `/tmp/470M1-oracle`.
**Candidate:** `target/release/470m1`, sha256
`b430c34ab2547f0c94f7432c5b4cad4bb7ee2c2b271f5f9a91bc0175028e06e6`.
**Corpus:** `corpus-v1`, sha256
`9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`.

`run_bench.py` is plan4's driver copied unchanged apart from its header. It
runs the three fixed cells in one interleaved session, protocol a only
(candidate exports, reference does not), seed 0 (the product default), three
repeats, one process at a time. `env.json` is the pin record; `results.json`
has the raw parsed rows, the summary and the `D14` checks.

## Command

```
python3 docs/plans/parity/measurements/plan5/run_bench.py \
  --provari /tmp/470M1-oracle/provari/target/release/provari \
  --candidate target/release/470m1 \
  --out docs/plans/parity/measurements/plan5/results.json \
  --seconds 10 --reference-seconds 10 --memory 192 \
  --repeats 3 --seeds 0 --protocols a --d14
```

## Result

| Cell | Ref wall | Cand wall | Factor | Ref mate | Cand mate | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.358 s | 0.335 s | 0.936 | 17 | 17 | 3/3 |
| `easy-win` | 0.078 s | 0.075 s | 0.962 | 15 | 15 | 3/3 |
| `deep-win` | 4.584 s | 4.301 s | 0.938 | 22 | 22 | 3/3 |

**Coverage 3/3, geomean 0.945, cap 10 s.** This is the `P5` closure row on
the product default. Every candidate proof was accepted by both checkers
(9/9); `D14` timed and untimed counters are identical for all three cells.

## `D14` counters

| Cell | Mate | Nodes | Child evals | Expansions | Match |
| --- | ---: | ---: | ---: | ---: | --- |
| `core` | 17 | 342,289 | 3,823,765 | 189,605 | yes |
| `easy-win` | 15 | 72,523 | 897,937 | 52,251 | yes |
| `deep-win` | 22 | 4,574,912 | 58,229,418 | 3,198,022 | yes |

The counters are seed 0's, identical to plan4's seed-0 run. No cap, cell,
memory split or source changed.

# Plan 2 measurements — search core

**Date:** 2026-11-17. **Host:** `8f5ca2f2750a` (aarch64, Linux 7.2.7).
**Oracle:** provari `0.4.0` at `bceb1ee`, built in `/tmp/470M1-oracle`.
**Candidate:** `target/release/470m1` from the plan2 working tree.
**Corpus:** `corpus-v1`, sha256
`9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`.

## Protocol

`run_bench.py` runs the three fixed cells, one process at a time:

- reference: `provari prove FEN --seconds 10 --memory 192 --guide hand --json`,
  three repeats;
- candidate: `470m1 prove FEN --seconds CAP --memory 192 --guide hand
  --seed S --proofs TMP --json`, seeds 0–2, three repeats each;
- every candidate proof is checked by `470m1 check` and `provari check`; only
  both-accepted rows count.

`results.json` is the plan's 120 s cap. `results-10s.json` is a same-session
10 s matrix, recorded as evidence for plan3, not as the plan2 gate.

## Results at 120 s

| Cell | Reference wall | Candidate wall | Factor | Ref mate | Cand mate | Child evals | Expansions | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.350 s | 0.382 s | 1.091 | 17 | 17 | 3,823,775 | 189,606 | 9/9 |
| `easy-win` | 0.079 s | 0.078 s | 0.987 | 15 | 15 | 897,937 | 52,251 | 9/9 |
| `deep-win` | 4.584 s | 4.065 s | 0.887 | 22 | 19 | 38,205,943 | 2,098,415 | 9/9 |

Geomean over three claimed cells: **0.985**. Coverage 3/3. Every proof was
accepted by both checkers.

## Results at 10 s (plan3 evidence)

| Cell | Reference wall | Candidate wall | Factor | Ref mate | Cand mate | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.365 s | 0.383 s | 1.049 | 17 | 17 | 9/9 |
| `easy-win` | 0.080 s | 0.078 s | 0.975 | 15 | 15 | 9/9 |
| `deep-win` | 4.902 s | 4.349 s | 0.887 | 22 | 22 | 9/9 |

Geomean over three claimed cells: **0.968**. This is a candidate `P5` matrix;
plan3 owns the parity gate and the ledger row at 10 s.

## Reading notes

- The candidate's `child_evals` counts every child scored, not provari's
  `nodes`; the two are not comparable one-to-one.
- The memory split depends on the cap: `proofs_needed(seconds)` gives the
  proof table more room at 120 s than at 10 s. That is why `deep-win` finds a
  19-move proof at 120 s and a 22-move proof at 10 s. The 10 s row is the
  like-for-like parity comparison.
- No raw transcripts, oracle binaries or proofs are committed.

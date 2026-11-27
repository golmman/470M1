# Bench harness

The performance ledger, its metric and reading rules are
[`../docs/bench.md`](../docs/bench.md). This directory holds the corpus, the
pinned `high200` file, the pre-registered sets and the driver.

| Path | Purpose |
| --- | --- |
| `corpus.json` | `corpus-v1` (`core` + `sample20`), copied from atomi/atomic_solver; sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e` |
| `high200.jsonl` | provari `b87d179`'s 200 engine-rated wins, copied verbatim; sha256 `857496451612904ef4eddc77a93ae4328f00d603b7755c49b8ae114d89682af6` |
| `sets/goal-v1.json` | selection result for the goal column (reference `model`, `N` threads, tablebases) |
| `sets/matched-v1.json` | selection result for the matched column (reference `model`, one thread, no tablebases) |
| `sets/guard.json` | the three historical hand-guide cells (`core`, `easy-win`, `deep-win`) |
| `run_bench.py` | the driver: calibration, selection, rows and claims |

## The three columns

| Column | Reference | Candidate | Books a factor? |
| --- | --- | --- | --- |
| `guard` | `provari prove --guide hand --threads 1` | `470MI prove --guide hand` | yes; the historical 1.000 side |
| `matched-v1` | `provari prove --guide model --threads 1`, no tablebases | `470MI prove --guide hand` | yes; the product factor with features equal apart from the guide |
| `goal-v1` | `provari prove --guide model --threads N --tablebases 3-4-5` | `470MI prove --guide hand` | reference-only coverage (our checker cannot check tablebase verdicts) |

All three use the canonical cap 10 s and 192 MB per search, three interleaved
repeats per cell per side, one process at a time, same session, same host.
The candidate has no threads or tablebase flags yet, so `matched-v1` is its
row; `goal-v1` sizes the distance in a footnote.

## Protocol

```
cargo build --release
make tablebases                     # /tmp/470M1-oracle/tablebases/3-4-5
make oracle                         # /tmp/470M1-oracle/provari, v0.5.0
make bench-calibrate                # chooses N for this host (env.json)
make bench-sets N=2                 # writes bench/sets/{matched,goal}-v1.json
make bench N=2                      # matched row, 3 repeats, claims
make bench-goal N=2                 # goal row / distance
make bench-guard                    # three-cell hand guard
```

- `N`: `Nmax = min(8, nproc)`; for `N in {1,2,4,8} ∩ [1,Nmax]` ascending the
  driver runs the eight calibration cells (`high200` 1-based lines 3, 23, …,
  143) three times each. `r(N)` is the geomean of the per-cell median walls;
  the largest `N` with `r(N)/r(1) <= 0.8` and `r(N)/r(prev) <= 0.9` wins,
  else `N = 1`. Calibration cells are excluded from every set forever.
- **Selection** precedes any candidate run and uses the reference only, one
  repeat per cell; a fast win (`< 9.0 s`) joins at 1/1, a slow win is retried
  and needs 2 of 3, two failures eliminate the cell. Members are sorted by
  cell id; every 5th is `held-out`, the rest is `strong`; a row measures
  `strong` only.
- Sets are written once: `--mode select` refuses to overwrite an existing
  `--set-out`; a changed command, `N`, cap or tablebase set makes a new
  `-v2` set file.
- **Row**: interleaved `ref,cand` per repeat; medians; spread
  `(max-min)/median`; a reference spread `>= 20%` marks the cell `unstable`
  and books no factor.
- **Claim**: one `--proofs` run per claimed cell per side, never feeding a
  factor. Candidate proofs pass `470MI check` **and** `provari check`;
  reference proofs pass `provari check` (with tablebases in the goal
  column). A disagreement is a veto, not a row.

The driver gates the digests of `corpus.json`, `high200.jsonl`, the set file,
the tablebase manifest and the pinned reference version, records both binary
sha256s and the data digests in `env.json`, and refuses to run if any differ.
`--expect-set-sha256`, `--expect-provari-sha256` and
`--expect-candidate-sha256` pin a re-run to the set file and the two binaries
a row recorded (their digests are in `docs/bench.md` and `env.json`).

`--side cand --baseline-bin PATH` compares two candidate binaries without a
provari reference: each repeat of each cell runs `base` (the candidate
command with the baseline binary) then `cand`, interleaved. The summary is
the same shape with `base` for `ref`: claimed iff both sides found on every
repeat and the base spread is under 20%, `factor = cand / base`, and per
cell `counters_identical` says whether all found runs share
(`mate`, `nodes`, `child_evals`, `expansions`). Without `--claim` no provari
is needed: its gate is skipped and `env.json` records `provari` null plus
`baseline` and `baseline_sha256`.
Every run writes `results.json`, `summary.json`, `env.json` and `README.md`
under the measurement directory; claims add `claims.json` and `proofs/`.

## Oracle

`make oracle` builds provari `v0.5.0` (`b87d179`) with `--features bundled`
in `/tmp/470M1-oracle`, never in the repository. `make tablebases` fetches
and md5-verifies the 3-4-5 `.atbw` set. A missing oracle makes the veto red,
not skipped.

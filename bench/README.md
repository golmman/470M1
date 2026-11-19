# Bench harness

The performance ledger is [`../docs/bench.md`](../docs/bench.md). This
directory holds the corpus and the reference-side driver.

| Path | Purpose |
| --- | --- |
| `corpus.json` | `corpus-v1`, copied from atomi/atomic_solver; sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e` |
| `run_bench.py` | Runs the pinned provari reference side and writes parsed results plus `env.json` |

## Protocol

- Reference: `provari prove "<FEN>" --seconds 10 --memory 192 --guide hand`.
- Candidate: `470MI prove "<FEN>" --seconds <cap> --memory 192` (movegen plan3).
- Three repeats per cell per side, one process at a time, same session.
- The driver refuses a corpus whose sha256 does not match.
- The driver writes parsed results, not raw transcripts, under
  `docs/plans/<initiative>/measurements/planN/`.
- A ledger row is appended only after the plan's correctness gates pass.

## Oracle

`make oracle` builds provari at the pinned commit in a scratch directory
outside the repository. The oracle is never committed. A missing oracle makes
the veto red, not skipped.

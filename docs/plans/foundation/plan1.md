# Plan 1 — F1: bench v2, the full-feature reference, the first row

**Item:** F1 (Bench v2). **Session:** plan → execution. **Status:** proposed
(2026-11-24). **Successor:** [`report1.md`](report1.md).

## Goal

Re-pin the oracle to the latest provari release, all features on (learned
guide, tablebases, `--threads N`); fix `N`, the tablebase set and the canonical
cap in [`bench.md`](../../bench.md); pre-register `strong`/`held-out` over
`corpus-v1` + `high200`; rebuild the driver; append the first same-session row
of the current product, hand guard row beside it. No search, guide, proof,
checker or default change.

## Probes (read-only, 2026-11-24)

| Fact | Value |
| --- | --- |
| Release | provari **v0.5.0** `b87d179c01ef8eae92a8ae93e4ca8b3df924dd71`, 2026-11-22, GPL-3.0-or-later |
| Data | `network.nnue` `3d76258c556ce216d088f89ff094dadab4310e3838110c64f66049b6ab5c40b1`; `model.json` `51be8f472304ce535a385ad88906c75a976e5c8d2b267162bd9818a59231882b` |
| `high200` | 200 cells, sha256 `857496451612904ef4eddc77a93ae4328f00d603b7755c49b8ae114d89682af6`; the 20 `sample20` FENs unchanged, `core` absent → universe 201 |
| Tablebases | `.atbw` `3-4-5` (~0.4 GB), manifest sha256 `a0c7a69bc720dae8c77a776810d2f33486c153655cebdef229e27a40fdeefc37`; `6-wdl`/`6-dtz` out |

## Columns

The product cannot run the goal configuration yet: plan1 measures a **matched**
column and sizes the goal beside it. Reference runs add `--memory 192 --seconds
10 --seed 0 --json`.

`guard` adds `--guide hand --threads 1` (hand = 1.000); `matched-v1` uses
`--guide model` without `--tablebases` (the product factor, features equal);
`goal-v1` is `matched-v1` plus `--threads N --tablebases DIR` (`PD1`'s
reference, whose distance plan1 reports).
Candidate: `470MI prove FEN --seconds 10 --memory 192 --guide hand --json`.
Canonical **cap 10 s**, **memory 192 MB** per search both sides (`N` threads
share it); 60 s is a later column version. Tablebases only in `goal-v1`: a
reference proof with `tablebases > 0` is uncheckable by ours — reference-only
coverage, no factor.

## N per host

`P = nproc` (affinity/container aware), `Nmax = min(8, P)`. Calibration cells:
`high200` 1-based lines 3, 23, …, 143 (8, none in `corpus-v1`), excluded from
every set forever. For `N ∈ {1,2,4,8} ∩ [1,Nmax]` ascending run `goal-v1` 3×
per cell; `r(N)` = geomean of median walls. Take the largest `N` with
`r(N)/r(1) ≤ 0.8` and `r(N)/r(prev) ≤ 0.9`, else `N = 1`. Record `P`, `Nmax`,
all `r(N)` and the choice in `env.json`: `N` is part of a row, per host.

## Bench sets

**Universe** = `core` + 200 `high200` = 201, minus the 8 calibration cells =
**193 eligible**. **Selection** precedes any candidate run, per column set,
`--guide model --seed 0`, one repeat; retry twice only if not proved or proved
at `≥ 9.0 s`; joins iff proved in `≥ 2 of 3`; a 1-of-3 cell is flaky and out.
Run twice: `goal-v1` (reference `N`, tablebases) and `matched-v1` (`N=1`, none);
each gives the product's `strong`; a row uses only its own column's set.

**Held-out:** members sorted by cell id, every 5th (1-based ≡ 0 mod 5); the rest
is `strong` — content-blind, fixed with the set, never tuned on. Sets are
`bench/sets/goal-v1.json` and `matched-v1.json`, sha256 in `bench.md`,
driver-gated. A changed command, `N`, cap or tablebase set makes `-v2`; sets are
never edited.

## Driver, scripts, provenance

1. `bench/run_bench.py` v2 adds `--side --bin --set --threads --tablebases
   --network --model --claim` to today's flags; gates on corpus-v1, `high200`,
   the set file, the tablebase manifest, the reference `--version` ("with its
   network and model built in") and both binary sha256s. Interleave the sides per cell
   (`ref,cand,…`); medians and spread; parse the result line. Writes
   `results.json`, `summary.json` (claimed list, coverage, per-cell factor,
   geomean, bounds) and `env.json` (host, `nproc`, rustc, binaries, oracle
   version, data/tablebase digests, `N`, cap, memory), plus `README.md`. Claim
   runs use `--proofs`, never feed a factor, record both checker verdicts.
2. `bench/high200.jsonl` verbatim from the pin, one `PROVENANCE.md` row;
   `bench/README.md` to the three columns. `scripts/build_oracle.sh`: pin
   `b87d179`, `data/fetch.sh`, build `--release --features bundled`, assert the
   version string, record binary sha256 and data digests in `version.txt`;
   oracle stays in `/tmp/470M1-oracle`, never committed.
   `scripts/fetch_tablebases.sh` (new): `3-4-5/*.atbw` + `.md5` into
   `/tmp/470M1-oracle/tablebases/3-4-5`, verify every md5 and the manifest;
   failure aborts the row. `Makefile`: `bench` = matched row, `bench-goal`,
   `bench-sets`, `tablebases`.
3. `PROVENANCE.md`: v0.5.0 replaces v0.4.0 as the current reference (old row
   kept), plus the data pins and the `high200` copy row.

## `bench.md` sections

*Metric*: factor = candidate / the row's reference column; a column owns its
command, `N`, tablebases, memory and cap; geomeans only within one. *Protocol
and set*: pin, bundled build, data digests, tablebase manifest, set files, cap,
memory, `N` rule, interleaving, repeats, universe, selection, held-out split,
exclusions;
delete the three-cell definition and its exclusion rows. *Ledger*: `Column |
Set | N | TB | Cap`, the two plan1 rows, footnotes (coverage, censored, length
vs `mate`, distance). *Pins*: v0.5.0 and data pins; *History*: one entry.

## The first row (one session, one host)

Preflight (`make test-full`, tree sha) → oracle → tablebases →
calibration → selection → rows → claims → guard → ledger.

On `strong-matched-v1`: matched reference and candidate, 3 interleaved repeats,
10 s, `N=1`, 192 MB, no tablebases. On `strong-goal-v1`: `goal-v1` 3 repeats
plus the candidate's runs (reused where sets overlap) → the distance table.
Then one candidate and one reference `--proofs` run per claimed cell, checked by
both/`provari` respectively.

Row A (`matched-v1`): candidate/matched geomean over claimed cells, with
coverage, bounds, `child_evals`, length beside the same-session reference
`mate`. Row B (`guard`): the three cells, both sides `--guide hand`, 3 repeats;
its geomean sits beside movegen plan 12's 1.001 as a cross-session diagnostic
only. Footnote: goal coverage and walls, the distance and caveat, set digests,
`N`, host identity.

## Gates (pre-registered)

1. **Correctness:** every candidate claim passes `470MI check` and `provari
   check`; every reference proof in a factor cell passes `provari check`; a
   disagreement is a veto — `not claimed (<verdict>)`, and the plan fails.
2. **Determinism:** the three candidate runs per cell at `N=1` have identical
   `nodes`, `child_evals`, `mate`; else the row is blocked as a finding.
3. **Set validity:** selection precedes any candidate run; digests and logs are
   committed; `|strong| ≥ 10` per set, else the row states the size and makes
   no geomean claim.
4. **Measurement:** 3 repeats, medians, spread; cap, `N`, memory, tablebases and
   coverage stated with the geomean; a reference spread `≥ 20%` makes the cell
   `unstable`, no factor. The guard geomean must be `≤ 1.05` vs hand; a
   regression there is a finding, not a bench result.
5. **Scope:** no search/proof/checker/default change; `make test-full` passes.

## Non-goals, risks, blind spots

No threads, learned guide, tablebase or proof-database work in the prover; no
tuning against `strong` or `held-out`; no new cap; earlier rows unchanged.

- `v0.5.0` may not build locally → stop and report; never fall back silently.
- The matched column is blind to thread and tablebase levers; the goal probe
  sizes them once (pessimistic for the candidate).
- `high200` is White-to-move, engine-rated wins; the split is no position-type
  sample and membership is cap-boundary sensitive.

## Tasks (one session, ~3 h)

1. Oracle/tablebase scripts, `PROVENANCE.md`, driver v2, sets, `Makefile`,
   `bench/README.md` (~90 min). 2. Preflight, calibration, both selections
   (~50 min). 3. Row, goal probe, guard, claims (~50 min). 4. `bench.md`, ledger
   rows, status line, [`report1.md`](report1.md), `measurements/plan1/` (~30 min).

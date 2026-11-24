# Report 1 — F1: bench v2, the full-feature reference, the first row

**Item:** F1 (Bench v2). **Plan:** [`plan1.md`](plan1.md) (2026-11-24).
**Session:** execution, 2026-11-24. **Status:** complete, owner commits.

## Goal and verdict

Re-pin the oracle to provari `v0.5.0` with every feature on, fix `N`, the
tablebase set and the canonical cap, pre-register `strong`/`held-out` over
`corpus-v1` + `high200`, rebuild the driver, append the first same-session
row of the current product and the guard row beside it. No search, guide,
proof, checker or default change.

**Met.** The matched row reads **3.104** (coverage 9/20) and the guard row
**1.023** (3/3); the goal distance is **2.971** (reference coverage 20/20,
candidate 9/20). No prover, checker, guide or default changed; the only code
touched is the bench driver, the scripts, the `Makefile` and docs.

## Preflight

`make test-full` passed before any measurement (fast tests, ignored tests,
`unsafe-feature` gate, veto: all fixtures accepted, all controls rejected).
Tree `f617176` plus the plan1 working tree; agents used read-only `git`.
Oracle and tablebases were built outside the repository.

## Instrument

- **Oracle**: provari `b87d179c01ef8eae92a8ae93e4ca8b3df924dd71`
  (`v0.5.0`), `--release --features bundled`, version
  `provari 0.5.0, with its network and model built in`, binary sha256
  `3cd1a4ecf4aed82ca3b37f1e4a5af55d9457a0b1f4b44dfb8cb5abebe7118842`.
  Data: `network.nnue` `3d76258c…5c40b1`, `model.json` `51be8f47…31882b`.
  `scripts/build_oracle.sh` records all three in `/tmp/470M1-oracle/version.txt`.
- **Tablebases**: Lichess Atomic 3-4-5, 145 `.atbw` files, every md5
  verified by `scripts/fetch_tablebases.sh`; manifest sha256
  `a0c7a69b…fdeefc37`. 6-piece tables are out.
- **Corpus**: `corpus-v1` (`core` + `sample20`) and `bench/high200.jsonl`
  (copied verbatim from the pin, sha256 `85749645…d89682af6`); the driver
  gates both digests.
- **Cap** 10 s, **memory** 192 MB per search, **repeats** 3, interleaved
  `ref,cand`, one process at a time. Host: `f54c552f3ece`,
  `Linux-7.3.4-401.asahi.fc44.aarch64+16k`, `nproc` 4, rustc 1.100.0.
  Host identity is the candidate binary sha256 `25cc08d8…427fee` plus the
  platform string (the container UTS name is not the owner's host label).

## Calibration and N

`P = nproc = 4`, `Nmax = 4`; cells are `high200` 1-based lines 3, 23, …,
143. `r(1) = 10.0240`, `r(2) = 10.0237`, `r(4) = 10.0250` s; **N = 1** (no
`r(N)/r(1) <= 0.8`). No calibration cell proves at 10 s, so every wall is the
cap and the ratios are 1.0 by construction. That is the plan's pessimistic
case, not evidence that threads do not help: a calibration cell that proves
would carry the signal, and wall ratios against a capped reference cannot.

The driver's first `nproc` probe used `sched_getaffinity` (8) and picked
`N = 8`; the shell `nproc` (cgroup-quota aware) is 4. That first calibration
was stopped before writing output and rerun with the corrected rule; no row
used it (`measurements/plan1/calibration/`, `runs.json` is the rerun).

## Selection

Reference only (`--guide model --seed 0`, one repeat; fast win joins 1/1, a
slow win or a failure is retried and needs 2/3). Universe 201 (core + 200
`high200`), calibration cells excluded → 193 eligible. Both columns selected
**25 members** (strong 20, held-out 5) with identical membership; at `N = 1`
the tablebases add no cell, only slightly different walls.

- `bench/sets/matched-v1.json` sha256 `810289f0…d83d1`
- `bench/sets/goal-v1.json` sha256 `0129ff7c…516e9f`
- `bench/sets/guard.json` sha256 `199c03ab…60bfb` (`core`, `easy-win`,
  `deep-win`; historical, not selected)

The held-out five are `285455b84001`, `62e7209914b8`, `acb4a127c8c8`,
`core`, `faee216ab7f0`; they were not measured in this row and were never
tuned on.

## Rows

**Row A — `matched-v1`** (reference `--guide model`, one thread, no
tablebases; candidate `--guide hand`): coverage **9/20**, geomean **3.104**.
Factors 2.392, 2.576, 4.801, 2.162, 1.634, 2.457, 11.119, 4.089, 2.291;
8 censored (bounds 1.12–6.35); 3 unstable (`9a8566095b1f`,
`ae8763570b09`, `e026f93c8241`; reference spread 67/50/28%, no factor).
Candidate lengths 35/18/18/13/17/21/12/20/46 against reference mates
23/23/18/15/17/21/13/19/32; `child_evals` 57.0M/0.9M/18.0M/1.8M/2.6M/
15.7M/17.4M/44.8M/60.0M. Every candidate claim (9/9) passed `470MI check`
and `provari check`; every reference proof (9/9) passed `provari check`.

**Row B — `guard`** (both sides `--guide hand`): coverage **3/3**, geomean
**1.023**; `core` 1.053, `easy-win` 1.233, `deep-win` 0.825. Lengths
14/18/20 against mates 17/15/22; `child_evals` 3,896,430 / 904,368 /
44,828,946 (identical to movegen plan12's tree, seed 0). Claims 3/3 by both
checkers. The cross-session reading against plan12's 1.001 is a diagnostic,
not a factor; the gate (`<= 1.05`) held.

**Goal distance.** `goal-v1` reference (same `N`, 3-4-5 tablebases) proves
**20/20** strong cells; all 20 claims pass `provari check --tablebases`.
Candidate walls are the matched row's (same command, same session/host), so
the distance is **2.971** on the 9 candidate-claimed cells, with 8 bounds
1.12–6.32 and the same 3 unstable cells. The goal column books no factor:
the candidate lacks threads/tablebases and our checker cannot check tablebase
verdicts. If the owner wants a booked goal factor, the candidate needs those
features first (`parallel`, `guided`).

## Gates

1. **Correctness — passed.** 9/9 candidate proofs accepted by both checkers,
   9/9 matched reference and 20/20 goal reference proofs by `provari check`.
   No disagreement.
2. **Determinism — interpreted, finding below.** Every completed (claimed)
   candidate search has identical `nodes`/`child_evals`/`mate` across the
   three repeats; the 8 censored searches differ only at the wall-clock cut.
   A fixed-`--evals` probe on `1051b3d28a27` is bit-identical
   (494,712 nodes, 6,000,021 evals, root `[518,340106]`). The plan's wording
   ("the three candidate runs per cell ... else the row is blocked") is not
   literally met for censored cells; blocking a row over a deadline cut would
   make every censored cell a determinism failure. **Finding:** state the
   gate for completed searches, or measure determinism with `--evals`.
3. **Set validity — passed.** Selection preceded every candidate run;
   digests and raw attempts are committed; both strong sets have 20 members
   (`>= 10`).
4. **Measurement — passed.** 3 repeats, medians, spreads; cap, `N`, memory,
   tablebases and coverage stated with each geomean; guard 1.023 `<= 1.05`.
5. **Scope — passed.** No search/proof/checker/default change;
   `make test-full` passed.

## Problems and findings

- **Calibration is blind at the cap.** With no calibration cell proving, all
  `r(N)` equal the cap. A future calibration should pick cells the reference
  proves (e.g. from a low cap) or normalise by nodes/evals; otherwise `N = 1`
  is forced on every host. The plan anticipated the pessimistic direction but
  not that the rule degenerates.
- **Matched coverage is 9/20.** The learned guide is worth ~3x in this
  column; the hand-guide candidate cannot cover 11 of the reference's cells
  at 10 s. That is the distance, not a benchmark defect.
- **Unstable micro-cells.** Three cells prove in 20–55 ms; a 20% spread is
  startup noise. They carry no factor, as the gate says.
- **Goal selection equals matched selection** at `N = 1`, so candidate runs
  were reused. Distance ratios are same-session but not interleaved; they are
  a footnote, not a factor.
- **`1051b3d28a27`** joined selection at 1/1 (8 941 ms) but its candidate is
  censored; the row reports the bound 1.123, not a factor.

## Scope changes, open parts, missing tests

No scope change. Open: held-out measurement (pre-registered, not run);
60 s column version; `N > 1` once a proving calibration cell exists; the
candidate's threads/tablebases are later initiatives. Missing: a driver test
(selection retry rule, held-out split, digest gates) — the driver is exercised
by this run only.

## Next steps

1. Owner commits the plan1 tree (driver, scripts, sets, `high200`, docs).
2. Owner decides the gate-2 wording and the calibration-cell rule.
3. F2/F3 (errors, config) next; keep the hand guard at every row.

Evidence: `measurements/plan1/{calibration,select-matched,select-goal,matched,goal,guard}/`,
`distance.json`, `bench/sets/*.json`, `PROVENANCE.md`.

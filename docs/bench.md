# Bench — performance ledger vs. provari

Status: **standing document, independent of any initiative.** It fixes the
bench set, the metric and the reading rules, and appends one row per executed
plan, each at the cap it was measured at. It is not a plan and not a report:
plans live in `docs/plans/<initiative>/planN.md`, their evidence in
`reportN.md` and `measurements/planN/`.

| If you want | Read |
| --- | --- |
| the goal, the rungs and their budgets | the active initiative, e.g. [`plans/parity/initiative.md`](plans/parity/initiative.md) |
| the reasoning behind a number or a convention | the initiative's `decisions.md` |
| what happened, plan by plan | the initiative's `history.md` and the plan's `reportN.md` |
| to measure or re-run a row yourself | [`../bench/README.md`](../bench/README.md) |

This file states no goal, no rung and no cap policy: it records what was
measured, at the cap each row used, and makes no goal claim.

## The metric

```
factor(cell) = median candidate wall [s] / median provari --guide hand wall [s]
```

- **provari `--guide hand` is 1.000 by definition** — the fair
  hand-evaluation-for-hand-evaluation column. The bundled model column
  (`--guide model`) is out of scope and never mixed in.
- **`< 1.0` means the candidate is faster than provari on that cell.**
- **Ledger score** = geometric mean of `factor(cell)` over the row's
  *claimed cells* — cells where both sides prove a win at the row's `Cap` and
  the claim passes the checker protocol. Unclaimed cells have no factor: they
  count in **coverage**, not in the mean, and the mean names its cell count.
- **Censoring.** A cell that is not a claimed win at **the row's own cap** is
  *censored*; a meaningful bound is printed as `>= cap / reference_wall`, a
  *lower* bound on the factor, never a factor. A run that reaches a mate whose
  claim fails (e.g. an export refused as `store-full`) is *not claimed
  (`<verdict>`)* with the found wall. A censored cell is never dropped and
  never converted into a win.
- **`--guide none` is a standing diagnostic control**, never a column: it
  isolates the guide's contribution and, with proof length as the axis,
  separates a badly ordered guide from genuinely long proofs. It is never a
  bench cell and never a factor.
- **Rungs are not a column.** The active initiative decides which rung a set
  of measurements reaches; rows keep their own session and are never
  rewritten.
- **Deterministic accompaniment.** Every row also records `child_evals` (the
  candidate's, deterministic per stack) and provari's `nodes` for the same
  cell. They are not comparable one-to-one: evals per expansion against
  provari's nodes is a diagnostic, not a ratio to optimise.
- **Proof length is a companion column, and it is compared.** Every row
  records the candidate's `length` (attacker moves) beside the *same-session*
  reference `mate` for that cell, and states the delta. It is not a factor — a
  longer proof is a sound proof — but it is the tree's depth, and depth drives
  expansions superlinearly. A row that cannot state both numbers is
  incomplete.
- **A measurement states what it cannot see.** Any census, profile or
  differential row names the structure it is blind to, so the next agenda is
  not the previous instrument's field of view.

### Measurement protocol

| Item | Value |
| --- | --- |
| Reference side | `provari prove "<FEN>" --guide hand --seconds 10 --memory 192`. Fixed: the set is selected so the hand guide proves inside 10 s, so the reference cannot move with a candidate cap |
| Candidate side | `470m1 prove "<FEN>" --seconds <cap> --memory 192 [--seed N] <stack flags>`; `--memory` covers the TT, the proof table and the filter |
| Caps planned | reference 10 s always; candidate at the row's `Cap` (10 s, 120 s, 600 s) |
| Repeats | 3 per cell per side, medians reported, one process at a time |
| Session | a row's reference and candidate runs are measured back to back on one host |
| Oracle | provari `v0.4.0` at `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`, rebuilt from source per row, never committed |
| Corpus | `corpus-v1` = `bench/corpus.json`, sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`; the driver refuses any other digest |
| Tablebases | none |
| Stretch column | none in the parity initiative |

The driver will be [`../bench/run_bench.py`](../bench/README.md) (flags, sides
and the derived ledger there), created by plan1. Until then, a plan may use
its own driver under `measurements/planN/` and record that in its report.

## The bench set

Three cells, drawn from `corpus-v1` (`core` + the pinned `sample20`).
**Selection rule (fixed):** the bench set is every cell of `core + sample20`
where the pinned provari oracle proves a win **with the hand guide** inside the
canonical budget. A cell where provari itself is censored cannot carry a factor
and is not a bench cell, however interesting it is.

| Short name | Cell id | FEN | hand wall[^ref] | hand nodes | mate | Why this cell |
| --- | --- | ---: | ---: | ---: | --- | --- |
| `core` | `core`[^coreid] | `r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1` | 0.49 s | 342,289 | 17 | the motivating FEN; here the hand guide *helps* provari |
| `easy-win` | `1eb80b92f92c` | `rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1` | 0.11 s | 72,523 | 15 | the one easy shared win; here the hand guide *hurts* provari (>2x), so the lever is case-dependent |
| `deep-win` | `db6a741c723f` | `rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1` | 6.20 s | 4,574,912 | 22 | the deepest hand-guide win in the corpus; the coverage cell |

[^ref]: **Provisional** values inherited from atomi's plan1 set card. Every
    row **re-measures** its own reference in the same session; never take a
    reference from this table for a new factor. plan1 of `parity` records the
    first same-session reference values.

[^coreid]: `core` is the one hand-named cell — it is not in `high200.jsonl`.
    Its corpus fingerprint would be `8d0ff8bca7f0`, so a measurement file
    showing that id means this position.

Cells measured and deliberately **out** of the set, so the exclusion is not
silently revisited:

| Cell id | Why not a bench cell |
| --- | --- |
| `b9a7ed7eb368`, `dd3f89766a40` | provari **hand** censored (10.1 M / 9.3 M nodes); only the out-of-scope bundled model wins them |
| the other 16 `sample20` ids | provari's model is censored on all of them, so the hand guide is expected to be censored too |

If a later census finds an additional hand-guide win, that cell joins the set
by the rule above — short name, measured reference, a new reference row.
Earlier rows are not rewritten.

## Names and ids (read this first)

Three kinds of identifier appear here and in the measurement files; **only one
of them is a git commit**.

| Kind | Example | Meaning |
| --- | --- | --- |
| Cell short name | `core`, `easy-win`, `deep-win` | use this in prose, plans, reports and ledger rows; defined in [The bench set](#the-bench-set) |
| Cell id | `1eb80b92f92c` | provari's record id in its corpus file — **not** a git hash: the first 12 hex of `sha256(FEN)` at the pinned commit, unique inside that file. Tooling and `corpus.json` key by it |
| Commit pin | `470m1@<sha>`, `provari-v0.4.0` | a real commit: the oracle build, or the tree a row was measured at; full sha in the [pins](#pins) table |

Rule of thumb: prose uses short names, tooling uses cell ids, rows record the
commit pins. A cell id is stable forever; a short name is stable as long as the
cell is in the set.

## Reading rules (do not violate these when appending a row)

1. **Same session, same host.** A factor compares two walls measured back to
   back on one host. Cross-host precision is not enough to call a lever.
2. **Wall is the product metric; `child_evals` is the mechanism metric.** A
   child-eval cut that does not move the wall is a mechanism finding, not a
   speed win — and a wall win at identical evals is a per-node cost effect.
3. **Censored is a result — at the row's cap.** `>= cap/reference` bounds and
   coverage counts are the honest form. A longer run is **its own row** with
   its own `Cap`; a cap is never chosen to drop a difficult cell (cap
   shopping, rule 6).
4. **Opt-in is not shipped.** The `Stack` column is part of the result: only
   a default-flag decision (a passed stack gate) turns a stack row into a
   product row. Until then every stack row stays visible.
5. **Correctness gates precede the number.** A row is written only after the
   plan's gates passed and, for new proof shapes, after the claim protocol
   accepted the exported proof.
6. **Coverage is stated in every row, and geomeans are compared only within
   one cap.** State `(cap, coverage, geomean)` together. The geomean is
   **non-monotone in the cap** — a longer cap claims more cells, hence more
   factors in the mean — so a cross-cap comparison is a reading error and a
   cap picked per plan to improve the headline is cap shopping. A rung is
   reached only through what the active initiative says.

## Ledger

**Convention.** The ledger is a **results history**: one row per executed
plan, each with the budget it was measured at. `Cap` is part of the result, so
`censored` always means "not claimed at this cap". A factor itself needs no
budget — it is `candidate wall / provari hand wall`, and the reference is
re-measured per row inside its own 10 s budget. Decisions append no row.

### Factors

| Plan | Date | Host | Stack | `core` | `easy-win` | `deep-win` | Geomean | Cap |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| plan2 | 2026-11-17 | `8f5ca2f2750a` | hand, single-thread | **1.091** | **0.987** | **0.887** | **0.985** | 120 s |
| plan3 | 2026-11-17 | `8f5ca2f2750a` | hand, single-thread | **1.062** | **0.988** | **0.854** | **0.964** | 10 s |


Plan2 footnote: coverage 3/3, seeds 0–2, three repeats, every proof accepted
by `470m1 check` and `provari check`. Candidate proof lengths 17/15/19 against
same-session reference mates 17/15/22; `child_evals` 3,823,775 / 897,937 /
38,205,943. A same-session 10 s matrix (not this row) gave factors 1.049 /
0.975 / 0.887, geomean 0.968, 9/9 accepted per cell; plan3 owns the 10 s gate.
See `plans/parity/measurements/plan2/`.

Plan3 footnote: coverage 3/3, seeds 0–2, three repeats, every proof accepted
by `470m1 check` and `provari check` (9/9 per cell). Candidate proof lengths
15/20/20 at the median-seed run against same-session reference mates
17/15/22 (seed 0 found 17/15/22); `child_evals` 4,160,024 / 906,543 /
56,461,476, the counters of the same median-seed run. **This row is one of
four same-session 10 s passes** (0.992 / 1.013 / 0.974 / 0.964, median 0.983):
the reference's own wall drifted ~10% between passes, so the 10 s gate is at
the edge of the instrument's resolution. The same pass under two other
protocols: both sides exporting a proof 0.988, and search wall only 0.918. A
120 s cap row reads 0.975 (3/3; `deep-win` 0.831).
See `plans/parity/report3.md` and `plans/parity/measurements/plan3/`.

Cell syntax: a **bold number** is a claimed factor (candidate wall / provari
hand wall, both from the row's own session). `censored` means no claimed win
at the row's cap. A seed fraction (`1/3`, `2/3`) means proved on that many of
seeds 0–2, hence censored and carrying no factor; `not claimed` means the mate
was found but the claim failed the checker protocol (verdict in the row's
note). Walls, `child_evals`, proof lengths and checker verdicts of each row
belong in its footnote or in the plan's `measurements/planN/README.md`.

### Pins

| Pin | HEAD | Rows |
| --- | --- | --- |
| `provari-v0.4.0` | `bceb1ee1faf7d9cf2422eb89c931be69ae30619d` | the 1.000 side of every factor |
| `470m1@<sha>` | _owner commits the plan2 tree_ | plan2 |
| `470m1@<sha>` | _owner commits the plan3 tree_ | plan3 |

## History

Chronological entries — including every plan session that appended no row —
are in the active initiative's `history.md`. Only ledger-structure changes are
kept here:

- **2026-11-17** — file opened for the `parity` initiative. The bench set,
  metric and reading rules are inherited from atomi's ledger; the reference
  values are provisional until plan1 re-measures them. No rows yet.

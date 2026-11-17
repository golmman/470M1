# Report 4 — P3 hardening

**Plan:** [`plan4.md`](plan4.md). **Item:** 4. **Date:** 2026-11-17.
**Outcome:** all gates green. The D8 seed now perturbs only exact tie-breaks;
seed 0 is unchanged. CLI allocation, thread and flag failures are errors, exit
codes and help are truthful, and the ignored 10 s bench-cell test is wired
into `make test-full`. One corrected-seed 10 s matrix was run; it is a
regression row (geomean 1.016), not a speed claim. No `movegen` work started.

## Phase 0 — audit baseline

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | 44 tests, 5.6 s wall: lib 1.66 s, checker 0.28 s, prover 3.53 s |
| `make test` | green |
| `make test-full` | green (old form: tests + veto) |
| `make veto` | 9/9 accepted, 1/1 control rejected by both checkers |
| `cargo doc --no-deps` | green |

User-facing panic audit before the change:

- `prove` called `Dfpn::new`, whose allocation failure panics.
- `check::run` had `expect` on thread spawn, `unwrap` on the result mutex and
  `expect` on every result slot.
- Numeric flags were parsed but not validated: non-finite/zero/negative
  `--seconds`, zero/oversized `--memory`, zero `--evals`, zero or huge
  `--workers`, zero `--repeats`, zero `--until`.
- `src/main.rs` and `src/lib.rs` headers still described the plan1 stubs.
- No stale provenance row: the only changed source is our own `dfpn.rs`; no
  dependency was added.

## Phase A — D8 seed correction

`Dfpn::perturb` scaled the guide step by ±1%, which reordered subtrees. It is
removed. `combine` now receives the seed and, only after deciding number and
history both tie, picks the lower hash `tie(child.hash, seed)`. Seed 0 gives
tie 0 and therefore exactly the old first-child choice.

Tests added in `src/dfpn.rs`:

- `a_seed_breaks_only_exact_ties`: equal numbers/history can be reordered by a
  seed; seed 0 keeps the first child.
- `seed_zero_keeps_the_core_tree_unchanged`: a 100,000-node `core` budget is
  pinned to `(None, 100_016, 1_161_335, 57_100, (370, 24_825))`, so a future
  guide or tie-break change is caught.
- `seeds_are_deterministic`: seeds 1–2 repeat their counters.
- `a_seed_does_not_change_a_proof`: seeds 1–2 prove `core`; both proofs pass
  our checker.

## Phase B — errors and CLI

- `prove` uses `Dfpn::try_new`; allocation failure is `error: the search's
  memory: ...` and exit 2.
- `check::run` returns errors for thread-spawn failure, a poisoned result
  mutex, a panicked checking thread and a missing result. `--workers 0` is
  rejected; the range is 1–1024.
- Validated flags: `--seconds` finite, positive and representable; `--memory`
  1–1,048,576 MB; `--evals >= 1`; `--workers 1..=1024`; `--repeats 1..=1024`;
  `--until >= 1`; negative seeds fail. Messages name the flag and range.
- Exit codes are documented in `--help`: 0 success, 1 invalid checked proof,
  2 usage or other failure.
- `--help` now lists `--repeats` and `--version`, says `--seed` perturbs
  tie-breaks only, and carries the exit-code table. Stale headers fixed.

## Phase C — tests and gates

- `tests/bench_cells.rs` (`#[ignore = "slow: ..."]`) proves `core`,
  `easy-win` and `deep-win` at 10 s / 192 MB twice each, exports and checks
  both proofs, and asserts identical counters. It takes ~14 s in debug.
- `tests/cli.rs` covers bad numeric flags, missing arguments, both proof
  verdicts and the documented exit codes, and rejects any `panicked` output.
- `make test-full` is now `test` + `cargo test -- --ignored` + `veto`;
  `make test` stays fast; `make veto` is unchanged.
- Licence gate green; no new dependency; `PROVENANCE.md` needed no row.

## Corrected-seed 10 s regression

One interleaved session, protocol a, seeds 0–2, three repeats, 10 s/192 MB.
Coverage 3/3, every claim accepted by `470m1 check` and `provari check`
(9/9 per cell), `D14` timed and untimed counters identical for all 9 pairs.

| Cell | Factor | Candidate mate | Reference mate |
| --- | ---: | ---: | ---: |
| `core` | 1.055 | 17 | 17 |
| `easy-win` | 1.106 | 15 | 15 |
| `deep-win` | 0.900 | 22 | 22 |
| **geomean** | **1.016** | | |

Seed 0 (the product default) reads **0.965** with all three cells below 1.0.
The three-seed median remains above it because tie-break-only seeding still
reorders `core`'s many equal scores: seed 1 1.147 (406,879 nodes), seed 2
1.055 (369,860), against seed 0's 342,289. This is the honest outcome of the
fix: the defect (perturbing magnitude) is gone, but changing exact ties is
still enough to move the tree. The plan's stop rule did not trigger: seeds are
deterministic, seed 0 is unchanged, and no proof's validity changed.

Because the corrected row differs from plan3's recorded row, a new ledger row
was appended at 10 s with `(cap, coverage, geomean) = (10 s, 3/3, 1.016)` and
marked a regression. Plan3's row was not rewritten.

## Problems and findings

- **Tie-break seeding is quieter but not free.** Exact-tie reordering still
  moves `core` by 8–19% in nodes. A future measurement protocol that wants a
  seed range must either accept that spread or record the product default
  (seed 0) separately.
- **The 10 s instrument is still drift-limited.** This session's seed-0
  geomean is 0.965 (kill review 1 read 0.947); the difference is host drift,
  not a code change on the seed-0 path.
- **Validated ranges are policy choices.** `--memory` 1–1,048,576 MB and
  worker/repeat limits are documented in the error messages, not in the help
  table; a future interface change should keep them together.
- **Internal invariants remain.** `board.rs`/`moves.rs` still use `expect` on
  pseudo-legal moves supplied by shakmaty; the FEN entry path and the CLI
  cannot reach those with an invalid move. No panic was observed in the flag
  or bench-cell tests.

## Missing tests / open parts

- No fuzzing of arbitrary FENs or malformed proof bytes; the checker's
  existing unit corpus is still the guard.
- No test that a failed allocation is reported rather than panicking (it would
  need a deterministic allocation failure).
- The proof export micro-optimization remains unmeasured as a product item;
  kill review 1 cancelled it as goal-driven work.

## Gates after the change

| Gate | Result |
| --- | --- |
| `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | 51 fast tests green (44 lib, 2 checker, 3 CLI, 2 prover) |
| `make test` | green (lint, licence, fast tests) |
| `make test-full` | green; ignored bench-cell test 1/1, veto 9/9 + 1/1 |
| `cargo doc --no-deps` | green |
| D8 | seed 0 fixture unchanged; seeds 1–2 deterministic; proofs valid |
| Corrected 10 s matrix | 3/3 coverage, 9/9 accepted per cell by both checkers, `D14` identical |

## Next steps

1. Owner commits the plan4 tree and records the pin in `docs/bench.md`.
2. Owner takes the `movegen` handoff decision: close `parity` and open
   `movegen`; no further parity margin work is justified by plan4.
3. If a future protocol insists on a seed range, record seed 0 as the product
   row and the seeds 1–2 spread as a robustness diagnostic, not a median.

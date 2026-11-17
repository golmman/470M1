# Plan 4 — P3: hardening

**Item:** 4 (P3 hardening). **Status:** proposed. **Session type:** execution.
**Predecessor:** [`plan3.md`](plan3.md), [`report3.md`](report3.md) and
[`kill-review1.md`](kill-review1.md). **Successor:** the `movegen` handoff
(item 5).

## Goal

Make the parity tree safe to hand off: fix the D8 seed defect, remove
user-triggerable panics, make the CLI/help/docs truthful, pin the slow and
bench regression tests, and close the licence/provenance/docs gates. This is
the last `parity` plan before `movegen`.

**No performance claim.** The only search change is the D8 tie-break fix. No
table, proof, checker or guide algorithm changes. A ledger row is written only
if the corrected seed changes the recorded `P5` row, and it is a regression
row, not a speed claim.

## Context

- Read [`initiative.md`](initiative.md), [`decisions.md`](decisions.md),
  [`gotchas.md`](gotchas.md), [`report3.md`](report3.md),
  [`kill-review1.md`](kill-review1.md) and [`../../bench.md`](../../bench.md).
- Kill review 1 found `P5` met on the product default (seed 0, geomean 0.947,
  all three cells `< 1.0`, 9/9 accepted). It cancelled backlog item 6
  ("Parity margin") and told this plan to fold in the seed correction.
- Baseline probes: `cargo test` is 44 tests in ~5 s; `make test-full` and
  `make veto` are green; `Dfpn::try_new` already exists beside the panicking
  `Dfpn::new`; `check::run` still has `expect`/`unwrap` on thread setup.
- The seed currently scales the guide's step by ±1% (`Dfpn::perturb`,
  `src/dfpn.rs`), which reorders subtrees. D8 and the CLI help say it perturbs
  only tie-breaks. That is the defect this plan fixes.

## Phases

### Phase 0 — audit and baseline (read-only)

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`, `make test`, `make test-full`, `make veto`, `cargo doc`.
2. Record per-target test wall time and list every user-facing `panic!`,
   `expect` and `unwrap` reachable from the CLI.
3. List stale docs (module headers, README status, AGENTS current state,
   initiative/index) and any missing `PROVENANCE.md` row.
4. Confirm the flag-validation gaps: non-finite/zero/negative `--seconds`,
   `--memory`, `--evals`, `--workers`, `--repeats`, `--until`.

**Stop rule:** no source change before the audit is recorded.

### Phase A — D8 seed correction (the only search change)

1. Change `Dfpn::perturb` so a seed perturbs only tie-breaks, never the
   magnitude of a guide step. Seed 0 must remain exactly "no perturbation".
   The same binary + FEN + flags must still give identical counters.
2. Add tests: seed 0 counters are unchanged; seeds 1–2 are deterministic; a
   seed may reorder equal-scored children but may not change a proof's
   validity. Pin a `core` counter fixture so a future guide change is caught.
3. Re-run one 10 s matrix (three cells, seeds 0–2, three repeats) as a
   regression check. Every claim must pass our checker and `provari check`;
   timed and untimed counters must match.
4. If the corrected seed changes the recorded `P5` row, append a new ledger
   row with its cap, coverage and geomean. Never rewrite plan3's row.

**Stop rule:** if tie-break-only perturbation cannot be made deterministic, or
changes seed 0, stop and report; do not keep tuning it in this plan.

### Phase B — error handling and CLI polish

1. `prove` uses `Dfpn::try_new` and reports an allocation failure as an error,
   not a panic.
2. `check::run` replaces the thread-spawn `expect` and the results-mutex
   `unwrap` with errors; define `--workers 0` (reject it, or document the
   current `max(1)`).
3. Validate numeric flags before use: finite and positive where required,
   `--memory` within the supported range, clear messages for bad values.
4. Fix exit codes and document them: `0` success, `1` invalid proof, `2`
   usage/error.
5. Make `--help` truthful: `--seed` perturbs tie-breaks only, and list
   `--version`, `--repeats`, `--workers`, `--compact-only`, `--reference`,
   `--evals`. Fix stale module headers in `src/main.rs` and `src/lib.rs`
   ("plan 1", "`prove` stub").

No search behaviour changes in this phase.

### Phase C — tests and gates

1. Add `tests/bench_cells.rs` with `#[ignore = "slow: ..."]`: prove the three
   bench cells at 10 s, export, check with our checker, and assert the
   counters are identical on a second run. Run it in `make test-full`.
2. Add fast tests for flag validation and seed determinism.
3. `make test-full` runs `cargo test -- --ignored` plus the veto; `make test`
   stays fast. Keep `make veto` unchanged.
4. Run the licence gate and confirm every copied fragment still has a
   provenance row. No new dependency is expected.

### Phase D — docs and closure

1. Update `README.md`, `AGENTS.md` current state, the initiative status and
   backlog (item 4 done; item 6 cancelled per kill review 1), the index and
   `history.md`.
2. Write `report4.md` with tools, problems, open parts, missing tests and next
   steps.
3. Do not start `movegen` in this plan.

## Gates

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `cargo doc --no-deps` green.
- `make test`, `make test-full` and `make veto` green.
- D8: seed 0 counters unchanged; seeds 1–2 deterministic and tie-break-only;
  no seed changes a proof's validity.
- 10 s regression: all three cells proved, 9/9 accepted by both checkers,
  timed and untimed counters identical.
- No user-triggerable panic for bad flags or memory; exit codes as documented.
- No performance claim. Any ledger row states `(cap, coverage, geomean)` and
  is a regression row.

## Deliverables

- The D8 seed fix and its tests.
- CLI/error-handling fixes and flag-validation tests.
- The ignored bench-cell regression test and the `make test-full` wiring.
- `measurements/plan4/` with `env.json` and parsed regression results.
- `report4.md` and the updated status docs.
- No new dependency, no licence change, no `movegen` work.

## Risks and stop rules

- The seed fix changes the search trajectory for seeds 1–2. If it invalidates
  a proof or breaks determinism, stop and report.
- Any change beyond the seed that touches search, table, proof or checker is a
  correctness change: run the full veto and record it as such.
- If the 10 s regression censors a cell, record the bound; do not raise the
  cap or drop the cell.
- If the oracle is unavailable, the result is `INCONCLUSIVE`; never substitute
  or skip it.
- If docs outgrow the budget, cut docs, not tests.

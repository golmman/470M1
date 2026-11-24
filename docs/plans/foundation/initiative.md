# Initiative: `foundation` — make the codebase fit the PRD

## Status

**In progress (2026-11-24)**, `PD4`. `F1` (bench v2) is done:
[plan1](plan1.md) → [report1](report1.md), first matched row 3.104 and guard
1.023. `F2`–`F7` next.

## Goal

Bring the code to [`architecture.md`](../../architecture.md) and
[`guidelines.md`](../../guidelines.md), and meet FR1–FR3 of
[`product.md`](../../product.md), **without changing the search**. Measure the
distance to the product goal against the new reference. Serves every
milestone; prepares `beat-provari`.

## Done definition

1. FR1 (config file + CLI), FR2 (typed errors, exit codes, atomic writes)
   and FR3 (stderr progress and error logs) meet their acceptance criteria,
   each with CLI tests.
2. The code audit passes: layers and crates as in `architecture.md`; no
   source file over ~10 KB of non-test code without a justified header; no
   `String` errors; `unwrap`/`expect` only as written invariants.
3. At `--threads 1`, counters for all three cells equal the movegen plan 12
   product, `.pvr` files are byte-identical, 9/9 are accepted by both
   checkers, and the differential stays 0/0.
4. No regression: on the hand guard column, candidate/old geomean `<= 1.01`
   in one same-session row, and `core` `instructions:u` within +0.5% of the
   plan 12 product.
5. Bench v2 exists and has its first row: the distance to the goal is a
   measured number.

## Non-goals

No search, guide, evaluation, table-policy, proof-format or checker-rules
change. No threads, learned guide or proof database: those are later
initiatives.

## Items

| # | Item | Content |
| --- | --- | --- |
| F1 | **Bench v2** | re-pin the oracle to the latest provari release (commit, data SHA-256, `PROVENANCE.md`); full-feature reference command (learned guide, tablebases, `--threads N`); fix `N`, the tablebase set and the canonical cap; select `strong` (cells the reference proves at the cap, from `corpus-v1` + provari's `high200`) and a `held-out` set; driver and `bench.md` sections; first row with the current product |
| F2 | **Errors and exit codes** | error enum per layer, exit code 3, atomic writes, invariant `expect`s; `String` errors retired |
| F3 | **Configuration** | `clap` + `serde` + `toml`, `--config FILE`, typed validated configs, effective config in `--json`; current defaults unchanged |
| F4 | **Logging** | `Observer`/`Progress` at clock ticks, stderr logger, levels, `-q`/`-v`; trajectory-neutrality test |
| F5 | **Module split** | `dfpn`, `moves`, `format`, `main` per `architecture.md`; `Guide`, `Budget`, `ProofSource` seams with contract tests; `instructions:u` per step |
| F6 | **Checker crate** | workspace with `atomic470mi-check` (shakmaty only); `check.rs` split; argument parsing moves to app |
| F7 | **Closure** | the three audits, a closing report, status line |

Suggested order: F1 (measurement only, gives the baseline), F2 + F3 (the CLI
edge), F4, F5, F6, F7. The plan session may regroup items into plans.

## Risks

| Risk | Mitigation |
| --- | --- |
| A split moves inlining and costs wall | `instructions:u` on `core` per step; done criterion 4 |
| Re-pin changes what a row means | new reference column; old rows are never rewritten |
| Tablebases and threads depend on the host | `N`, tablebase set and host recorded per row; geomeans compared only within one `N` and cap |
| The new reference makes the gap look large | it is the goal's true distance; report it, do not tune against it here |
| Config plumbing changes a default | defaults pinned by counters (criterion 3) |

# AGENTS.md

## Goal

`470M1` is a from-scratch atomic chess win prover and independent proof
checker. The first initiative, `parity`, must reach wall-time parity with
provari on the fixed bench set: all three cells proved at 10 s and a geomean
wall factor `<= 1.0` against `provari --guide hand`, with every claim accepted
by our checker and by `provari check`. The second initiative, `movegen`,
replaces the parity rules dependency with `atomic-movegen` only after parity is
reached.

The goal, rungs and budgets live in
[`docs/plans/parity/initiative.md`](docs/plans/parity/initiative.md). The
performance ledger lives in [`docs/bench.md`](docs/bench.md) and makes no goal
claim.

## Current state

`parity` is closed (2026-11-17). Plan 5 recorded the `P5` closure row on the
product default (seed 0) at 10 s: all three cells, 3/3 coverage, geomean
**0.945**, 9/9 proofs accepted by both checkers, `D14` timed and untimed
counters identical. The crate, rules layer, proof format, independent checker,
`line`, bench harness, oracle/veto and the DF-PN search core with the hand
guide are in place. `movegen` is open with a documented rules surface and a
candidate `atomic-movegen` pin (`6e04488`, v2.8.0, MIT); its first plan starts
with the differential harness. See
[`docs/plans/parity/report5.md`](docs/plans/parity/report5.md) and
[`docs/plans/movegen/initiative.md`](docs/plans/movegen/initiative.md).

## Collaboration

- Fixed point: the goal plus the correctness and licence rules. Everything
  else (milestones, working agreement, decisions) is a means; challenge it.
- Push back first when a plan, decision or request conflicts with the goal or
  rests on an untested assumption.
- Ideation: include at least one option that changes the problem formulation,
  plus an estimated gain and the cheapest decisive experiment.
- "Should we X?": answer yes/no first, give the strongest case each way, then
  the cheapest decisive experiment.

## Architecture (planned)

The parity layout follows provari's separation of concerns:

- `src/board.rs` — atomic position, move and effect representation; hashes
  equal shakmaty's.
- `src/moves.rs` — per-position move precomputation, legality, checks and
  one-ply explosion facts.
- `src/table.rs` — bounded transposition table: 4-slot clusters, filter,
  least-work replacement.
- `src/proofs.rs` — full-key proof table; keeps `(move, length, limit)` and
  never evicts.
- `src/dfpn.rs` — depth-first proof-number search, hand guide, simulation,
  null-threat replay, history.
- `src/eval.rs` — hand-crafted atomic evaluation.
- `src/format.rs` — compact `.pvr` proof format and JSONL writer/reader.
- `src/check.rs` — independent checker; uses shakmaty only and shares no code
  with the prover.
- `src/walk.rs` — proof walk, length and tree measurement.
- `src/main.rs` — CLI: `prove`, `check`, `line`, `bench`, `--version`,
  `--help`.
- `tests/`, `bench/`, `scripts/` — integration tests, corpus/driver, oracle
  build.

The movegen initiative later swaps the prover's rules layer to
`atomic-movegen`; the checker stays on shakmaty so the two rules
implementations remain independent.

## Commands (to be created by plan 1)

- `cargo test` — fast unit and integration tests.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo doc` — lint and
  documentation gates.
- `make test` — licence gate plus the fast test suite.
- `make test-full` — slow tests plus the oracle veto.
- `make oracle` — build the pinned provari oracle in a scratch directory
  outside the repo.
- `make veto` — our checker plus `provari check` on the proof fixtures.
- `make bench` — run the bench driver and append a ledger row.

## Licence and copy boundary

- This repository is **GPL-3.0-or-later**. Every dependency must be
  GPL-compatible.
- provari is GPL-3.0-or-later. Its code may be copied or adapted into this
  repository, but every copied file or fragment must carry a provenance line
  and a row in [`PROVENANCE.md`](PROVENANCE.md).
- `atomi` and `atomic_solver` are MIT. Their code and data may be copied or
  adapted under the same provenance rule.
- Never copy material whose licence is incompatible with GPL-3.0-or-later.
  Never weaken the licence gate.
- A copied file keeps its upstream copyright and licence notice. After the
  copy, this repository owns the file; re-copies are deliberate changes with a
  new provenance row, never implicit syncs.
- provari is also used as an **external process** for the veto and the bench
  reference. It is built from source at the pinned commit in a scratch
  directory outside the repository and is never committed.

## Correctness

- Correctness first: a faster false win is a failure.
- A win is *claimed* only after its proof passed our independent checker
  **and** `provari check`. A checker disagreement is a veto.
- Repetition and the 50-move limit are never wins. Identity is the board, side
  to move, castling rights and legal en passant square, without the clocks;
  proofs carry length and 50-move limit.
- The checker shares no code with the prover. A helper both sides need is
  written twice.
- The search is deterministic: same binary + FEN + flags = same counters.
  Collecting a proof never changes the search trajectory.
- Read [`docs/plans/parity/gotchas.md`](docs/plans/parity/gotchas.md) before
  touching search, table, proof or checker code.

## Session protocol

- **Plan session:** read the initiative and prior plans/reports; run only
  read-only probes; write `planN.md` under `docs/plans/<initiative>/` (one
  sitting, <= 8 KB).
- **Execution session:** implement the plan, run the gates, write `reportN.md`
  (tools, problems, open parts, missing tests, next steps; <= 12 KB). Scope
  changes go in the report.
- **Kill review after every 2 plans** (fixed prompt, do not implement): "What
  is the absolute distance to the goal now? If we started today, would we
  build this? Which initiative or plan would you cancel? Re-run the prior-art
  search, delta only. What is the smallest next experiment?"
- **Docs budget:** plans <= 8 KB, reports <= 12 KB, this file <= 8 KiB,
  one-line status entries. If docs outgrow the code, cut docs, not tests.
- **Measurements:** one process at a time, same-session ratios, reading rules
  in `docs/bench.md`; commit drivers, parsed results, `env.json` and READMEs
  under `docs/plans/<initiative>/measurements/planN/`; never raw transcripts,
  oracle artifacts or `__pycache__`.
- **Closure:** end with `SESSION COMPLETE`, deliverables and follow-up
  options, option 1 a copy-pasteable kickoff prompt.
- Items and plans are numbered independently; a plan header names its item.

## Conventions

- Rust 2024; `cargo fmt`, `cargo clippy -- -D warnings`, `cargo doc`.
- Avoid `unsafe` unless a measured win justifies it (documented, guarded).
- Source files under ~10 KB (larger: header justification). Unit tests in
  `#[cfg(test)] mod tests`, integration in `tests/`; slow tests use
  `#[ignore = "slow: ..."]`, never `cfg_attr(debug_assertions, ignore)`.
- Agents use read-only `git`; the owner commits.
- `docs/notes/` is the owner's unfiltered notebook; do not treat it as
  normative.

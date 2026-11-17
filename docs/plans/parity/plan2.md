# Plan 2 — P1: DF-PN search core, hand guide, proof writer

**Item:** 2 (P1 search core). **Status:** next. **Session type:** execution.
**Predecessor:** [`plan1.md`](plan1.md). **Successor:** [`plan3.md`](plan3.md).

## Goal

Turn the plan1 foundation into a working attacker-only win prover: port
provari's board/move/effect layer, transposition table, full-key proof table,
DF-PN search, hand guide and proof writer. Reach at least `P1` (`core` proved
at 120 s) and preferably `P2` (`core` + `easy-win` at 120 s), with every
claimed proof accepted by our checker and `provari check`.

## Context

- Read [`initiative.md`](initiative.md), [`decisions.md`](decisions.md) and
  [`gotchas.md`](gotchas.md) first.
- plan1 provides the crate, rules layer, proof format, checker, `line`,
  bench harness and oracle.
- The parity reference is provari `v0.4.0` at `bceb1ee`. The search is DF-PN
  with proof/disproof numbers, 1+ε, hand guide, move classes, EF-PN,
  simulation, null-threat replay and history.
- No tablebases, NNUE/model, web, UCI, portfolio or parallel search (D7).
- The hand guide is the product. `--guide hand` is the only guide in this
  plan; `--guide none` is a diagnostic control.

## Tasks

1. **Board, moves and effects.** Copy/adapt provari's `src/board.rs`,
   `src/moves.rs` and `src/eval.rs`. Keep the 16-bit move, `Effect`
   precomputation, child key before materialization, and the hand evaluation.
   Remove tablebase and network/model branches. Add differential tests against
   shakmaty for legal moves, checks, explosions, hashes and en passant.

2. **Transposition table.** Copy/adapt provari's `src/table.rs`: 4-slot
   clusters, filter, least-work replacement, `Open`/`Refuted`/`Mate` entries.
   Keep the invariant that `Open` numbers saturate below `INFINITY`. Add a
   test that the filter changes no result.

3. **Proof table.** Copy/adapt provari's `src/proofs.rs`: full 64-bit keys,
   `(move, length, limit)` per key, never evicts, refuses at the load limit.
   The proof table is the writer's source, so collecting a proof cannot change
   the search trajectory. Add a test for the non-dominated set and the
   `store-full` failure mode.

4. **DF-PN search.** Copy/adapt provari's `src/dfpn.rs` and `src/search.rs`.
   Keep the value algebra, 1+ε, per-frame child numbers, move classes,
   EF-PN hand scaling, quiet grouping, simulation, null-threat replay and
   history. Remove tablebase, network/model, threads and progress features
   that are not needed for parity. Keep the search deterministic. Add unit
   tests for values, thresholds, repetition and 50-move handling.

5. **Proof writer.** Wire the proof table to `src/format.rs`. Implement
   `prove FEN [--seconds S] [--memory MB] [--guide hand|none] [--proofs PATH]
   [--json] [--seed N]`. A timed run writes no proof; a separate untimed run
   with `--proofs` writes the proof and must match the timed run's counters.
   The writer fails closed on a cycle, a 50-move failure or a missing move.

6. **CLI and bench.** Complete `prove` and the candidate side of `bench`.
   Keep the reference side from plan1. Add `--attacker white|black` only if
   it is needed for the fixtures; otherwise leave it out. Add `--evals N` as a
   deterministic child-eval budget for diagnosis.

7. **Fixtures and differential tests.** Add the legacy decisive-win fixtures
   and never-prove negatives (with provenance). Run a differential against
   provari on a small set: same FEN, same guide, compare found/refuted and
   proof acceptance, not necessarily the same tree. A refutation on a known
   win is a bug; a censored run is a result.

8. **First measurements.** Run the three bench cells at 120 s/192 MB, seeds
   0–2, three repeats, same session as the reference. Record walls,
   `child_evals`, `expansions`, proof length and D14 verdicts under
   `measurements/plan2/`. Append one candidate row to `docs/bench.md` only if
   the gates pass.

9. **Report.** Write `report2.md` with tools, problems, open parts, missing
   tests, the rung reached, the ledger row (if any), and next steps. Update
   the initiative and index status.

## Gates

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` green.
- `make test` and `make test-full` green; `make veto` green.
- No refutation on a known-win fixture; no proof of a never-prove negative.
- `P1`: `core` proved at 120 s/192 MB, 3/3 seeds, every proof accepted by our
  checker and `provari check`.
- `P2` (stretch): `easy-win` also proved at 120 s, 3/3 seeds, both checkers.
- The timed and untimed runs have identical `child_evals` and `expansions`.
- A ledger row is appended only if the gates pass; it records the cap,
  coverage, geomean and proof lengths.

## Deliverables

- Working `prove` and candidate `bench`.
- Board/moves/effect, table, proofs, DF-PN, hand guide, proof writer.
- Differential and fixture tests.
- `measurements/plan2/` with parsed results and `env.json`.
- `report2.md` and updated status.

## Risks and stop rules

- If `core` is not proved at 120 s, do not start plan3. Diagnose the search
  core: tree size, per-node cost, guide configuration, proof-table pressure.
- If a proof is rejected by either checker, stop and fix correctness before
  any speed work.
- If the port is much slower than provari, record the gap and the per-node
  and tree diagnosis; plan3 decides whether to port atomi's measured levers.
- Do not add tablebases, NNUE/model, web, UCI, portfolio or parallel search.
  A proposal to add one goes to the kill review, not into this plan.

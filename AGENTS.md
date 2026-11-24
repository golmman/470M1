# AGENTS.md

## Goal

`470MI` is the fastest Atomic chess win prover, with an independent proof
checker. The goal moves; it, the milestones and the requirements are in
[`docs/product.md`](docs/product.md). Status lives only in
[`docs/plans/README.md`](docs/plans/README.md).

## Read first

| Document | Read when |
| --- | --- |
| [`docs/product.md`](docs/product.md) | always: purpose, goal, milestones, requirements |
| [`docs/decisions.md`](docs/decisions.md) | before proposing anything that touches scope, metric or process |
| [`docs/architecture.md`](docs/architecture.md) | before adding or moving code |
| [`docs/guidelines.md`](docs/guidelines.md) | before writing code |
| [`docs/gotchas.md`](docs/gotchas.md) | before touching search, table, proof, checker or measurement code |
| [`docs/workflow.md`](docs/workflow.md) | before a plan, execution, audit or documentation session |
| [`docs/bench.md`](docs/bench.md) | before measuring or appending a ledger row |
| [`docs/proof-format.md`](docs/proof-format.md) | before touching `.pvr`/JSONL code |

## Roles

AI writes all code and documentation, runs all tests and reviews, proposes
changes and consults. The owner provides guides (goals, priorities,
decisions) and sensors (hosts, releases, review) and commits. Agents use
read-only `git`.

## Collaboration

- Fixed point: the product goal plus the correctness and licence rules.
  Everything else (milestones, workflow, decisions) is a means; challenge it.
- Push back first when a plan, decision or request conflicts with the goal or
  rests on an untested assumption.
- Ideation: include at least one option that changes the problem formulation,
  plus an estimated gain and the cheapest decisive experiment.
- "Should we X?": answer yes/no first, give the strongest case each way, then
  the cheapest decisive experiment.

## Correctness (non-negotiable)

- A faster false win is a failure.
- A win is claimed only after its `.pvr` passed our checker **and**
  `provari check`. A disagreement is a veto.
- Repetition and the 50-move limit are never wins. Identity is board, side to
  move, castling rights and legal en passant square, without clocks; proofs
  carry length and 50-move limit.
- The checker uses shakmaty only and shares no code with the prover; a helper
  both need is written twice. The prover's rules layer is `atomic-movegen`.
- At `--threads 1` the same binary, position and configuration give the same
  counters; proof collection, logging and configuration never change the
  search trajectory.

## Licence and copy boundary

- The repository is **GPL-3.0-or-later**; every dependency is
  GPL-compatible. Never weaken the licence gate.
- provari (GPL-3.0-or-later), `atomi` and `atomic_solver` (MIT) may be copied
  or adapted. Every copied file or fragment keeps its upstream notice and has
  a provenance line and a row in [`PROVENANCE.md`](PROVENANCE.md). After a
  copy the file is ours; a re-copy is a deliberate change with a new row.
- Never copy material whose licence is incompatible with GPL-3.0-or-later.
- provari also runs as an external process for the veto and the bench
  reference, built from source at a pinned commit outside the repository and
  never committed.
- `atomic-movegen` is ours upstream: rules-layer needs go in a
  `wishlistN.md`, never a fork or path dependency.

## Commands

- `cargo test` — fast unit and integration tests.
- `make test` — lint (`fmt`, `clippy -D warnings`), licence gate, fast tests.
- `make test-full` — plus slow tests, the `unsafe-feature` gate and the veto.
- `make doc` — `cargo doc --no-deps`.
- `make oracle` — build the pinned provari oracle under `/tmp/470M1-oracle`.
- `make veto` — our checker plus `provari check` on the proof fixtures.
- `make bench` — run the bench driver.

## Layout

`src/` prover library and CLI (module map in
[`architecture.md`](docs/architecture.md#as-built-map-2026-11-24)), `tests/`
integration tests and fixtures, `bench/` corpus and driver, `scripts/`
gates and oracle build, `docs/plans/<initiative>/` plans, reports and
measurements, `docs/notes/` the owner's notebook (not normative).

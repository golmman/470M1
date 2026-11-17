# History and findings — `parity`

Chronological record. The initiative keeps only the current state; details
live in the plan and report files.

## Findings

### Start from provari's architecture, not from a general solver (2026-11-17)

The predecessor projects measured the cost of starting from a general
Win/Loss/Draw solver. `atomic_solver`'s `winprover` initiative ported every
portable provari lever into the general solver and plateaued: its best opt-in
stack was 6.30x provari hand on two shared cells, and the shipped default was
44.06x on one. The gap was per-node cost and search efficiency in the engine
core, not a missing small lever.

`atomi` then started from scratch as an attacker-only DF-PN prover and reached
`T_3` (all three cells at 120 s) before its per-node route to `T_4` closed by
measurement. Its durable lessons are the planning process (initiative,
decisions, gotchas, history, bench ledger), the stack-level gating discipline,
and the measured levers (HOTPATH-V2, NULL-PIPE, proof-retention share, guide
tuning).

`parity` takes the next step: start from provari's own architecture and proof
format, which the GPL licence allows, and reach wall-time parity before
changing the rules layer. The movegen replacement is deliberately deferred to
a second initiative so search and rules effects are not confounded.

## History

- **2026-11-17** — documentation scaffolding created. Read provari `v0.4.0`
  (`bceb1ee`), atomi (`a4802e7`) and atomic_solver (`14abcf2`). Recorded
  decisions D1–D12, the parity rungs `P1`–`P5`, the source take-list, the
  independent bench ledger, and the first three plans. No code, no
  measurements.

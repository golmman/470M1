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

- **2026-11-17** — plan 3 executed. Measured three protocol variants in four
  interleaved same-session 10 s passes and one 120 s pass. `P4` reached (all
  three cells, 9/9, both checkers at 10 s, timed and untimed counters
  identical). `P5` recorded as met on the protocol: median geomean 0.983
  (range 0.964–1.013) against provari `--guide hand`, margin inside the host's
  ~10% wall drift. No lever adopted: the candidate's per-node cost is already
  at or below provari's on an identical tree, so HOTPATH-V2/NULL-PIPE are
  closed; the proof-retention share was swept and closed (proofs never fill;
  the tree moves by up to 35% with the table size). The `core` gap is the seed
  protocol's 10–17% tree inflation plus the 26 ms proof export. Findings: the
  plan2 driver mixed seeds in its counters; the cap selects the memory split,
  so it is a search parameter. Ledger row at 10 s. See
  [`report3.md`](report3.md).
- **2026-11-17** — plan 2 executed. Ported provari `v0.1.0`'s single-threaded
  table, full-key proofs, DF-PN and exhaustive baseline, stripped tablebase,
  network, model, progress and sampling code, and added the hand guide, proof
  writer and `prove`. At 120 s all three cells proved, 9/9 seeds/repeats,
  every proof accepted by both checkers, geomean 0.985. A same-session 10 s
  matrix gave geomean 0.968, 9/9 each. See [`report2.md`](report2.md).
- **2026-11-17** — plan 1 executed. Crate, rules helpers, compact/JSONL
  proof format, independent checker, `line`, reference-side bench, oracle and
  veto landed. Eight oracle-written proofs accepted by both checkers; one
  truncated control rejected by both. Reference medians: `core` 0.342 s,
  `easy-win` 0.075 s, `deep-win` 4.721 s. No ledger row. See
  [`report1.md`](report1.md).
- **2026-11-17** — documentation scaffolding created. Read provari `v0.4.0`
  (`bceb1ee`), atomi (`a4802e7`) and atomic_solver (`14abcf2`). Recorded
  decisions D1–D12, the parity rungs `P1`–`P5`, the source take-list, the
  independent bench ledger, and the first three plans. No code, no
  measurements.

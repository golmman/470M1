# `parity` — document index

The initiative is [`initiative.md`](initiative.md). It is the only normative
statement of the goal, the rungs and the current priority. Everything else is
a decision record, a measurement record, or a historical plan/report.

## Read in this order

| Document | Role |
| --- | --- |
| [`initiative.md`](initiative.md) | Goal, rungs, current status, decisions summary, backlog, risks |
| [`decisions.md`](decisions.md) | Full text and rationale for D1–D12 |
| [`gotchas.md`](gotchas.md) | Binding correctness gotchas — read before touching search, table, proof or checker code |
| [`history.md`](history.md) | Chronological record and durable findings |
| [`analysis1.md`](analysis1.md) | Prior-art map of provari, atomi and atomic_solver; what parity takes and defers |
| [`plan1.md`](plan1.md) | P0: repository, rules, proof format, independent checker, bench harness |
| [`report1.md`](report1.md) | Plan 1 result: gates green, reference values, scope changes |
| [`plan2.md`](plan2.md) | P1: DF-PN search core, hand guide, proof writer |
| [`report2.md`](report2.md) | Plan 2 result: search core, 120 s and 10 s matrices |
| [`plan3.md`](plan3.md) | P2: parity measurement and gap closure |
| [`report3.md`](report3.md) | Plan 3 result: `P4`/`P5` measured, levers closed, ledger row at 10 s |
| `measurements/planN/` | Committed drivers, parsed results, `env.json`, provenance READMEs |

## Current state (2026-11-17)

- Plan 1 complete: crate, rules helpers, compact/JSONL proof format,
  independent checker, `line`, reference-side bench, oracle and veto.
- Plan 2 complete: board/moves, table, full-key proofs, DF-PN with the hand
  guide, proof writer and `prove`.
- Plan 3 complete: `P4` at 10 s (all three cells, 9/9, both checkers, timed and
  untimed counters identical); `P5` recorded as met on the protocol (median
  geomean 0.983 over four same-session passes, range 0.964–1.013). No lever
  adopted; the `core` gap is the seed protocol's tree plus the proof export.
- Measurements are in [`measurements/plan1/`](measurements/plan1/),
  [`measurements/plan2/`](measurements/plan2/) and
  [`measurements/plan3/`](measurements/plan3/); ledger rows are in
  [`../../bench.md`](../../bench.md).
- Next: the kill review (plans 2–3 since the last one); it decides between
  closing `parity` for `movegen` and buying `core` margin first.

## Conventions

- Plans are numbered sequentially and independently of backlog items; a plan
  header names its item.
- Plans <= 8 KB, reports <= 12 KB. If docs outgrow the code, cut docs, not
  tests.
- Measurements: one process at a time, same-session ratios, reading rules in
  [`../../bench.md`](../../bench.md); commit drivers, parsed results,
  `env.json` and READMEs under `measurements/planN/`; never raw transcripts,
  oracle artifacts or `__pycache__`.
- The licence and copy boundary is hard: see `AGENTS.md` and D1/D10.

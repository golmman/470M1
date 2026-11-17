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
| [`plan2.md`](plan2.md) | P1: DF-PN search core, hand guide, proof writer |
| [`plan3.md`](plan3.md) | P2: parity measurement and gap closure |
| `measurements/planN/` | Committed drivers, parsed results, `env.json`, provenance READMEs |

## Current state (2026-11-17)

- Documentation scaffolding only; no code, no measurements.
- Next session: execute [`plan1.md`](plan1.md).
- The performance ledger is [`../../bench.md`](../../bench.md).

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

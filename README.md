# 470M1

A from-scratch atomic chess win prover and independent proof checker.

The first initiative, [`parity`](docs/plans/parity/initiative.md), aims at
wall-time parity with [provari](https://github.com/aiorla/provari) on a fixed,
reproducible bench set. The second, [`movegen`](docs/plans/movegen/initiative.md),
replaces the parity rules dependency with `atomic-movegen` after parity is
reached.

**Status:** documentation scaffolding only. No code has been written yet.

## Documents

| Document | Purpose |
| --- | --- |
| [`AGENTS.md`](AGENTS.md) | Working rules for agents: goal, architecture, gates, GPL/copy boundary, session protocol |
| [`docs/bench.md`](docs/bench.md) | Initiative-independent performance ledger and measurement rules |
| [`docs/plans/README.md`](docs/plans/README.md) | Initiative index and status |
| [`docs/plans/parity/initiative.md`](docs/plans/parity/initiative.md) | Parity goal, rungs, decisions, backlog |
| [`docs/plans/movegen/initiative.md`](docs/plans/movegen/initiative.md) | Movegen replacement goal and prerequisites |
| [`PROVENANCE.md`](PROVENANCE.md) | Record of every copied file or fragment |

## License

GPL-3.0-or-later. See [`LICENSE`](LICENSE).

# 470MI

The fastest Atomic chess win prover, with an independent proof checker.

`470MI prove FEN` searches for a forced win and writes a compact `.pvr`
proof; `470MI check` verifies it with a separate rules implementation, and
every claimed win is also accepted by
[provari](https://github.com/aiorla/provari)'s checker. Run `470MI --help`
for commands and options.

Status: [`docs/plans/README.md`](docs/plans/README.md).

## Documents

| Document | Purpose |
| --- | --- |
| [`docs/product.md`](docs/product.md) | Purpose, moving goal, milestones, requirements |
| [`docs/decisions.md`](docs/decisions.md) | Product-level decisions |
| [`docs/architecture.md`](docs/architecture.md) | Layers, seams, as-built map |
| [`docs/guidelines.md`](docs/guidelines.md) | Code guidelines |
| [`docs/workflow.md`](docs/workflow.md) | Initiatives, plans, reports, audits |
| [`docs/gotchas.md`](docs/gotchas.md) | Binding correctness traps |
| [`docs/bench.md`](docs/bench.md) | Measurement rules and performance ledger |
| [`docs/proof-format.md`](docs/proof-format.md) | The `.pvr` and JSONL proof formats |
| [`AGENTS.md`](AGENTS.md) | Entry point for agents |
| [`PROVENANCE.md`](PROVENANCE.md) | Every copied file or fragment |

## License

GPL-3.0-or-later. See [`LICENSE`](LICENSE).

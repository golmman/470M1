# Initiatives

| Initiative | Status | Documents |
| --- | --- | --- |
| [`parity`](parity/initiative.md) | **closed** — plan 5 complete. `P5` closure row on the product default (seed 0) at 10 s: 3/3 coverage, geomean 0.945, 9/9 accepted by both checkers, `D14` identical. Goal: all three bench cells proved at 10 s, geomean wall `<= 1.0` vs `provari --guide hand`, every claim accepted by our checker and `provari check` | [index](parity/README.md), [initiative](parity/initiative.md), [decisions](parity/decisions.md), [gotchas](parity/gotchas.md), [history](parity/history.md), [plan5](parity/plan5.md), [report5](parity/report5.md), [measurements/plan5](parity/measurements/plan5/README.md) |
| [`movegen`](movegen/initiative.md) | **closed** — plan 13 closed the initiative on `MD1`'s done definition: the prover runs on `atomic-movegen`, the checker stays shakmaty-only, differential 0/0, `.pvr` byte-identical, 9/9 accepted. The `MD6` no-regression clause is a dated miss (1.001 product / 1.013 vs plan 2); plan 13's export split shows the walk is the rules layer (1.58x plan 2, 1.57x the oracle), with ≤0.08e9 re-derivable. No ledger row, no source or pin change | [initiative](movegen/initiative.md), [decisions](movegen/decisions.md), [rules surface](movegen/rules-surface.md), [hot-path audit](movegen/hot-path-audit.md), [report12](movegen/report12.md), [report13](movegen/report13.md), [measurements/export-split](movegen/measurements/export-split/README.md) |

## Conventions

- Each subdirectory of `docs/plans/` is an initiative with one high-level goal.
- `initiative.md` is the only normative statement of an initiative's goal,
  rungs, current priority and backlog.
- Plans are numbered sequentially and independently of backlog items; a plan
  header names its item.
- Plans are self-contained so a fresh session can execute one in a sitting.
- The final task of an execution plan is the corresponding `reportN.md`.
- `wishlistN.md` files are numbered requests to the upstream `atomic-movegen`
  crate, which this project owns. They are not plans and imply no
  implementation. A release is integrated only by an owner-triggered
  plan/report and a pin update.
- Measurements live under `docs/plans/<initiative>/measurements/planN/`.
- The performance ledger is [`../bench.md`](../bench.md); it is independent of
  any initiative and makes no goal claim.
- Older plans and reports may not reflect the current state; the initiative
  and the latest report do.

# Initiatives

| Initiative | Status | Documents |
| --- | --- | --- |
| [`parity`](parity/initiative.md) | **closed** — plan 5 complete. `P5` closure row on the product default (seed 0) at 10 s: 3/3 coverage, geomean 0.945, 9/9 accepted by both checkers, `D14` identical. Goal: all three bench cells proved at 10 s, geomean wall `<= 1.0` vs `provari --guide hand`, every claim accepted by our checker and `provari check` | [index](parity/README.md), [initiative](parity/initiative.md), [decisions](parity/decisions.md), [gotchas](parity/gotchas.md), [history](parity/history.md), [plan5](parity/plan5.md), [report5](parity/report5.md), [measurements/plan5](parity/measurements/plan5/README.md) |
| [`movegen`](movegen/initiative.md) | **open** — plan 2 green; decisions MD1–MD4 recorded. Next: plan 3, the rules adapter, starting with the rename and a native-key bench. Goal: replace shakmaty completely in the prover with `atomic-movegen`; the checker stays independent on shakmaty | [initiative](movegen/initiative.md), [decisions](movegen/decisions.md), [rules surface](movegen/rules-surface.md), [wishlist 1](movegen/wishlist1.md) |

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

# Initiatives

| Initiative | Status | Documents |
| --- | --- | --- |
| [`parity`](parity/initiative.md) | open — plan 4 complete (P3 hardening). `P4` at 10 s (9/9, both checkers); `P5` met on the product default (seed 0, geomean 0.965 this session). D8 seed now tie-break-only; the corrected three-seed row is 1.016 (regression). Next: `movegen` handoff decision. Goal: all three bench cells proved at 10 s, geomean wall `<= 1.0` vs `provari --guide hand`, every claim accepted by our checker and `provari check` | [index](parity/README.md), [initiative](parity/initiative.md), [decisions](parity/decisions.md), [gotchas](parity/gotchas.md), [history](parity/history.md), [analysis1](parity/analysis1.md), [plan4](parity/plan4.md), [report4](parity/report4.md), [measurements/plan4](parity/measurements/plan4/README.md) |
| [`movegen`](movegen/initiative.md) | not started — begins only after `parity` reaches its goal. Goal: replace the prover's shakmaty rules layer with `atomic-movegen` while keeping the checker independent | [initiative](movegen/initiative.md) |

## Conventions

- Each subdirectory of `docs/plans/` is an initiative with one high-level goal.
- `initiative.md` is the only normative statement of an initiative's goal,
  rungs, current priority and backlog.
- Plans are numbered sequentially and independently of backlog items; a plan
  header names its item.
- Plans are self-contained so a fresh session can execute one in a sitting.
- The final task of an execution plan is the corresponding `reportN.md`.
- Measurements live under `docs/plans/<initiative>/measurements/planN/`.
- The performance ledger is [`../bench.md`](../bench.md); it is independent of
  any initiative and makes no goal claim.
- Older plans and reports may not reflect the current state; the initiative
  and the latest report do.

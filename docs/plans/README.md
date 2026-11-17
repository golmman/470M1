# Initiatives

| Initiative | Status | Documents |
| --- | --- | --- |
| [`parity`](parity/initiative.md) | open — plan 1 complete (P0 foundation: rules, proof format, independent checker, `line`, reference bench, oracle/veto). Next: plan 2 (search core). Goal: all three bench cells proved at 10 s, geomean wall `<= 1.0` vs `provari --guide hand`, every claim accepted by our checker and `provari check` | [index](parity/README.md), [initiative](parity/initiative.md), [decisions](parity/decisions.md), [gotchas](parity/gotchas.md), [history](parity/history.md), [analysis1](parity/analysis1.md), [plan1](parity/plan1.md), [report1](parity/report1.md), [plan2](parity/plan2.md), [plan3](parity/plan3.md) |
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

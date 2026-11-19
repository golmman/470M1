## Context

Our last try to build the fastest atomic chess win-prover was `https://github.com/golmman/atomi`.
Our current assessment is that we can't close the T_5 gap within a reasonable amount of effort. So here is my plan to move forward.

Start from scratch in a new repository (this repo, `470M1`) with GPL-3.0 license.

That way we can copy freely from

- https://github.com/aiorla/provari
- https://github.com/golmman/atomi
- https://github.com/golmman/atomic_solver

and achieve parity with provaris wall times, then improve further by adopting ideas from atomi and atomic_solver.

The planning process and structure of atomi and atomic_solver proved to be fruitful, so i propose adopting the best ideas, e.g. initiative structure, decisions, history, gotchas etc.

In particular i like to have:

**bench.md**

- similar to atomis `docs/plans/prover/bench.md`
- independent of initiative
- focus on the "Ledger" table

**Initiatives**

- `parity`
  - pure win prover and checker with cli based on provari
  - goal: wall times <= provari
  - no web frontend
  - sub-goals and non-goals similar to the `prover` initiative of `atomi`
- `movegen`
  - goal: replace `shakmaty` with `atomic-movegen`
  - started after `parity` is finished

## Task

Your task is build only the documentational scaffolding now, no coding in this session.
Get a thorough understanding of provari, atomi, atomic_solver and then
set up the directory and file structure, add the initiatives, add the first plans in the `parity` initiative, add an AGENTS.md, etc.

---

Bookkeeping/Brainstorming session, no code changes.

Note that we own `atomic-movegen`, i.e. if needed we can propose changes.
Wishlists of changes should be added as `wishlistN.md` files in the `movegen` initative. integration of new releases is triggered by the owner and follows the usual `planN.md` and `reportN.md` cycle.
If this was not known we should add it to the `AGENTS.md` or `movegen` initiative file.

---

Note that the items of `docs/plans/movegen/wishlist1.md` have been added upstream and `atomic-movegen` is now at version `2.9.0`.
Check the changelog then draft the first plan in the initiative.

---

Plan session. Read AGENTS.md, docs/plans/movegen/{initiative,report1,rules-surface}.md and tests/differential.rs. Write docs/plans/movegen/plan2.md (<= 8 KB) to fix the en-passant immediate-explosion gap in moves::exploding_capture and Mover::exposes, starting from FEN "8/6kp/8/6P1/8/8/8/K7 b - - 0 1" move h7h5 witness g5h6. Include the differential harness, checker/veto gates and a same-session 10 s bench row; no adapter and no key-table change. Run only read-only probes; do not implement.

---

open decisions?

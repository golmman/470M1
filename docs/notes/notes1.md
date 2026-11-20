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

---

1. Movegen Goal: Replace shakmaty completely with not negative impact on performance; Stretch goal: improve overall performance
2. Use atomic-movegen current keys if more performant. If not, formulate a wishlist so atomic-movegen can adopt shakmaty's keys. In the end we want only atomic-movegen but faster than shakmaty.
3. Option B
4. the name `470MI` is final

---

Plan session for movegen: profile the M1 adapter against the stored plan 2 binary, identify the per-node regression in pseudo_legal, legal and effect, and propose the smallest measured fix that brings the 10 s row back to <= 1.0.

---

Decisions:
1B
3B

I don't understand Decision 2 as stated.
I want clean and performant code, no extra adapter. If that means `atomic-movegen` needs to adapt then we put this into a wishlist.

---

Decision 2: B
Decision A: yes
Decision B: B
Decision C: B

Let's do the "Patch atomic-movegen locally in /tmp" first though

---

Please analyze: why is it so hard to replace `shakmaty` with `atomic-movegen` without losing performance?
What can we do to succeed with both goals?

---

Read docs/plans/movegen/report8.md , decisions.md ( MD7 ) and hot-path-audit.md . Plan 9: measure the evasion-mask prototype ( /tmp/mg-ev shape: per-node block mask between the king and a single sliding checker, castling rejected in check) and atomic-movegen/unsafe-optimizations on core , easy-win and deep-win with three interleaved perf sta t -e instructions:u pairs each on the bench host. If the mask clears 0.31e9 on core with identical counters and .p vr , write wishlist3.md W10 (evasion-aware LegalityContext ) and a dated MD8 reopening MD7 ; record the unsafe-feature result for an owner decision.

---

Read  docs/plans/movegen/plan9.md  and  measurements/plan9/README.md . On  c90face26537 , run  python3 docs/plans/move gen/measurements/plan9/run_plan9.py  (with  /tmp/470M1-oracle  built). Apply plan 9's Phase B rule to the result: on release, write  wishlist3.md   W10  and a dated  MD8 ; otherwise record the  MD7  note. Record the  uo  result for an owner decision and write  report9.md ; run  make test  before and after.


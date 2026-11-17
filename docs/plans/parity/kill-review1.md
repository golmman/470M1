# Kill review 1 — after plans 2–3

**Date:** 2026-11-17. **Scope:** `parity` plans 2–3 (the first review after two
plans). **Inputs:** `initiative.md`, `report2.md`, `report3.md`,
`measurements/plan3/`, `docs/bench.md`. Read-only probes: `cargo test` (44
green), the plan3 parsed results, the seed code, and the three upstream pins.
This review implements nothing.

## 1. Absolute distance to the goal

`P5` = all three cells proved at 10 s, geomean `<= 1.0` vs `provari --guide
hand`, every claim accepted by both checkers.

| Reading | `core` | `easy-win` | `deep-win` | Geomean |
| --- | ---: | ---: | ---: | ---: |
| Product default, seed 0, protocol a | 0.987 | 0.988 | 0.870 | **0.947** |
| Product default, seed 0, protocol b (both export) | 0.987 | 0.988 | 0.870 | **0.947** |
| Protocol median, seeds 0–2, protocol a | 1.062 | 0.988 | 0.854 | 0.964 |
| Session median over four passes | — | — | — | 0.983 |

**The goal is met on the product default: all three cells are below 1.0 and
the geomean is 0.947.** The reported `core` gap is not a product gap. It is
the seed protocol: seed 0 reads 0.987, seed 1 reads 1.062 and seed 2 reads
1.121 on `core`. `Dfpn::perturb` scales the guide's step by ±1% per position
(`src/dfpn.rs:724-735`), so it reorders subtrees rather than breaking ties.
That contradicts D8 ("a seed perturbs only tie-breaks") and the CLI help
("perturbs tie-breaks deterministically"). The 9-run median is therefore a
median over two searches that are not the product's search.

The export asymmetry is real but not the gap at seed 0: protocol a charges the
candidate's 26 ms export and not the reference's; protocol b charges both and
reads the same 0.947 at seed 0. The 0.983 session median is the seed protocol
plus host drift, not a missing lever.

**Absolute distance: zero on the stated metric for the product default.** The
only open item is a measurement-protocol defect (D8), not a speed gap.

## 2. If we started today, would we build this?

**Yes for the architecture and the checker; no for the seed protocol and the
export asymmetry.** Copying provari's board/moves/table/proofs/DF-PN and
keeping the checker on shakmaty was the right way to get a trustworthy
baseline, and it is the prerequisite for `movegen`. The seed should have been
tie-break-only from day one; the current form is a bench-driven perturbation
that makes the product look slower than it is. The export fix is not needed
for the goal and should not be a plan of its own.

## 3. Which initiative or plan would you cancel?

**Cancel backlog item 6, "Parity margin (optional)".** Do not open a plan4 to
buy `core` margin: there is no product margin to buy. Do **not** cancel
`movegen`; it is the next value. Fold the seed correction into a small
measurement-protocol fix (hardening item 4, or a short plan4 if the owner
wants a clean `P5` row first). Cancel the export micro-optimization as a
goal-driven item; keep it only if it falls out of hardening for free.

## 4. Prior-art delta

- `provari` HEAD = `bceb1ee` — unchanged from the pin.
- `atomi` HEAD = `a4802e7` — unchanged.
- `atomic_solver` HEAD = `14abcf2` — unchanged.
- `golmman/atomic_movegen` exists at `6e04488` (MIT), not yet pinned; it is
  the `movegen` dependency, not a parity lever.
- No new solver, lever or upstream commit changes the parity conclusion.

## 5. Smallest next experiment

**Fix `Dfpn::perturb` to perturb only tie-breaks** (or drop the step scaling
and jitter only equal-scored children), add a test that seeds 1–2 do not
change `core`'s tree beyond tie-break noise, and re-run one 10 s pass.
Expected: `core` median returns to ~0.99, geomean ~0.95, and the `P5` row
becomes robust instead of sitting inside the host's noise band. Cost ~1–2 h;
no product speed change.

Cheaper fallback, zero code: record the product default (seed 0) as the `P5`
row and treat seeds 1–2 as a robustness diagnostic. That is honest but leaves
D8 violated.

## Recommendation

**Close `parity` and open `movegen`.** `P5` is met on the product default
(0.947, all three cells < 1.0, 9/9 accepted by both checkers). Record the seed
finding as a measurement-protocol defect, do the tie-break fix as a small
hardening item, and do not spend a plan on margin.

# Kill review 1 — `movegen`, after plans 4–12 (overdue)

**Date:** 2026-11-20. **Scope:** the whole initiative; the fixed prompt is run
now because no review followed plans 4, 6, 8 or 10 (flagged in `report9`,
`report11`, `report12`). **Inputs:** `initiative.md`, `decisions.md`
(`MD1`–`MD9`), `hot-path-audit.md`, `report6`/`9`/`11`/`12`, `docs/bench.md`,
`measurements/plan12/`, and the probe in [`measurements/kill-review1/`](measurements/kill-review1/README.md).
Read-only: `cargo test` (green), `grep -rn shakmaty src/` (checker only), the
probe below. This review implements nothing and appends no ledger row.

## 1. Absolute distance to the goal

| Goal | State | Distance |
| --- | --- | --- |
| Prover on `atomic-movegen` (`MD1`) | **met** (plan 12) | 0 |
| Checker independent (`D4`/`D11`) | **met** (`check.rs` only) | 0 |
| `MD6` no regression | **miss** | row **1.013** cand/old (probe 1.024) |
| Product row vs provari | **1.001** (3/3, 9/9) | 0.1% on the letter; **0.944** search-only |

The `MD6` miss is one number with two components, and only one of them has
ever been measured. Paired 5-round probe, same host, same 10 s flags
([`results.json`](measurements/kill-review1/results.json)):

| Cell | with proof | search only | export, cand | export, old |
| --- | ---: | ---: | ---: | ---: |
| `core` | 1.110 | **1.084** | 51 ms / **0.770e9** | 34 ms / 0.486e9 |
| `easy-win` | 1.075 | **1.051** | 9 ms / 0.117e9 | 5 ms / 0.068e9 |
| `deep-win` | 0.899 | **0.877** | 2.490e9 | 1.565e9 |
| **Geomean** | **1.024** | **1.000** | | |

The probe pairs the two binaries directly (the row's 1.013 is two separate
candidate-vs-provari passes), so it is a decomposition, not a replacement row.

The bench row runs the candidate **with `--proofs`** (protocol `a`,
`docs/bench.md`) but every instruction witness since plan 5 ran **without**
it. So plan 12's "parity" (`core` +0.061%) covers the search only:

| `core` | search | with proof | ratio |
| --- | ---: | ---: | ---: |
| plan 12 product | 7.0616e9 | 7.8317e9 | — |
| plan 2 reference | 6.5694e9 | 7.0558e9 | — |
| ratio | **1.075** | **1.110** | export delta **0.284e9** |

The export delta is **58%** of the 0.492e9 `core` instruction gap, and the
export itself is 1.58–1.72× plan 2's on all three cells (0.770/0.486,
0.117/0.068, 2.490/1.565). It is 9.8% / 7.1% / 3.1% of the candidate's
instructions while the proof file is 656 bytes. Symbol attribution
(`core`, with minus without): `legal_in` +203M, `Board::is_pseudo_legal`
+92M, `PositionView::checkers` +67M, `play` +64M, `pseudo_legal` +63M,
`memmove` +54M, `format::*` +56M — i.e. the per-candidate rules work, not the
format logic.

On the reference side the same walk is free: provari writes a 1021-position
proof for `core` and its wall moves by **+4.7 ms** (noise; `easy-win` −4.6,
`deep-win` −22.9). The plan 2 port already paid 34 ms for the same walk, so
the export was ~7× the oracle before the crate and ~10× now. **Neither gap is
explained by a committed measurement**: the walk is copied from provari, the
moves are identical, and no plan has ever instrumented it.

Against provari, the product row's 1.001 is likewise the export:
`core` 0.528/0.516 = 1.022, `easy-win` 0.136/0.135 = 1.004, `deep-win`
5.424/6.614 = 0.820, geomean **0.944** — the parity closure value (0.945) on
the search, with the candidate's export charged and the reference's not.

Two instrument facts bound any claim: `MD6`'s anchor 0.972 is stale (the same
binary re-reads 0.987, 0.987, 0.991, 0.991, 0.988), and the row's 3-repeat
medians at 0.13–0.6 s are at the resolution the parity review already called
"the edge of the instrument". A perfect export recovery (export back to plan
2's cost) is worth ~2.4 wall points: **0.998** by the wall estimator and
**1.010** by the instruction proxy — at or inside the noise, not a robust
pass.

## 2. If we started today, would we build this?

**Yes for the structure, no for the two instruments that now disagree.**
Owning `atomic-movegen`, keeping the checker on shakmaty, and building the
differential before the adapter were right: the differential caught the
en-passant explosion, `W10` paid 0.669e9, `unsafe-optimizations` 0.090e9, and
the crate is now the only rules layer with a byte-identical `.pvr`. We would
build that again.

We would not (a) measure the performance witness with a different command
from the one the row charges, (b) let `MD6` sit on an anchor whose same-session
value drifts 1.5–2.5%, or (c) price the residual from a split taken two
releases earlier (plan 6's buckets predate `W10`/`uo`/`M8`). If starting
today, the witness *is* the row: `--proofs` on, same protocol, same binary.

## 3. Which initiative or plan would you cancel?

**Cancel any plan that chases the last percent through search, evaluation,
node geometry or the table.** `MD1` and the non-goals forbid search changes,
`MD7` priced the rules residual, and the audit's crate-led ceiling (~1.035×)
is already beaten (1.013 row). The remaining buckets are not rules work.

**Do not cancel the export path as a question, but do not promise it either.**
It is the one lever inside the prover's boundary, it is 58% of `core`'s
instruction gap, and it is unexplained. A single plan is justified only if the
owner wants `MD6`'s letter; its expected land is 0.998–1.010, i.e. a coin
flip. Otherwise close `movegen` now on `MD1`'s terms with the dated `MD6` miss.

## 4. Prior-art delta

- provari `bceb1ee` — unchanged (pin). atomi `a4802e7` — unchanged (pin).
  atomic_solver `14abcf2` — unchanged. No new solver or lever.
- `golmman/atomic_movegen` `main` = **`0aaf958`** = our pin; no new commit and
  **no `v2.11.0` tag** (the highest is `v2.10.0`; the crate is published
  from `main`). `PROVENANCE.md` labels the pin `v2.11.0`: the version is right, the
  tag label does not exist upstream. Label fix only, no pin change.
- GitHub delta: new atomic engines (e.g. `Atomic-Stockfish`) are UCI engines,
  not provers; nothing changes the conclusion.

## 5. Smallest next experiment

**One counted split of the proof-export walk on `core`** — a scratch build
that counts `format::prove`'s candidate loop, `wins_in_two`, `judge` and the
movegen calls they pay (the plan 6/8 method, no `src/` edit), ~1–2 h.
Pre-register: if **≥0.31e9** of the 0.770e9 is re-derivable — answers the
proof table already holds, or positions already proven in the same walk — a
one-plan encoder change with byte-identical `.pvr`, identical counters and
9/9 accepted is worth it. If the cost is per-candidate legality (the `MD7`
residual), nothing in this initiative can close it: close with the miss.

Zero-code fallback: record the search-only geomean (0.944 vs provari, 1.000
vs plan 2) as a diagnostic column beside the claim row, and say in the ledger
that the row's gap is the export. The letter still reads 1.001/1.013.

## Recommendation

**Close `movegen`** on `MD1`'s done definition (prover replaced, checker
independent, differential 0/0, `.pvr` byte-identical), keep `MD6` as a dated
miss with this decomposition, and add the export split as a single optional
follow-up only if the owner wants the letter. Fix the `PROVENANCE.md` tag
label and re-anchor `MD6`'s reference at the next row. Do not open a search
initiative for 1 point that the instrument cannot resolve.

# Initiative: `movegen` — replace shakmaty with `atomic-movegen`

## Status

**Open (2026-11-17).** Plan 3 applied the `MD4` rename and moved the prover's
rules hot path onto `atomic-movegen` with native `Legal` keys (`M1`); plan 4
adopted 2.10.0 (`W6`-`W9`). The differential is green (`mismatches: 0`,
`key_mismatches: 0`, EP 87/0/0) and the checker stays on shakmaty. Plan 5
measured the adopted tree against the `MD6` bar: 3/3 cells, 9/9 accepted,
`D14` identical, geomean **1.094** vs the same-session plan 2 binary's
**0.997** (candidate/old **1.107**). The bar is **not held**: a regression
against it, at the pre-recorded ~1.12× crate-led ceiling, with the `W7`
(121.9) and `W8` (215.2) per-call targets still missed. The adoption stays
(`MD1`); `M1`'s performance part and `M6` stay open, and there is no `M8`
handoff. Plan 6 resolved the non-rules residual (`M11`): rules are +0.62e9 of
the 1.435e9 `core` gap, node geometry +0.36e9 (51.7%, but only +0.15e9 once
the `attackers_to` inlining boundary is accounted for), generation +0.13e9,
and the `NodeFacts` formulation change (`C1`) was falsified at **+0.038e9**
instructions with identical counters. Tree unchanged, no ledger row. Plan 7
adopted the eval-geometry formulation (`M12`): one per-node pass reusing the
`Mover`'s check masks, **−0.1879e9** instructions on `core` (8.0041e9 ->
7.8162e9) with identical counters, mate and `.pvr`; 10 s geomean **1.056**
against a same-session plan 2 binary at **0.987** (candidate/old 1.070, from
1.107). The gap is now 1.190×; `MD6` is still not held. Plan 8 decided the
rules endgame: the exposed `FastLegality` ladder was correct but **slower** on
`core` in every shape (best **+0.1641e9**, census projection ≤0.125e9), so the
rules residual is accepted as the dated `MD7` miss; no fourth release, no
`wishlist3`. `M12` is the last measured prover-side gain; `M1`'s performance
part and `M6` close as a documented miss, and `M8` is the only open item. See
[`report5.md`](report5.md), [`report6.md`](report6.md), [`report7.md`](report7.md),
[`hot-path-audit.md`](hot-path-audit.md) and [`wishlist2.md`](wishlist2.md).

## Goal

Replace `shakmaty` completely in the prover with `atomic-movegen`: parsing,
identity/hash, move generation, effects, legality, terminality, proof paths and
line display. No stage may regress the 10 s bench; the stretch goal is a
measured improvement over the shakmaty-based prover.

The checker is the one exception: it stays on `shakmaty` and shares no code
with the prover. The two rules implementations must remain independent, so a
movegen bug cannot hide in both the prover and the checker.

## Prerequisites

- `parity` `P5` is recorded: all three bench cells proved at 10 s, geomean
  wall `<= 1.0` against `provari --guide hand`, every claim accepted by our
  checker and `provari check`. **Met:** plan 5 closure row, geomean 0.945.
- The parity prover's rules surface is documented: position parsing, legal
  moves, terminal outcomes, hashes, en passant, castling, promotions and
  insufficient material. **Met:** [`rules-surface.md`](rules-surface.md).
- A pinned `atomic-movegen` version is chosen and recorded in
  `PROVENANCE.md`/`Cargo.toml`. **Met:** 2.10.0 (`5083647`), default features.
- A differential harness exists that compares the two rules implementations
  on the fixtures, the bench corpus and random play-out positions. **Met:**
  [`tests/differential.rs`](../../../tests/differential.rs) is green after
  plan 2; numeric keys stay a recorded divergence, see
  [`report2.md`](report2.md) and `MD2`.

## Non-goals

- **No checker change.** The checker remains on shakmaty. A movegen
  replacement in the checker would destroy the independent-veto property.
- **No search-formulation change.** The search stays DF-PN with the hand
  guide. Movegen work is a rules-layer replacement, not a new solver.
- **No proof-format change.** The compact `.pvr` v1 format and its move order
  stay unchanged.
- **No web, UCI, tablebases, NNUE/model, portfolio or parallel search.**
  Those remain out of scope unless a separate initiative says otherwise.
- **No silent rules divergence.** A difference between shakmaty and
  `atomic-movegen` is a finding, not a convenience.

## Decisions

One-line resolutions; full text and rationale in [`decisions.md`](decisions.md).

| # | Decision | Resolution |
| --- | --- | --- |
| MD1 | Definition of done | Complete prover replacement; checker stays shakmaty; no bench regression, stretch faster |
| MD2 | Key strategy | Native `atomic-movegen` keys first; W5 only if measured slower; end state prover-only movegen |
| MD3 | EP-blind `threats` | Census first; change only with measured evidence |
| MD4 | Product name | Final name `470MI`; package/lib `atomic470mi`, binary `470MI`; rename before the first adapter bench |
| MD5 | Hot-path boundary | Direct crate calls on the rules path; conversions only at the FEN/checker/proof edge; a needed conversion is a wishlist item |
| MD6 | What "no regression" means | Same-session plan 2 reference, 3/3 and 9/9, reference near 0.972; `1.0` against provari is not the bar |
| MD7 | Rules residual | Accepted as a dated `MD6` miss (plan 8: ladder +0.1641e9, projection ≤0.125e9); no fourth release |

## Upstream ownership and wishlists

`golmman/atomic_movegen` is maintained by this project. The dependency stays a
released, pinned crate: no fork and no path dependency is committed. When the
adapter needs an API or behaviour change, record it in `wishlistN.md` here and
implement it upstream. A wishlist item is a request, not a promise and not a
plan.

A new `atomic-movegen` release is integrated only when the owner triggers it.
That is a normal `planN.md`/`reportN.md` cycle: update the pin in
`PROVENANCE.md` and `Cargo.toml`, run the differential harness and the gates,
and measure the bench. No implicit sync.

## Candidate work items

These are placeholders; the next plan turns them into concrete items.

| # | Item | Note |
| --- | --- | --- |
| M1 | **Rules adapter** | Wrap `atomic-movegen::Board` behind the prover's position interface; **performance part closed as a documented `MD6` miss** (plan 8, `MD7`); parser removal follows in `M8` |
| M2 | **Move/effect layer** | Replace the parity `Mover`/`Effect` precomputation with `atomic-movegen`'s movegen and effect APIs where they are faster |
| M3 | **Hash/key choice** | Use native movegen keys unless the bench shows a key-driven regression; W5 is the fallback |
| M4 | **Terminality** | Use `atomic-movegen`'s outcome/terminal APIs; preserve the checker's insufficient-material rule |
| M5 | **Differential harness** | Compare legal moves, outcomes, hashes and proof acceptance on fixtures, corpus and random play-outs |
| M6 | **No regression, stretch faster** | Re-run the 10 s bench at every stage; a regression is a finding, not an adoption; **closed as a documented `MD6` miss** (plan 8, `MD7`) |
| M7 | **Upstream asks** | File measured API requests in `wishlistN.md`; integrate a release only by an owner-triggered plan/report and pin update |
| M8 | **Complete shakmaty removal** | Parser, display, proof paths and line output leave the prover; `check.rs` stays shakmaty |
| M9 | **Rename** | Apply `470MI` / `atomic470mi` before the first adapter bench; historical docs stay unchanged |
| M10 | **Hot-path cost** | Adopt `W6`-`W9` lean APIs; no translator; measured per call, then by the 10 s row |
| M11 | **Non-rules residual** | The gap left after the rules layer: node geometry, generation adapter, tree, `null`; **closed negative** (plan 6: split resolved, `NodeFacts` falsified at +0.038e9) |
| M12 | **Eval geometry** | One per-node pass over roles reuses the `Mover`'s check masks; **adopted** (plan 7: −0.1879e9 on `core`, identical tree/`.pvr`; geomean 1.056, `MD6` still missed) |
| M13 | **Rules endgame decision** | Expose `FastLegality` and measure the ladder on `core`; **closed negative** (plan 8: best +0.1641e9, projection ≤0.125e9; dated `MD7`, no release) |

## Gates

- The differential harness reports zero disagreements on the frozen fixtures
  and the bench corpus, and no proof is accepted by one rules implementation
  and rejected by the other.
- The checker still uses shakmaty only and shares no code with the prover.
- The bench does not regress against the same-session plan 2 reference (`MD6`:
  3/3, 9/9, reference near 0.972); a regression is a finding, not a silent
  adoption. The stretch goal is a measured improvement.
- A new `atomic-movegen` release enters only by owner trigger, a
  `planN.md`/`reportN.md`, a pin/provenance update and the same gates.
- `make test`, `make test-full` and `make veto` green.

## Risks

| Risk | Mitigation |
| --- | --- |
| Rules divergence (en passant, castling, promotions, insufficient material) | Differential harness; checker veto; classify and report |
| Checker loses independence | D4/D11: checker stays on shakmaty |
| Movegen is slower on the prover's hot path | Measure before adopting; keep the parity path until a gate passes; `MD2` key fallback |
| Proof format changes accidentally | Pin the format tests; the checker is the external contract |
| Complete removal needs a missing display/parser API | Wishlist it; do not keep shakmaty in the prover as a silent exception |
| Rename breaks commands, tests or bench paths | Do it as one mechanical change before the first adapter bench; update current docs only |
| Scope creep into search tuning | Movegen is a rules-layer initiative; search changes need their own item and gate |

## History

- **2026-11-19** — plan 8 closed the rules endgame as a dated `MD7` miss. The
  exposed `FastLegality` ladder is oracle-equivalent (counters, mate and
  `.pvr` identical) but costs **+0.1641e9** instructions on `core` in its best
  shape (built-in oracle +218.1e6, counts-only +290.0e6, hybrid
  `decide_or_none` +164.1e6), against a 0.31e9 release threshold. The census
  found 82.0% of the 10.01M fallback candidates in check, where the ladder
  defers to the oracle; it decided 1.56M calls and projects at most 0.125e9.
  No fourth release, no `wishlist3`, no pin or ledger row. `M12` is recorded
  as the last prover-side gain; `M1`'s performance part and `M6` close as a
  documented miss, and `M8` is the only open item.
  [`report8.md`](report8.md), [`measurements/plan8/`](measurements/plan8/README.md).
- **2026-11-19** — plan 7 adopted `M12`, the eval-geometry formulation: walk
  the pieces by role and reuse the `Mover`'s per-role check masks for the side
  to move, building only the defender's table once. Counted build
  pre-registered `N − R` = 19 per `evaluate` (median) and a 3,555,443-call
  reduction; the falsifier measured **−0.1879e9** instructions on `core`
  (8.0041e9 -> 7.8162e9, outside both spreads) with identical counters, mate
  and `.pvr`. 10 s row 3/3, 9/9 accepted, `D14` identical, geomean **1.056**
  against a same-session plan 2 binary at **0.987** (candidate/old 1.070, from
  1.107); `MD6` still not held. Remaining gap 1.247e9, half rules.
  [`report7.md`](report7.md), [`measurements/plan7/`](measurements/plan7/README.md).
- **2026-11-19** — plan 6 resolved the non-rules residual and closed `M11` as
  a negative. Counted build (15 sites) + `perf record -c 500000` on `core`
  reconcile to −0.09%; the 1.435e9 gap is rules **+0.62e9**, node geometry
  **+0.36e9** (51.7% of the 0.693e9 non-rules residual, 30.5% if
  `attackers_to`'s plan-2 inlining is accounted for), generation +0.13e9,
  tree +0.12e9, search/table +0.25e9. Legality measures 120.9 per call from
  the whole binary against plan 5's 121.9. `C1` (`NodeFacts`) reached the
  adoption rule and failed its falsifier: **+0.038e9** instructions, counters
  identical, so the accessor traffic it removes is already inline field
  reads. Reverted; binary bit-identical to plan 5's. No tree change, no
  ledger row. [`report6.md`](report6.md),
  [`measurements/plan6/`](measurements/plan6/README.md).
- **2026-11-19** — plan 5 decided the `M1` question on the `MD6` bar: the
  adopted 2.10.0 tree measured 1.208 / 1.158 / 0.936, geomean **1.094**,
  3/3 and 9/9 accepted, `D14` identical, against a same-session plan 2 binary
  at 1.014 / 1.015 / 0.962, geomean **0.997** (candidate/old **1.107**). The
  bar is not held; the row is a regression against it, within the
  pre-recorded ~1.12× crate-led ceiling. `core` is 8.004e9 instructions vs
  6.569e9; `W7` 121.9 and `W8` 215.2 remain missed. No source, search,
  evaluation, proof or checker change; no `M8` handoff. Ledger row and pins
  appended. [`report5.md`](report5.md),
  [`measurements/plan5/`](measurements/plan5/README.md).
- **2026-11-19** — plan 4 pinned and adopted `atomic-movegen` 2.10.0
  (`5083647`): native move storage, `LegalityContext`, `ChildEffect` and
  `null_for_advanced`, with the `Setup` null rebuild and per-candidate
  translators removed. Differential green, gates and veto green, 9/9
  accepted; 10 s row 3/3, geomean **1.099** (old binary 0.988), a near miss
  with the per-call `W7`/`W8` targets still open. `core` 8.004e9 vs plan 2's
  6.569e9. [`report4.md`](report4.md), [`measurements/plan4/`](measurements/plan4/README.md).
- **2026-11-19** — hot-path audit and cost wishlist. The shipped
  `EffectContext`/batch-key API is measured **slower** than the adapter's
  current path (394 vs ~200 instructions per candidate; +17.9% on `core`), as
  is caching node state in `Position` (+16.6%). The rules layer is 31% of all
  instructions; the optimistic ceiling of a crate-led fix is ~1.12× the plan 2
  binary. Owner decisions: keep the movegen rules layer and optimise upstream
  (`MD1` reaffirmed), no translator on the hot path (`MD5`), same-session
  bench bar (`MD6`). [`hot-path-audit.md`](hot-path-audit.md),
  [`wishlist2.md`](wishlist2.md); plans renumbered so [`plan4.md`](plan4.md)
  adopts the next release and [`plan5.md`](plan5.md) measures it.

- **2026-11-19** — plan 3 renamed the product to `470MI`/`atomic470mi` and
  wrapped `atomic_movegen::Board` behind `Position` with native `Legal` keys.
  Differential green (0 mismatches, 0 key mismatches, EP census 87/0/0); all
  gates and 9/9 checker verdicts green. The 10 s row is 3/3, geomean **1.206**
  (plan 2: 0.972), a regression finding; same-session old binary 0.987.
  [`report3.md`](report3.md).
- **2026-11-19** — owner resolved the four open decisions: complete prover
  replacement with no regression and a faster stretch goal (`MD1`), native
  keys measured first with W5 as fallback (`MD2`), an EP `threats` census
  (`MD3`), and the final name `470MI` (`MD4`). [`decisions.md`](decisions.md).
- **2026-11-18** — plan 2 fixed en-passant immediate explosions in
  `moves::exploding_capture` and `Mover::exposes`; full sweep green (0
  mismatches), gates green, 10 s row 3/3 geomean 0.972, 9/9 accepted.
  [`report2.md`](report2.md); numeric Zobrist keys remain a plan 3 decision.
- **2026-11-18** — plan 1 pinned 2.9.0 (`af3faf0`), added the differential
  harness and stopped on two findings: the prover misses en-passant immediate
  explosions (minimal FEN `8/6kp/8/6P1/8/8/8/K7 b - - 0 1`, `h7h5`), and
  numeric Zobrist keys differ. [`report1.md`](report1.md); no adapter.
- **2026-11-18** — recorded upstream ownership and the wishlist protocol;
  opened [`wishlist1.md`](wishlist1.md) with the legal-EP child key,
  Lichess-style FEN acceptance, immediate-explosion query and analysis-view
  asks. No code, no dependency, no pin change.
- **2026-11-17** — initiative opened after `parity` closed at `P5`. Rules
  surface documented; `atomic-movegen` `6e04488` (v2.8.0, MIT) recorded as the
  candidate pin. No plans, no code, no dependency yet.

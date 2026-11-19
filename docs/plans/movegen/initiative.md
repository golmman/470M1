# Initiative: `movegen` — replace shakmaty with `atomic-movegen`

## Status

**Open (2026-11-17).** `parity` reached `P5`; its closure row is in
[`../../bench.md`](../../bench.md). Plan 1 delivered the differential harness
and found two blockers; plan 2 fixed the en-passant immediate-explosion gap.
Plan 3 applied the `MD4` rename and the `M1` adapter: the prover now uses
`atomic_movegen::Board` for move generation, pseudo-legality, legality,
effects/child keys and null moves, with movegen's native `Legal` keys and the
`MD3` census recorded (87 EP children, 0 verdicts, 0 differences). The full
differential is green (`mismatches: 0`, `key_mismatches: 0`). The 10 s bench
**regressed** to 3/3, 9/9 accepted, geomean **1.206** (1.352 / 1.250 / 1.038);
a same-session plan 2 binary reads 0.987, so the loss is the adapter's
per-node cost. The adoption bar did not hold and is reported, not tuned away.
See [`report1.md`](report1.md), [`report2.md`](report2.md),
[`report3.md`](report3.md) and [`wishlist1.md`](wishlist1.md) (W1-W3 landed,
W4 open, W5 proposed). The next step is an owner decision on the regression,
then `M8`/performance; the checker stays on shakmaty.

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
  `PROVENANCE.md`/`Cargo.toml`. **Met:** 2.9.0 (`af3faf0`), default features.
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
| M1 | **Rules adapter** | Wrap `atomic-movegen::Board` behind the prover's position interface; parser removal follows in the same initiative |
| M2 | **Move/effect layer** | Replace the parity `Mover`/`Effect` precomputation with `atomic-movegen`'s movegen and effect APIs where they are faster |
| M3 | **Hash/key choice** | Use native movegen keys unless the bench shows a key-driven regression; W5 is the fallback |
| M4 | **Terminality** | Use `atomic-movegen`'s outcome/terminal APIs; preserve the checker's insufficient-material rule |
| M5 | **Differential harness** | Compare legal moves, outcomes, hashes and proof acceptance on fixtures, corpus and random play-outs |
| M6 | **No regression, stretch faster** | Re-run the 10 s bench at every stage; a regression is a finding, not an adoption |
| M7 | **Upstream asks** | File measured API requests in `wishlistN.md`; integrate a release only by an owner-triggered plan/report and pin update |
| M8 | **Complete shakmaty removal** | Parser, display, proof paths and line output leave the prover; `check.rs` stays shakmaty |
| M9 | **Rename** | Apply `470MI` / `atomic470mi` before the first adapter bench; historical docs stay unchanged |

## Gates

- The differential harness reports zero disagreements on the frozen fixtures
  and the bench corpus, and no proof is accepted by one rules implementation
  and rejected by the other.
- The checker still uses shakmaty only and shares no code with the prover.
- The bench does not regress against the current baseline (plan 2: geomean
  0.972); a regression is a finding, not a silent adoption. The stretch goal
  is a measured improvement.
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

# Initiative: `movegen` — replace shakmaty with `atomic-movegen`

## Status

**Open (2026-11-17).** `parity` reached `P5` and its closure row is recorded
in [`../../bench.md`](../../bench.md). The prover's rules surface is documented
in [`rules-surface.md`](rules-surface.md), and the candidate dependency is
pinned: `golmman/atomic_movegen` `6e04488fd737af17b2edbe101038f3f69754f3c6`
(v2.8.0, MIT). No plans yet; plan 1 starts with the differential harness.
[`wishlist1.md`](wishlist1.md) records the integration asks against that pin.
We own the upstream crate; a release is integrated only by an owner-triggered
plan/report.

## Goal

Replace the parity prover's `shakmaty` rules layer with `atomic-movegen`
while preserving correctness, proof acceptance and wall-time parity, then use
the new movegen's APIs to improve the prover.

The checker stays on `shakmaty`. The two rules implementations must remain
independent, so a movegen bug cannot hide in both the prover and the checker.

## Prerequisites

- `parity` `P5` is recorded: all three bench cells proved at 10 s, geomean
  wall `<= 1.0` against `provari --guide hand`, every claim accepted by our
  checker and `provari check`. **Met:** plan 5 closure row, geomean 0.945.
- The parity prover's rules surface is documented: position parsing, legal
  moves, terminal outcomes, hashes, en passant, castling, promotions and
  insufficient material. **Met:** [`rules-surface.md`](rules-surface.md).
- A pinned `atomic-movegen` version is chosen and recorded in
  `PROVENANCE.md`/`Cargo.toml`. **Pin recorded in `PROVENANCE.md`; not yet a
  dependency.**
- A differential harness exists that compares the two rules implementations
  on the fixtures, the bench corpus and random play-out positions. **Remaining:
  plan 1's first phase.**

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

These are placeholders; the first plan will turn them into concrete items
after the parity handoff.

| # | Item | Note |
| --- | --- | --- |
| M1 | **Rules adapter** | Wrap `atomic-movegen::Board` behind the prover's position interface; keep shakmaty for parsing and the checker |
| M2 | **Move/effect layer** | Replace the parity `Mover`/`Effect` precomputation with `atomic-movegen`'s movegen and effect APIs where they are faster |
| M3 | **Hash/key parity** | Match the prover's board-only key and legal en passant handling; pin against shakmaty |
| M4 | **Terminality** | Use `atomic-movegen`'s outcome/terminal APIs; preserve the checker's insufficient-material rule |
| M5 | **Differential harness** | Compare legal moves, outcomes, hashes and proof acceptance on fixtures, corpus and random play-outs |
| M6 | **Wall parity** | Re-run the bench; a movegen change that regresses wall time is not adopted without a measured win |
| M7 | **Upstream asks** | File measured API requests in `wishlistN.md`; integrate a release only by an owner-triggered plan/report and pin update |

## Gates

- The differential harness reports zero disagreements on the frozen fixtures
  and the bench corpus, and no proof is accepted by one rules implementation
  and rejected by the other.
- The checker still uses shakmaty only and shares no code with the prover.
- The bench remains at `P5` or better; a regression is a finding, not a
  silent adoption.
- A new `atomic-movegen` release enters only by owner trigger, a
  `planN.md`/`reportN.md`, a pin/provenance update and the same gates.
- `make test`, `make test-full` and `make veto` green.

## Risks

| Risk | Mitigation |
| --- | --- |
| Rules divergence (en passant, castling, promotions, insufficient material) | Differential harness; checker veto; classify and report |
| Checker loses independence | D4/D11: checker stays on shakmaty |
| Movegen is slower on the prover's hot path | Measure before adopting; keep the parity path until a gate passes |
| Proof format changes accidentally | Pin the format tests; the checker is the external contract |
| Scope creep into search tuning | Movegen is a rules-layer initiative; search changes need their own item and gate |

## History

- **2026-11-18** — recorded upstream ownership and the wishlist protocol;
  opened [`wishlist1.md`](wishlist1.md) with the legal-EP child key,
  Lichess-style FEN acceptance, immediate-explosion query and analysis-view
  asks. No code, no dependency, no pin change.
- **2026-11-17** — initiative opened after `parity` closed at `P5`. Rules
  surface documented; `atomic-movegen` `6e04488` (v2.8.0, MIT) recorded as the
  candidate pin. No plans, no code, no dependency yet.

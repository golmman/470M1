# Hot-path audit — `atomic-movegen` 2.9.0 against the plan 2 shakmaty shape

**Date:** 2026-11-19. **Pin:**
`atomic-movegen` 2.9.0 (`af3faf0`), `shakmaty` 0.30.1. **Host:**
`c90face26537`.

This file exists because the earlier diagnosis attributed the 10 s regression to
"missing fast paths" that turned out to exist. Everything below is either read
from the two sources or counted with `perf stat`; nothing is estimated unless
it says so.

## Method

- **Per call:** `/tmp/w6-exp/costs` links both crates, parses
  ten FENs once, then runs one operation in a hot loop for 2,000 passes
  (49,600 candidate calls). `perf stat -e
  instructions` around the process, divided by that count.
- **Whole binary:** the product's `core` cell at 10 s, 192 MB, seed 0,
  `perf stat -e task-clock,instructions,cycles`. `nodes` and `child_evals`
  are identical in every variant below (350,430 / 3,896,430), so instruction
  deltas are per-node cost, not tree drift.
- **Call counts per run** are the earlier counter probe, per *expanded* node
  (191,557): legality 43.7, effect 13.6, `pseudo_legal` 1.86, `play` 4.0,
  `populate_state` 1.74, `in_check` 1.45.

## Per-call costs (instructions)

| Operation | `atomic-movegen` 2.9.0 | plan 2 shakmaty shape |
| --- | ---: | ---: |
| pseudo-legal generation, whole position | **833** | 964 (earlier counter probe) |
| `populate_state` (checkers + pins + counts) | **166** | — |
| counts only (two popcounts) | **18** | — |
| legality, per candidate, state populated | **144** | **83** (`occupied_after` + `checked_after`) |
| `is_legal` (repopulates its own state) | **429** | — |
| child occupancy only (blast + toggles) | (inside effect) | **29** |
| effect + child key, batched | **394** (shipped `post_move_and_key_with(Legal)`); **566** (a 3-in-1 patch adding classification + legality) | **50–75** (occupancy + Zobrist delta) |
| `hash_and_rule50_after` + `is_double_push` | **164** | included above |
| `checkers()` | **108** | **0** (cached in `state`) |
| clone + apply a move | **230** (`Board::clone` + `do_move`) | **158** (`Atomic` clone + `play_unchecked`) |
| convert one move to/from the 16-bit prover format | **38** | 0 (the format is native) |

Whole-binary anchors this session: plan 3 HEAD **9.026e9** / 639 ms; plan 2
binary **6.569e9** / 494 ms; the earlier `/tmp/probe4` bundle **8.527e9** / 608 ms.

## Rules-call budget per run

| Call | Calls | Plan 3 cost | Total | Plan 2 shape | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| legality | 8.4M | 144 | 1.21e9 | 83 | 0.70e9 |
| effect + key | 2.6M | 215 | 0.56e9 | 60 | 0.16e9 |
| pseudo + conversion | 0.36M | ~2,200 | 0.79e9 | 964 | 0.35e9 |
| `populate_state` | 0.33M | 166 | 0.05e9 | 0 | 0 |
| `play` | 0.77M | 230 | 0.18e9 | 158 | 0.12e9 |
| `in_check` | ~0.4M | 108 | 0.04e9 | 0 | 0 |
| **Total** | | | **2.83e9** | | **1.33e9** |

So the rules layer is **31% of all instructions**, and the movegen-minus-plan2
part of it is **1.50e9 of the 2.46e9 gap (61%)**. The rest is the 2.4% larger
tree (0.16e9), `Mover`'s accessor/geometry traffic through the crate board,
`null`, and the remaining adapter code.

## What each side actually does

- **Representation.** shakmaty's `Atomic` keeps a mailbox plus role and colour
  bitboards. Plan 2's `Position` copied `colors[2]`, `roles[6]` and `occupied`
  into its own fields, so `legal`/`effect`/`blast` read direct fields.
  `atomic-movegen`'s `Board` keeps the same data as `by_color[2]` +
  `by_type[6]` in a `PositionView`; the prover reaches it through accessor
  methods. - **Moves.** shakmaty's `Move` is an enum. Both engines' hot paths use a
  packed `u16`; `atomic-movegen`'s is the semver-stable
  `to | from | type | promotion` layout, the prover's is the shakmaty-derived
  `from | to | kind`. Plan 3 converts both ways per candidate (~38 each).
- **Legality.** Both recompute the child occupancy and ask whether the mover's
  commoner is attacked. `legal_with_counts` (`src/position.rs:551`) adds the
  castling pass-through rule, en-passant victim removal, commoner counting, and
  a `view` indirection; plan 2's `checked_after` is a single
  `attackers_to(king, occupied) & theirs & !kings` with a touching-kings guard.
  Documented consequence: **`Board::legal` reads only `commoners_count` and
  `them_commoners_count` from `StateInfo`; it never reads `checkers` or
  `pinned`.** Those are used only by `has_legal_move_with_state` and the
  existence screen.
- **Effect and key.** `atomic-movegen` derives a `PostMove` = a whole
  `PositionView` (2+6 bitboards, side, ep, rights, clock) per candidate, then
  an incremental Zobrist delta from the parent hash. Plan 2 did the same
  arithmetic on the parent's own bitboards and kept a per-node key base, so
  the key was a handful of XORs.
- **Null and state.** Plan 2's `null` flipped fields. The adapter rebuilds a
  `Setup` and a `Board`; `null_for` exists but does not advance `rule50`.
- **Pseudo-legal generation.** The crate's staged generator is *faster* than
  plan 2's hand generator (833 vs 964); the adapter gives the gain back through
  per-move conversion.

## Three negative results (measured, whole binary)

| Variant | Instructions | Wall | vs HEAD |
| --- | ---: | ---: | ---: |
| HEAD (plan 3) | 9.026e9 | 639 ms | — |
| control: unmodified prover + patched crate | 9.026e9 | 641 ms | 0.0% |
| E1: `StateInfo` cached in `Position` | 10.521e9 | 728 ms | +16.6% |
| E2: `EffectContext` for effect + child key | 10.637e9 | 729 ms | +17.9% |
| E1+E2: `EffectContext::is_legal` as well | 12.794e9 | 908 ms | +41.7% |

1. **The "hot path" API already shipped and is slower.** `EffectContext` +
   `PostMove` + `post_move_and_key_with` is what the earlier `W6` ask wanted; it is
   394 instructions per candidate against ~200 for the adapter's current
   effect, because it materialises a 72-byte view per candidate.
2. **A single 3-in-1 pass is slower still** (566): one `analyze` + `classify` +
   `is_legal_from` + key costs more than the two cheaper calls it replaces.
3. **Caching node constants in `Position` loses** (+16.6%): children are
   created more often than `Mover::new` runs, so `populate_state` moves from
   0.33M to ~0.5M calls and the `Position` grows by a `StateInfo`.

Correction to the earlier diagnosis: `in_check` measures **108** instructions in
the probe, not 690; the larger figure is not a per-call count and probably
includes the call site. The `Mover::new` and pseudo/effect ratios are unchanged.

## Ceiling for a crate-led fix

Apply every item a plan-2-shaped crate API could remove (legality 83, effect
50–75, no population, no conversion, `play` 158):

`9.026e9 − 1.50e9 (rules delta) − 0.18e9 (native move storage) ≈ 7.35e9`

That is **1.12× the plan 2 binary**, i.e. roughly 12% short of the `3B` bar
(6.57e9), before counting the 2.4% tree difference and `Mover`'s traffic. The
estimate assumes the crate reaches plan 2's *exact* per-call costs, which is
the optimistic bound.

**Consequence.** A release that adopts the whole wishlist is expected to land
near 1.1× the plan 2 binary, not at parity. That is a finding to report, not a
reason to hide the residual in search or evaluation.

## The escape hatch that the goal disallows

The prover could rebuild plan 2's legality on public crate surface: `Board::attackers_to(square, occupied)` plus
`attacks::{bishop, rook, queen, king, pawn, knight}_attacks` (the sliders are
re-exported from `magic.rs`, so no new API is needed). That reaches plan 2's
cost by construction. It is still a second rules implementation inside the
prover and a translator on the hot path — exactly what `MD1` and `MD5` forbid —
so it may be used as a diagnostic bound, never adopted. Recorded so it is not
rediscovered by accident.

## Licence note

Plan 2's hot path is provari-derived (`GPL-3.0-or-later`); the crate is MIT.
The wishlist asks for *behaviour and cost*, and the crate-side code must be
written from the crate's own structures, not copied from this repository. If
the owner prefers to move this repository's tuned code into the crate instead,
that is a licence change for the published crate and needs its own decision.

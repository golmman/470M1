# Report 2 — en-passant immediate explosions

**Plan:** [`plan2.md`](plan2.md). **Item:** none; unblocks `M5`, precedes the
`M1` adapter. **Date:** 2026-11-18. **Session:** execution. **Outcome:** the
prover now sees en-passant immediate explosions; the full differential sweep is
green; the gates and a same-session 10 s bench row pass. No adapter, pin, hash,
checker or proof-format change.

## Outcome

- `moves::exploding_capture` and `Mover::exposes` now consider an en-passant
  capture whose destination is beside the victim commoner, in addition to the
  existing piece targets. The EP branch runs after the piece-target loop, so
  witness order is unchanged.
- The full sweep reports `mismatches: 0`, all three EP counters 0, and two
  identical runs. `key_table_differences` is 27,088, recorded not asserted: the
  two Zobrist tables still differ, as report 1 found.
- `make test`, `make test-full` (including the slow bench cells and the veto)
  and `make veto` are green. All nine bench proofs are accepted by both
  checkers; `D14` counters match.
- Bench, product default (seed 0), 10 s: coverage 3/3, factors 0.981 / 0.978 /
  0.958, geomean **0.972**, 9/9 accepted. The pre-fix diagnostic in the same
  session reads geomean 1.011 with `deep-win` at mate 22; the fixed binary finds
  a 19-move proof accepted by both checkers.

## Phase 0 — baseline

Recorded before any source edit: fast suite and lint green, veto 9/9 plus the
control, release binary kept as `target/release/470m1-prefix` (never committed).
The full sweep was red exactly as report 1 described:

| Count | Before |
| --- | ---: |
| Positions / legal moves / child keys | 26,869 / 538,453 / 538,453 |
| Explosion mismatches (81 per move + 5 node) | 86, all EP |
| `exposes` mismatches | 81, all EP |
| Other mismatches | 0 |
| Key-table numeric differences | 26,869 |

## Phase A — patch

`src/board.rs`, additive only: `Position::maybe_ep_square` exposes the stored
passed square, and `Position::en_passant_move` builds the `EN_PASSANT` move.
The `ep` field, `effect`, `apply`, `legal_ep_hash`, the Zobrist table and
`hash` are untouched.

`src/moves.rs`:

- `ep_explodes(to, victim, taker)` is the shared predicate: the blast on the
  passed square takes the victim commoner and not the taker's own commoner.
- `exploding_en_passant` guards `pos.turn() == side`, requires a passed square
  and a pawn taker, applies `ep_explodes`, and returns the move only when
  `pos.legal` accepts it. `exploding_capture` runs the original piece-target
  loop first, then `or_else` this branch.
- `Mover::exposes` keeps its three quiet answers as `covered` and returns
  `covered || (effect.en_passant && self.ep_exposes(m, effect))`. `ep_exposes`
  uses `(from + to) / 2`, our king after the move filtered by the effect's
  occupancy, and their king as the taker.
- `threats`, `eval`, `gives_check`, `plain`, `format.rs`, `dfpn.rs` and
  `check.rs` are unchanged. `threats` stays EP-blind as planned; its local test
  helper mirrors it.

## Phase B — tests

- `moves::tests::en_passant_explosion`: both witnesses, the child's passed
  square, `effect.en_passant`, `exposes`, legal witness and exploded commoner.
- `moves::tests::en_passant_explosion_negatives`: no taker, and the passed
  square beside both commoners (`d2d4` is rejected).
- `moves::tests::en_passant_witness_order`: with both a piece target on `f6`
  and the EP explosion available, the witness still takes `f6`. This pins the
  proof-contract order.
- The five EP FENs join `matches_the_played_positions` so its per-move
  cross-check covers them.
- `tests/differential.rs`: `ep_fens()` feeds the smoke and full acceptance sets;
  `en_passant_explosion_regression` compares, per legal move, explosion
  presence, witness destination and `Mover::exposes` with
  `EffectContext::exploding_capture_after`; `run_playouts` takes its starts as a
  parameter, so the 3 x 128 sweep keeps its counts and a separate 1 seed x 8
  game EP sweep runs. `tests/fixtures/*.tsv` is untouched.

Two scope details are recorded rather than hidden: the regression skips a
decided child (movegen documents `None` there and the search never asks), and
the suite adds five acceptance FENs, which is why the sweep totals grow by 219
positions and 1,930 legal moves.

## Phase C — gates

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green after one seed-literal grouping fix |
| `cargo test` | green |
| `cargo doc --no-deps` | green |
| `make test` | green |
| `make test-full` | green: full sweep, slow bench cells, veto |
| `make veto` | green: 9/9 accepted, control rejected by both |
| Full sweep determinism | two identical runs |
| `check.rs`, `format.rs`, Zobrist/`hash`, proof order | untouched |

Post-fix sweep: 84 accepted / 11 rejected FENs, 27,088 positions, 540,383
legal moves and child keys, 72,131 immediate explosions, 6,684 decided
children skipped, 642 material exemptions, 27,088 recorded key-table
differences, 392 games / 26,612 plies, 0 mismatches. Elapsed 1.47 s and 1.50 s.

## Phase D — bench

One session, one process at a time, protocol `a`, three repeats, seed 0,
10 s, 192 MB, `--d14`. Driver: plan5's, header only, at
`measurements/plan2/run_bench.py`.

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.523 s | 0.513 s | **0.981** | 17 / 17 | 3,823,720 | 3/3 |
| `easy-win` | 0.136 s | 0.133 s | **0.978** | 15 / 15 | 897,937 | 3/3 |
| `deep-win` | 6.415 s | 6.147 s | **0.958** | 19 / 22 | 54,633,842 | 3/3 |
| **Geomean** | | | **0.972** | | | **9/9** |

`D14` matched the timed counters for all three cells and both checkers
accepted every proof. The pre-fix `prefix.json` reads 1.016 / 0.992 / 1.025,
geomean 1.011, `deep-win` mate 22, `child_evals` 58,229,418. The fix therefore
moved `deep-win`'s tree and wall together; the 19-move proof is shorter, not a
changed move of an existing written proof. `core` and `easy-win` are inside the
reference spread.

## Problems and scope changes

- The first clippy run failed on an ungrouped `u64` literal in the new EP
  play-out seed; the literal is now byte-grouped.
- The plan's bench command writes `env.json` on each run. The claimed
  `results.json` run ran first; its `env.json` was saved, the prefix run came
  second, and the committed `env.json` is the claim's record with explicit
  `prefix_*` fields. No raw transcripts are committed.
- `run_playouts` gained a `starts` parameter (needed by the plan); the existing
  sweep call sites pass `accepted_starts()` exactly as before.
- The regression uses `EffectContext::terminal_after` to skip decided children.
  This is the same contract the full harness already uses, not a new filter.

## Open parts and missing tests

- Numeric Zobrist parity is still open: 27,088 paired positions differ. Plan 3
  must decide keep-our-hash, translate, or wait for an upstream compatible
  table (wishlist W5).
- The full harness still compares explosion existence, not witness identity;
  the new EP regression compares the destination on the EP fixtures only.
- `threats` remains EP-blind by design; a scoring undercount is possible in an
  EP-only attack position. It is unchanged and its local test helper mirrors it.
- No adapter, pin, hash, `check.rs`, `format.rs`, `dfpn.rs`, eval or move-order
  change was made.

## Next steps

1. Plan 3: the rules adapter (`M1`) with the key decision explicit. Run the
   full differential sweep and the same bench protocol.
2. Add an EP-only scoring fixture if a future census finds `threats` making a
   measurable difference; do not reopen this plan for it.
3. The owner's kill review follows this plan; the next plan is the adapter, not
   another EP patch.

## Deliverables

Patch in `src/board.rs` and `src/moves.rs`; tests in `src/moves.rs` and
`tests/differential.rs`; `measurements/plan2/{run_bench.py,results.json,
prefix.json,env.json,README.md}`; one `bench.md` ledger row; this report and the
initiative/status/rules-surface updates.

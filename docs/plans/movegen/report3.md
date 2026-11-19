# Report 3 — rename and the `M1` rules adapter

**Plan:** [`plan3.md`](plan3.md). **Items:** `M1`, `M3`, `M9`, `MD3`.
**Date:** 2026-11-19. **Session:** execution. **Outcome:** the rename,
`atomic-movegen`-backed rules adapter, native keys and EP-`threats` census are
in place; every gate and both checkers are green. The 10 s bench **regresses**
to geomean **1.206** (3/3, 9/9), so the plan's adoption bar did not hold and
the result is reported as a finding, with no search or evaluation tuning.

## Outcome

- The product is `atomic470mi` / `470MI`; current source, tests, scripts,
  `Makefile`, bench README and current bench commands use the new name.
  Historical rows and plans keep `470m1`.
- `board::Position` is `{ board: atomic_movegen::Board, hash: u64 }` on the
  `Legal` key. Move generation, pseudo-legality, legality, child keys and
  `null()` come from the native board; shakmaty remains the typed boundary,
  `to_atomic`/`to_shakmaty`, `Mover`'s geometry and the checker.
- The full differential is green with `key_mismatches: 0`, two identical runs
  and all three EP counters zero. The `MD3` census is 87 `ep_child`, 0
  EP-threat verdicts and 0 EP-threat differences.
- The bench is **not** at parity: factors 1.352 / 1.250 / 1.038, geomean
  **1.206**, 3/3 coverage, 9/9 accepted, `D14` counters identical. A
  same-session old-binary diagnostic reads 0.998 / 1.007 / 0.958, geomean
  0.987, so the regression is the adapter, not host drift.

## Phase 0 — rename

`atomic470m1` → `atomic470mi`, `470m1` → `470MI` in `Cargo.toml`,
`Cargo.lock`, `src/**`, `tests/**` (including `CARGO_BIN_EXE_470MI` and
`Usage: 470MI`), `scripts/veto.sh`, `Makefile`, `bench/README.md`, and the
current command examples in `docs/bench.md`. `docs/plans/**`, `docs/notes/**`,
historical ledger rows and `/tmp/470M1-oracle` were left alone. The rename
passed fmt, clippy and the test suite before any adapter code.

## Phase A — adapter

`src/board.rs`:

- `Position::from_atomic` builds an `atomic_movegen::Setup` from the shakmaty
  position (pieces, side, castling bits, the **stored** EP square, `rule50`)
  and `Board::from_setup`; `hash = board.hash_with(Legal)`.
- Square/bitboard/piece accessors map shakmaty types at the edge
  (`Bitboard(pub u64)`, `Role::King` ↔ `PieceType::Commoner`). The shakmaty
  geometry helpers (`attackers_to`, `checked_after`, `blast`, `ep_explodes`,
  `capture_move`, `to_shakmaty`) remain.
- `pseudo_legal` walks `movegen::for_each_pseudo_legal` into the
  `MaybeUninit` `Moves` and converts the native move encoding to the 16-bit
  format. `is_pseudo_legal` delegates after rejecting the unused promotion
  kind. `legal` is `board.is_legal`; `Mover::new` populates one `StateInfo`
  and the fallback is `board.legal(m, &state)`.
- `effect` drops the hash base: occupancy/capture/counter are computed from
  the position and the child key is native. The risk fallback is implemented:
  `hash_and_rule50_after` classifies once, and only a double push asks for the
  `Legal` key again. `full_effect` is an alias; `child_base`, `ep_hash`,
  `Mover::base` and `Effect::en_passant` are gone.
- `play` clones the board and caches the child `Legal` key. `null()` rebuilds
  a `Setup` with the side flipped, EP cleared and **`rule50 + 1`**; a unit test
  pins that clock.
- `to_atomic` rebuilds the shakmaty setup, so `line`, proof writing and the
  checker boundary are unchanged. `squares`, `blast`'s public copy and
  `occupied_after` were removed where unused.

Other modules changed only for the moved identity: `walk.rs` and `lines.rs`
now key proofs by `Position::from_atomic(pos).hash()` instead of shakmaty's
`crate::hash`, and `format.rs`'s test helpers use the native key.
`check.rs` changed only in the renamed CLI strings.

## Phase B — tests and differential

- `tests/differential.rs`: `atomic470mi`; `compare_node` asserts the prover's
  root/child keys against `board.hash_with`/`hash_after_with(Legal)` and counts
  `key_mismatches` (0). The old numeric-table divergence test is retired;
  `native_keys_match_movegen` covers the standard position, fixtures, corpus
  and EP FENs plus a play-out, and `native_keys_are_deterministic` pins
  repeatability.
- The EP key block now checks only movegen's `Always == Legal` iff the stored
  square is absent or a legal EP capture exists.
- `src/board.rs::matches_shakmaty` keeps moves and placement comparisons and
  drops the shakmaty-Zobrist and `swap_turn` hash assertions. Move sets, not
  generation order, are compared: the native generator's order differs from
  the old hand generator.
- `src/moves.rs` keeps the EP regressions, witness order and
  `matches_the_played_positions`; `ep_threat` restores the old "an enemy pawn
  can take" gate now that `Effect::en_passant` is gone.
- The pinned deterministic core test moved with the native tree: the `core`
  proof is **14** attacker moves (was 17), and the 100,000-node seed-0 counters
  are 100,005 nodes / 1,181,957 evals / 58,248 expansions / (371, 26,198).
  Both checkers accept the 14-move proof. This is a key/move-order tree
  change, not a rules or search change.

## Phase C — `MD3` EP-`threats` census

Per node and legal move the differential now records:

- `ep_child`: the child's side to move has an EP explosion;
- `ep_threats_verdicts`: of those, `threats >= 2`, the case that reaches
  `dfpn.rs`'s `threats >= 2 && exposed` win check;
- `ep_threats_difference`: `threats` differs from a test-only
  `exploding_captures_ep(child, us)` that adds the empty-destination EP target
  and the EP turn guard, on positions without an immediate explosion before the
  move.

Full sweep: **87 / 0 / 0**. The expected result holds: after our move the EP
right is the opponent's, so it can void a double threat through `exposed` but
never count as one of our next-move threats. No eval or `threats` change.

## Phase D — gates

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | green |
| `cargo doc --no-deps` | green |
| `make test` | green |
| `make test-full` | green: full sweep, slow bench cells, veto |
| `make veto` | green: 9/9 proofs accepted, control rejected by both |
| Differential determinism | two identical full runs |
| `make oracle` | built at the pinned provari commit |

Full sweep: 84 accepted / 11 rejected FENs, 27,088 positions, 540,383 legal
moves and child keys, 72,131 explosions, 6,684 decided children skipped, 642
material exemptions, 392 games / 26,612 plies, 0 mismatches, 0 key mismatches
and the three EP counters 0 / 0 / 0.

## Phase E — bench

One session, one process at a time, protocol `a`, three repeats, seed 0,
10 s, 192 MB, `--d14`; driver plan2's, header only.

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.528 s | 0.714 s | **1.352** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.136 s | 0.170 s | **1.250** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.618 s | 6.868 s | **1.038** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.206** | | | **9/9** |

Coverage 3/3; `D14` timed and untimed counters identical; both checkers
accepted every proof. Delta to the plan 2 row's 0.972, cross-session and not a
factor: +0.371 / +0.272 / +0.080, geomean +0.234.

The same-session old-binary diagnostic (`old.json`) reads 0.998 / 1.007 /
0.958, geomean 0.987, with the plan 2 tree (17 / 15 / 19). The adapter's
`deep-win` tree is 18% cheaper in `child_evals` but 3.8% slower in wall; the
small cells lose 25–35%. The regression is per-node adapter cost, not key
quality alone, host drift or proof export (the reported `ms` excludes export).

## Problems and scope changes

- The plan's fallback for a bench regression was tried first:
  `hash_and_rule50_after` with a double-push-only `Legal` correction, plus
  `Mover::new` legality state. It did not close the gap.
- Caching the board's bitboards in `Position` and an occupancy-based legality
  fast path were measured as experiments; neither moved the wall for `core`
  beyond noise and both were reverted. `Position` is the planned
  `{ board, hash }`.
- The native generator changed pseudo-move order, so `board::matches_shakmaty`
  compares move sets; proof order (`format::number`) is untouched.
- `walk`/`lines` needed the native key, not only moved signatures: their
  `crate::hash` lookup was shakmaty's and would never have found a prover move.
- The pinned `dfpn` core proof/test counters moved to 14 moves and the numbers
  above; this follows from native keys plus generator order, not a search edit.
- The `MD3` difference is counted only where no immediate explosion existed
  before the move, mirroring the existing `matches_the_played_positions`
  guard; the unguarded count also reports pre-existing non-EP differences and
  is not an EP finding.

## Open parts and missing tests

- The adoption bar failed. `MD2` says native keys are kept only when they do
  not regress; the adapter must be optimised or the key choice revisited
  before `movegen` can claim no regression.
- The hot path still pays for `Board::pseudo_legal`, `Board::legal` and the
  native effect key on every node. A profile-guided next plan should measure
  moving `Mover`'s geometry to movegen's attack tables, a cached effect
  context, or a hot-path API that returns effect + key in one derivation.
- The EP census is a count, not a search-impact experiment; a future plan that
  makes `threats` EP-aware still needs its own bench.
- The parser, display/`to_atomic` proof path and `crate::hash` remain on
  shakmaty for `M8`; the checker remains shakmaty-only, and
  `uses_nothing_from_the_prover` still passes.

## Next steps

1. Owner decision: accept the adapter code with a regression finding, or
   require a performance plan before committing the `1.206` row.
2. If performance is next, profile the adapter's per-node cost against the
   stored old binary; do not tune search or eval.
3. Otherwise, `M8` removes the remaining shakmaty parser/display surface from
   the prover.

## Deliverables

Rename in the crate/tests/scripts/Makefile/current docs; adapter in
`src/board.rs` and signature touch-ups in `src/moves.rs`, `src/walk.rs`,
`src/lines.rs`, `src/format.rs`, `src/dfpn.rs`, `src/main.rs`;
`tests/differential.rs` native-key and census fields;
`measurements/plan3/{run_bench.py,results.json,old.json,env.json,README.md}`;
the ledger row and pin; this report; `initiative.md`, `rules-surface.md`,
`AGENTS.md` and `PROVENANCE.md` updates.

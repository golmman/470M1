# Report 1 — `atomic-movegen` 2.9.0 pin and rules differential

**Plan:** [`plan1.md`](plan1.md). **Item:** 1 (`M5` harness, `M7` pin).
**Date:** 2026-11-18. **Session:** execution. **Outcome:** pin and harness
delivered; plan **blocked** by two findings; full sweep is not green. No
prover source change, no adapter, no bench row.

## Outcome

- `atomic-movegen = "=2.9.0"` is a default-feature dependency; the packaged
  crate matches tag `af3faf0`, its source is unchanged from the tag, it has
  zero dependencies and the licence gate accepts it.
- `tests/differential.rs` compares FEN acceptance, legal moves, keys, effects,
  immediate explosions and terminality on the fixtures, the bench corpus,
  generated mutations and 3 x 128 seeded play-outs (26,406 plies). It is
  deterministic and runs in about 1.5 s.
- **The full sweep is red by design**: 86 immediate-explosion disagreements,
  all en passant. `Mover::exposes` shares 81 of them. The prover's
  `moves::exploding_capture` and `Mover::exposes` miss an en-passant capture
  whose blast removes the commoner next to the destination.
- **Numeric Zobrist parity does not hold**: the crate ships its own generated
  table. The plan's item 3 premise was false. The harness compares key
  semantics instead (own child key, legal-EP gating, the EP component), records
  the numeric difference, and keeps an explicit failing-if-fixed test.

The plan's gates and deliverables required a green full sweep and numeric key
equality. Neither is reached, so this report does not claim plan 1 complete.

## Phase 0 — audit and baseline

| Gate | Baseline result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | green (all fast tests) |
| `make test` | green (lint, licence, fast tests) |
| `make test-full` | green before this change (10 s bench cells + veto) |
| `make veto` | 9/9 accepted, 1/1 control rejected by both checkers |
| `cargo doc --no-deps` | green |

The `P5` closure row was re-read: 3/3 cells, geomean 0.945, 9/9 accepted,
`D14` identical. No bench run was made for this plan.

Packaged 2.9.0: `.cargo_vcs_info.json` says
`af3faf0dd80f48cc7fa0cb737f4944bca8f6bf85`; `src/` and `CHANGELOG.md` are
byte-identical to the tag checkout; the package differs only in the normalized
`Cargo.toml`, `.cargo-ok`, `Cargo.lock` and the omitted `tests/gen_moves.py`.
`cargo tree -p atomic-movegen` is one node. Inputs: 8 veto rows, 31
`never_prove.tsv` rows, 21 corpus FENs, plus the standard start and 29
generated FEN mutations.

## Phase A — pin

`Cargo.toml` has `atomic-movegen = "=2.9.0"`, default features only; no
`unsafe-optimizations`, path dependency, patch or vendoring. `Cargo.lock` pins
2.9.0. `PROVENANCE.md` records `af3faf0` (`v2.9.0`, MIT) as "differential
harness and tests only, not yet used by the prover". `make licence` is green.
`cargo tree -e normal` adds only the one node.

## Phase B — harness

`tests/differential.rs` has two entry points: a fast smoke set (fixtures,
corpus, mutations and 4 short play-outs) and the ignored full sweep. Comparison
categories:

1. **FEN acceptance.** shakmaty/`position` versus
   `Board::from_fen_with(_, FenPolicy::Lichess)`: accept/reject, canonical
   legal-EP FEN, stored EP square.
2. **Legal moves.** UCI sets from shakmaty `legal_moves`, our
   `pseudo_legal` filtered by `Mover::legal`, and `generate_legal`; duplicate
   checks both ways; pseudo-legal membership checked against the set.
3. **Keys.** Own child key, own parent-to-child delta, legal-EP gating and each
   engine's EP component. Numeric cross-equality is recorded, not asserted (see
   finding 2).
4. **Effects.** Capture flag, 50-move counter, occupancy, blast casualties,
   full child placement (covering promotion and castling rook moves).
5. **Explosion.** Per child `moves::exploding_capture` versus
   `exploding_capture_after`; `Mover::exposes` versus the same child query;
   node-level immediate explosion; witnesses checked for legality and blast.
   Decided children skip the query because movegen documents `None` for them.
6. **Terminality.** Extinction, no-moves-in-check, stalemate and fifty-move,
   with WDL side; only material (`cannot_win` / `MaterialDraw`) is exempt and
   counted, with a separate test for K+B/K+N/K+R vs K.
7. **Play-outs.** Deterministic LCG, 3 seeds x 128 games x <= 200 plies,
   uniform choice from the sorted legal set, every node compared, stop on
   repetition or terminal.
8. **Fail closed.** Every mismatch prints FEN, move, ours and theirs. The full
   test asserts `mismatches == 0`; the counts are aggregated so one run shows
   all occurrences.

Deviations from the plan text, recorded as scope changes:

- item 3's numeric key equality is impossible (finding 2); replaced by semantic
  equality plus a recorded count and an explicit divergence test;
- explosion on a decided child is skipped because it is moot and the API
  contracts differ there, not because a disagreement was filtered;
- `Mover::exposes` and the node-level query are also compared; the plan only
  named `exploding_capture`.

## Findings

**1. En-passant immediate explosion (prover bug).** Minimal FEN
`8/6kp/8/6P1/8/8/8/K7 b - - 0 1`, move `h7h5`. After the double push,
`g5h6` is a legal en-passant capture; its blast on h6 removes the black
commoner on g7. `moves::exploding_capture(&child, White)` returns `None` and
`Mover::exposes(h7h5, ..)` is `false`; `exploding_capture_after` returns
`g5h6`. The prover's helpers test captures of pieces beside the commoner and
never consider that an en-passant destination can be beside it. Full sweep:
86 disagreements (81 per-move + 5 node-level, all en passant, 0 other) and 81
`exposes` misses. This is a rules gap in the prover, not a movegen
disagreement, and plan 1 forbids a prover change.

**2. Independent Zobrist tables.** Standard start: ours
`0x463b96181691fc9c`, movegen `0x5ab84d1cfc77f79e`. Both gate the EP file by
legal capture and both compute their own child key consistently; the numeric
values differ on all 26,869 paired positions. Shakmaty's table is the parity
prover's identity; `atomic-movegen` generated its own splitmix64 table with an
FSF-compatible schema, not FSF/Stockfish values. An adapter must either keep
our hashing, translate keys, or upstream a selectable compatible table (new
wishlist item W5).

No other category diverged: legal moves, effects, placement, terminality and
`MaterialDraw` handling agree everywhere else.

## Measurements

`measurements/plan1/` has `env.json`, `results.json` and `README.md`; no raw
transcripts. Two full runs had identical counts and identical mismatch lines,
1.48 s and 1.55 s.

| Count | Value |
| --- | ---: |
| Positions / legal moves / child keys | 26,869 / 538,453 / 538,453 |
| Games / plies | 384 / 26,406 |
| Explosions present / decided children skipped | 71,971 / 6,644 |
| Material exemptions | 640 |
| Key-table numeric differences | 26,869 |
| Explosion mismatches (all EP) | 86 |
| `exposes` mismatches (all EP) | 81 |
| Other rules mismatches | 0 |

Smoke: 168 positions, 3,467 moves, 0 mismatches, 0.01 s.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --check`, clippy, `cargo test`, `cargo doc` | green |
| `make test` / licence / `make veto` | green |
| 2.9.0 pin, zero dependencies, MIT | green |
| `make test-full` | **red** at the full sweep; bench cells pass, veto green separately |
| Zero unexplained disagreements | **not met**: 86 explained EP disagreements |
| Item 3 numeric key equality | **not met**: independent tables |
| Determinism, two runs | green |
| `check.rs` unchanged, no prover source change | green |
| No adapter, proof-format, table, proof or search change | green |
| No bench row | green; the binary's rules path is unchanged |

## Open parts and missing tests

- The EP-explosion divergence needs a prover fix and a plan with its own
  search/table/proof gates and a same-session bench row; plan 1 does not touch
  prover source.
- The adapter's key strategy is undecided: keep shakmaty hashing (smallest
  change, keeps the parser dependency), translate keys, or wait for an upstream
  compatible table.
- Explosion comparison is existence-only, per the plan; witness identity is
  not compared.
- `make test-full` stays red until the EP finding is fixed; fast `make test`
  and the smoke differential remain green.

## Next steps

1. **Fix the EP explosion gap first.** Owner-triggered plan for
   `moves::exploding_capture` and `Mover::exposes`, using the minimal FEN above
   as the first test; then rerun this harness, the checker/veto gates and a
   10 s bench row. This is the smallest next experiment and unblocks item M5.
2. **Decide key parity.** Add W5 upstream (selectable shakmaty-compatible table
   or a key helper), or keep our key path in the adapter and revise plan 1's
   numeric key gate to the semantic gate this harness now uses.
3. **Only then start plan 2 (adapter).**

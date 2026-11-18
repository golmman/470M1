# Plan 1 — `atomic-movegen` 2.9.0 pin and rules differential

**Item:** 1 (`M5` harness, `M7` pin). **Status:** drafted
2026-11-18. **Session type:** execution. **Predecessor:**
[`initiative.md`](initiative.md), [`rules-surface.md`](rules-surface.md),
[`wishlist1.md`](wishlist1.md). **Successor:** plan 2, the rules adapter.

## Goal

Add released `atomic-movegen` 2.9.0 as a pinned dependency and build the
differential harness that proves its rules layer equivalent to the prover's on
the fixtures, the bench corpus and seeded play-outs. The harness gates every
later adapter step.

**Not in this plan:** no prover source change, no adapter, no parser
replacement, no movegen in `format.rs`/`check.rs`, no search/table/proof/
checker change, no `unsafe-optimizations`, no bench claim, no ledger row.

## Context

- Upstream `af3faf0dd80f48cc7fa0cb737f4944bca8f6bf85` is `v2.9.0`
  (2026-11-18). The changelog lands **W1**
  (`EffectContext::post_move_and_key_with(_, _, EnPassantMode)`, plus
  `PostMove::rule50/terminal/ep_square/casualties/is_capture`), **W2**
  (`Board::from_fen_with(_, FenPolicy::Lichess)`,
  `ValidationErrors::IMPOSSIBLE_CHECK`) and **W3**
  (`EffectContext::exploding_capture_after`). **W4** (analysis view) did not
  land: its report calls the measurement gate unevaluable, so it stays a
  watchlist item.
- The release is additive: no key table, `ZOBRIST_VERSION`, move order, perft,
  FEN output or `Outcome`/`Terminal` change. Zero dependencies,
  `default = []`, `forbid(unsafe_code)` unless `unsafe-optimizations`,
  `rust-version = 1.85` (ours 1.99.1), MIT, so the licence gate accepts it.
- W2's disposition data was frozen against shakmaty 0.30.2; we pin `=0.30.1`.
  A read-only probe of `shakmaty-0.30.1..0.30.2` shows no change to
  `ignore_*`, `PositionErrorKinds` or the validation predicates, so it
  applies. The harness still compares against our 0.30.1.
- The prover still derives effects in `board.rs::effect` and
  `moves.rs::Mover`. W1 removes the double-push clone from the key path; this
  plan only checks equality.
- Expected divergence: no insufficient-material table upstream (K+B/K+N/K+R
  vs K can win) versus shakmaty's broader rule in `moves.rs::cannot_win`. The
  harness exempts exactly that class and counts the hits.
- Move order and explosion witnesses differ by design: compare moves as UCI
  **sets**, `exploding_capture_after` as existence, and require a witness to
  be legal in the child and to explode.

## Options considered

1. **Chosen:** test-only harness, pin only, prover untouched.
2. Runtime `--rules atomic` diff during real search: finds divergence on bench
   trees but adds two rules paths a flag can leak into proofs. Decisive
   experiment: harness green on the bench FENs, then the flag adds nothing.
   Rejected.
3. Keep shakmaty's parser (the wishlist formulation): smaller first adapter,
   incomplete replacement. Decisive experiment: harness green with shakmaty
   parsing; then W2 is a cleanup. Kept alive.

## Phases

### Phase 0 — audit and baseline (read-only)

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`, `make test`, `make test-full`, `make veto`, `cargo doc`.
2. Re-read the `P5` row (3/3, geomean 0.945, 9/9, `D14` identical). No bench
   run here.
3. Confirm the packaged `atomic-movegen-2.9.0` matches the tag and that
   `cargo tree` stays zero-dependency.
4. Inventory the comparison surface in `board.rs`, `moves.rs`, `lib.rs`; count
   the inputs (8 veto rows, 21 corpus FENs).

**Stop rule:** no manifest or source change before the audit is recorded.

### Phase A — pin 2.9.0

1. `Cargo.toml`: `atomic-movegen = "=2.9.0"`, default features only; never
   `unsafe-optimizations`.
2. `PROVENANCE.md`: replace the planned 2.8.0 source pin with `af3faf0…`
   (`v2.9.0`, MIT), marked "planned; not yet used by the prover". No copied
   row.
3. Run the licence gate; update the initiative and wishlist status lines
   (W1–W3 landed in the pin, W4 open, harness in progress).

**Stop rule:** no path dependency, no feature that widens the rules surface,
no licence-gate edit.

### Phase B — `tests/differential.rs`

Two entry points: a fast smoke set for `cargo test`, and the full sweep as
`#[ignore = "slow: rules differential over fixtures, corpus and play-outs"]`.

1. **FEN acceptance.** `atomic470m1::position` vs
   `Board::from_fen_with(_, Lichess)`: accept/reject and, on accept, placement,
   side, rights, ep, clocks. Inputs: fixtures, corpus, generated mutations
   (each castling right with/without its pieces, consistent/inconsistent ep,
   excess material, back-rank pawn, 0/1/2 commoners, wrong-side check,
   opposite check, malformed syntax).
2. **Legal moves.** UCI sets from shakmaty `legal_moves`,
   `Position::legal` over `pseudo_legal`, and `atomic_movegen` iteration;
   exactly once each way.
3. **Keys.** `lib::hash` (Legal) and `Position::hash` equal
   `Board::hash_with(Legal)`; `ep_hash` matches the ep component; per legal
   move `Mover::full_effect(...).hash` equals `post_move_and_key_with(m,
   Legal)`.
4. **Effects.** Capture flag, 50-move counter (saturating), occupancy, blast
   casualties, promotion and castling rook move against `PostMove`,
   `casualties` and `rule50`.
5. **Explosion.** `moves::exploding_capture` presence vs
   `exploding_capture_after`; witness legality and explosion checked, identity
   not compared.
6. **Terminality.** Shared reasons (extinction, no-moves-in-check, stalemate,
   fifty-move) and WDL side. Exempt only `cannot_win`/`MaterialDraw`; assert
   the prover's rule separately on the K+B/K+N/K+R vs K cases.
7. **Play-outs.** 3 seeds × 128 games × ≤ 200 plies, deterministic LCG in the
   test (no `rand`), uniform choice from the sorted legal set, compare every
   node, log node/move counts, stop at terminal or repetition.
8. **Fail closed.** Print FEN, move, expected and actual; no tolerance list or
   "known difference" filter beyond item 6.

### Phase C — wiring and measurement

1. Fast smoke: fixtures plus corpus, a few seconds.
2. Full sweep runs under `make test-full` (`cargo test -- --ignored`).
3. Record `measurements/plan1/` (`README.md`, `env.json`, `results.json` with
   positions, moves, child keys, nodes and per-category mismatches). No raw
   transcripts.
4. No bench row: the binary's rules path is unchanged. When a later phase
   touches it, a same-session 10 s row is required.

### Phase D — docs and closure

Write `report1.md`; update the initiative status/history, the
`rules-surface.md` candidate paragraph and `AGENTS.md`. Do not start the
adapter.

## Gates

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `cargo doc --no-deps`, `make test`, `make test-full`,
  `make veto` green.
- Licence gate green with 2.9.0 (MIT, zero dependencies, default features).
- Zero unexplained disagreements in every category; only the material class is
  exempt and counted.
- Determinism: two runs, same seeds, same counts, zero mismatches.
- `check.rs` imports no prover module; no prover source file changes.
- No move-order, proof-format, table, proof or search change.

## Deliverables

- `Cargo.toml`/`Cargo.lock` with `=2.9.0`; updated provenance pin and status
  lines.
- `tests/differential.rs` with the fast and ignored sweeps.
- `measurements/plan1/` evidence and `report1.md`.
- No adapter, no bench row.

## Risks and stop rules

- Divergence outside the material exemption: stop, record the minimal FEN and
  move; never add a filter to make the gate green.
- Lichess policy mismatches shakmaty 0.30.1: keep shakmaty's parser, record
  the blocker for parser removal, continue the rest.
- Witness differs but the answer agrees: expected; do not force identity.
- Full sweep > 60 s: shrink play-outs first, never fixtures or corpus; record
  the reduced count.
- API surprise: no vendoring, fork, path dependency or patch section; report a
  new wishlist item.
- If docs outgrow the budget, cut docs, not tests.

## Estimated cost

One session: pin/provenance < 30 min, harness 4–6 h, report 1 h. The decisive
experiment is the fixture/corpus smoke set.

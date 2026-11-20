# Report 12 — `M8`: shakmaty removed from the prover

**Item:** `M8` (closed). **Plan:** [`plan12.md`](plan12.md).
**Measurements:** [`measurements/plan12/`](measurements/plan12/README.md).
**Decisions:** `MD1`, `MD5`, `MD6`, `MD9` unchanged; no new decision row.
**Budget:** one sitting. **Tree:** `470MI@0d8ac90` plus this plan's working tree.

## Outcome

`MD1`'s done definition is met: the **prover** no longer uses shakmaty —
parsing, FEN writing, value types, attack tables, move/proof walking and line
display are all `atomic-movegen` or this crate. `grep -rn shakmaty src/` names
only `src/check.rs` and one comment saying the prover does not use it. The
checker is untouched and still the independent veto.

Correctness is unchanged: the full release differential reports the same
84/11 FENs, **27,088** positions and **540,383** moves/keys with 0/0 and EP
87/0/0; all three cell proofs are **byte-identical** to plans 10/11
(`core` `714723211c7a0246…`, `easy-win` `32ffb30d96ccb172…`, `deep-win`
`50f062de2a67b1dd…`), with identical counters and mate, accepted 9/9 by
`470MI check` and `provari check`.

Instructions are parity, not a regression: after two prover-side fixes the
M8 binary reads **7,061,619,735** on `core` against plan 11's
**7,057,344,364** (**+4,275,371**, 0.061%), and **falls** 3,999,606 on
`easy-win` and 124,456,407 on `deep-win`. The same-session `MD6` row is
3/3, 9/9 and `D14` identical, geomean **1.001** against the plan 2 binary's
**0.988**: candidate/old **1.013** (plan 11: 1.012), so `MD6` is still a
recorded **miss**. `M8` closes independent of that performance bar.

## Phase A — inventory and native vocabulary

The remaining shakmaty surface was exactly the typed boundary: `lib.rs`
(`Atomic`, `Setup`, `Fen`, `ending`/`status`), `board.rs`
(`from_atomic`/`to_atomic`/`to_shakmaty` and the value types), `format.rs`
(`Strategy` on `shakmaty::Move`, `board_move`, `Fen`, `play_unchecked`),
`lines.rs` (`SanPlus`, `Fen`, `UciMove`), `walk.rs`/`search.rs`/`dfpn.rs`
(`Atomic`, move lists, proof read-back) and the value/attack vocabulary in
`moves.rs`/`eval.rs`. The search itself already ran on the crate-backed
`Position`; only its public boundary was shakmaty.

`src/types.rs` is the single native vocabulary: crate re-exports
(`Bitboard`, `Color`, `Piece`, `PieceType as Role`, `Square`, `attacks`), a
compile-time line/ray table, a direct bit-scan `Squares` iterator and the
set/geometry helpers the hot path uses. It contains no rule; legality and
move generation stay the crate's.

## Phase B — the removal

- `lib.rs::position` is `Board::from_fen_with(fen, FenPolicy::Lichess)`. The
  old `ignore_invalid_castling_rights`/`ignore_invalid_ep_square`/
  `ignore_too_much_material` policy and the structural rejections are the
  crate's `Lichess` policy; error text still names the classification.
- `board.rs` drops `from_atomic`, `to_atomic` and `to_shakmaty`; adds
  `from_fen`, `to_fen`, `fullmoves`, `legal_moves`, `by_color_role`.
- `format.rs` walks `Position` and our `Move` directly. JSON lines parses
  `Board::move_from_uci`; there is no `board_move` translator.
- `walk.rs`, `lines.rs`, `main.rs`, `search.rs` use `Position`/`Move`; SAN is
  `Board::san`, the PGN move number is `Position::fullmoves`, play is
  `Position::play`.
- `dfpn` takes `&Position` and returns our `Move`; the dropped-mate replay no
  longer converts to shakmaty.
- `moves.rs`/`eval.rs` use the crate's attack tables and `PieceType as Role`;
  `cannot_win` keeps the prover's broad material rule (written twice, as
  before) and now says so.

`src/check.rs` has no diff. `tests/differential.rs` keeps shakmaty as the
independent oracle and now rebuilds its side from FEN with a local
`shak_position` helper, so the prover exports no conversion method for tests.
That is the deliberate remaining shakmaty use: the independent implementation.

## Phase C — performance finding and fix

The first removal was **+176,245,439** instructions on `core` (+2.50%),
−43,198,145 on `easy-win` and −2,090,611,526 on `deep-win`; not acceptable as
a silent stage. Profiling (`perf record -c 500000`) put the excess in the
attack dispatch and value loops. Two changes, both in this repository:

1. `types::attacks_from` is `#[inline(always)]`; the crate's `#[inline]`
   helper stayed a call in the `Mover`/evaluation loops. This removed almost
   all of the gap.
2. `Squares::next` pops the least-significant square directly instead of
   `first()` + `without()` (two shifts per iteration), and `aligned` uses a
   precomputed full-line table rather than two directional table loads.

The result is the parity row in the README. No crate, pin, feature or
`PROVENANCE.md` change; `unsafe-optimizations` stays on 2.11.0.

## Gates and witnesses

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `cargo doc --no-deps`: green.
- `make test`, `make test-full`, `make veto`: green. `test-full` includes the
  pinned crate's feature suite, every ignored test, the full differential and
  the nine proof fixtures plus the control.
- Release full differential: `mismatches: 0`, `key_mismatches: 0`, EP 87/0/0,
  27,088 positions, 540,383 moves/keys, 392 games / 26,612 play-out nodes.
- Cell proofs: `.pvr` sha256 and counters identical to plans 10/11; 3/3
  cells, 9/9 checker verdicts, `D14` timed/untimed counters identical.

## Problems, scope changes and open parts

- **The first removal regressed 2.5%.** It was fixed on the prover side, not
  by keeping a shakmaty attack path; the final row is 0.061% on `core` and a
  fall on the other two cells. The core residual is about 12 instructions per
  node and below the `MD6` class.
- **Two unit witnesses were rewritten, not weakened.** `board.rs`'s random
  shakmaty game comparison and `format.rs`'s `orders_and_judges_as_shakmaty`
  now test the native implementation for self-consistency; the shakmaty
  comparison lives in `tests/differential.rs`, which is the stronger and
  larger witness.
- **Error text changed.** `lib.rs` now reports the crate's classified flaws
  (`"Black has no king"`, `"the side that just moved is in check"`) instead of
  the old sentence template. The classifications and accept/reject decisions
  are unchanged; the differential checks acceptance, not wording.
- **Test-only shakmaty remains.** `tests/differential.rs`, `src/check.rs` and
  its fixtures use shakmaty on purpose. This is the independent oracle, not a
  prover dependency.
- **No host-label test.** The `MD6` row again rests on the platform string,
  binary hashes and the plan 11 product's `core` reading; the container's UTS
  name is not the owner's label. Recorded in `bench/env.json`.
- **`MD6` remains missed.** The removal is correctness work; it neither
  closes nor widens the ~1.01× class. The row is 1.013 candidate/old against
  plan 11's 1.012.

## Distance to the goal

`MD1`'s prover-replacement goal is **met**: the prover is clean and the checker
is independent. The performance claim remains a dated `MD6` miss (`core` is
**7.0616e9**, **1.075×** plan 2's 6.5694e9; plan 11 was 1.074×). The remaining
gap is the rules/non-rules residual plans 6–9 already priced, not a shakmaty
remnant. `check.rs` is unchanged and still the external contract.

## Next steps and options

1. **Kill review (recommended).** The initiative has no kill review after
   plans 4, 6, 8 or 10; `M8` closes the last item. Run the fixed prompt before
   any further performance plan: "What is the absolute distance to the goal
   now? If we started today, would we build this? Which initiative or plan
   would you cancel? Re-run the prior-art search, delta only. What is the
   smallest next experiment?"
2. **Record the done state and stop.** Update the initiative to closed on
   `MD1`'s terms while keeping the `MD6` miss dated; no further `movegen` plan
   unless a measured lever appears outside the residual.
3. **Push the upstream CI workflow** from
   `measurements/plan11/upstream-ci.patch` at `0aaf958`; no pin change.

Option 1's kickoff prompt:

> Run the fixed kill-review prompt over `movegen`: read
> `docs/plans/movegen/initiative.md`, `report12.md`, `report11.md`,
> `decisions.md` (`MD1`, `MD6`, `MD7`, `MD8`, `MD9`) and
> `measurements/plan12/`. Do not implement; answer the five questions with the
> measured distance and the smallest next experiment.

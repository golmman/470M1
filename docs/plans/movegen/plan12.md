# Plan 12 — `M8`, complete shakmaty removal from the prover

**Item:** `M8` (complete prover replacement). **Session:** execution.
**Predecessors:** [`initiative.md`](initiative.md), [`report11.md`](report11.md),
[`decisions.md`](decisions.md) (`MD1`, `MD5`, `MD6`, `MD9`),
[`measurements/plan11/`](measurements/plan11/README.md). **Date:** 2026-11-20.

## Goal

Finish `MD1`: the prover parses FEN, writes it, walks proofs and displays lines
without shakmaty. `src/check.rs` stays shakmaty-only and independent. The
differential stays green, every gate and veto stays green, and the same-session
`MD6` row is recorded. The adopted `atomic-movegen` 2.11.0 pin and its audited
`unsafe-optimizations` feature are **not changed**.

## Context

Plan 11 closed `M15`; `M8` is the only open item and the initiative's done
definition names it. The remaining shakmaty surface is not the search: `dfpn`
already works on the crate-backed `Position`; it is the typed boundary
(`Atomic`, `Fen`, `Move`), the proof walk in `format.rs`, `lines.rs`, `walk.rs`,
and the value/attack vocabulary still used by `board.rs`, `moves.rs` and
`eval.rs`. The crate already ships the needed parser
(`Board::from_fen_with(FenPolicy::Lichess)`), FEN writer (`fen_with`), UCI
(`move_from_uci`/`to_uci`) and SAN (`san`/`move_from_san`), so parser/display
removal needs no upstream ask (`W2` is released).

The hot-path boundary stays `MD5`: direct crate calls, no translator. Removing
shakmaty's attack tables from `moves.rs`/`eval.rs` changes the geometry source,
so the instruction count is measured against the plan 11 product and must not
regress materially.

## Not in this plan

- No pin or feature change; no crate, `PROVENANCE.md` or licence-gate change.
- No search, tree, proof-format, table or checker semantics change.
- No checker import of any prover module; `check.rs` is untouched.
- No new wishlist item: no missing API was found.

## Phases

### Phase A — inventory and native vocabulary

1. List every shakmaty name outside `check.rs`; classify as parser, display,
   proof path, line output, or value/attack vocabulary.
2. Add one prover vocabulary module (`src/types.rs`) re-exporting the crate's
   `Bitboard`, `Color`, `Piece`, `PieceType as Role`, `Square` and `attacks`,
   plus compile-time line/ray tables and the few set helpers the hot path uses.
3. Record the acceptance rule: after this plan `grep -rn shakmaty src/` names
   only `check.rs` and one "the prover does not use shakmaty" comment; tests
   may keep shakmaty as the independent oracle.

### Phase B — parser, display, proof paths, line output

1. `lib.rs`: `position` uses `Board::from_fen_with(_, Lichess)`; `ending`,
   `hash` and `status` take the crate-backed `Position`; error text preserves
   the old Lichess classifications.
2. `board.rs`: drop `from_atomic`/`to_atomic`/`to_shakmaty`; add `from_fen`,
   `to_fen`, `fullmoves`, `legal_moves`; value types are the crate's.
3. `format.rs`: root and strategy are `Position`/our `Move`; JSON-lines uses
   `Move::from_uci`; no `board_move` translation remains.
4. `walk.rs`, `lines.rs`, `main.rs`, `search.rs`: our `Position`/`Move`;
   SAN from `Board::san`, FEN from `Position::to_fen`.
5. `moves.rs`, `eval.rs`: crate attack tables and role dispatch.

### Phase C — correctness and performance witnesses

1. Update the unit and integration witnesses to the native vocabulary; keep
   `tests/differential.rs` on shakmaty as the independent oracle, rebuilding
   the shakmaty side from FEN through a local helper.
2. Run the full release differential (expect the plan 10/11 counters:
   84/11 FENs, 27,088 positions, 540,383 moves/keys, EP 87/0/0, 0/0).
3. Re-prove the three cells and compare `.pvr` sha256 and counters with plan
   10/11; both checkers must accept.
4. Three interleaved `perf stat -e instructions:u` rounds against the plan 11
   product binary (`0d8ac90`), same cells and flags.
5. Same-session `MD6` row: candidate and the plan 2 binary (`2f629fd`) back to
   back, 10 s, protocol `a`, seed 0, three repeats, `--d14`.

**Stop rule:** a behavioural or `.pvr` difference stops the adoption and is
reported as a finding, not explained away. A material instruction regression
is fixed on the prover's side or recorded as a dated `MD6` finding.

### Phase D — gates, docs, evidence

`make test`, `make test-full`, `make veto`; `cargo fmt`, `clippy`, `doc`.
Record `measurements/plan12/` (README, driver, `perf.txt`, `env.json`,
`bench/`), update this report, the initiative, `AGENTS.md` and `docs/bench.md`.
No pin, feature or `PROVENANCE.md` change.

## Acceptance

- No `use shakmaty` outside `check.rs`; checker untouched and still vetoes.
- Differential 0/0 with the expected counters; 9/9 accepted by both checkers.
- `.pvr` sha256 and counters identical to plan 10/11 on all three cells.
- Instruction count does not regress materially against plan 11; `MD6` row
  recorded (a miss is still a recorded row, `MD6` is the bar for a pass).

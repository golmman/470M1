# Plan 1 measurements — rules differential

**Date:** 2026-11-18. **Host:** `f043976fff92` (x86_64, Linux 7.2.13).
**Harness:** `tests/differential.rs`, pin `atomic-movegen` `2.9.0`
(`af3faf0`), shakmaty `0.30.1`. **Outcome:** blocked by two findings; no
bench row (the binary's rules path is unchanged).

`env.json` is the pin record; `results.json` has the parsed counts from two
identical full-sweep runs and the minimal findings. The counts are the
`DIFFERENTIAL` line and the mismatch lines from the test output; no raw
transcripts or oracle artifacts are committed.

## Commands

```
cargo test --test differential fen_acceptance_and_rules_smoke -- --nocapture
cargo test --test differential rules_differential_full -- --ignored --nocapture
```

The smoke run is green. The full run fails closed as designed: it prints every
mismatch with FEN, move and both answers, then asserts `mismatches == 0`.

## Full sweep

3 seeds x 128 games x <= 200 plies, plus the fixture/corpus FENs and
generated acceptance mutations. Two runs gave identical counts and identical
mismatch lines; elapsed 1.48 s and 1.55 s.

| Category | Count |
| --- | ---: |
| FEN accepted / rejected | 79 / 11 |
| Positions compared | 26,869 |
| Legal moves compared | 538,453 |
| Child keys compared | 538,453 |
| Play-out games / plies | 384 / 26,406 |
| Immediate explosions present | 71,971 |
| Decided children skipped (explosion is moot) | 6,644 |
| Material exemptions (shakmaty broad rule vs `MaterialDraw`) | 640 |
| Numeric key-table differences | 26,869 (every paired position) |
| Immediate-explosion mismatches | 86, all en passant |
| `Mover::exposes` mismatches | 81, all en passant |
| Other rules mismatches | 0 |

The 86 immediate-explosion mismatches are 81 per-move child queries plus 5
node-level queries. `Mover::exposes` misses 81 of the 81 per-move cases; the
other 5 are only visible at the node query. Every one has an en-passant
witness.

## Findings

1. **En-passant immediate explosion is missed by the prover.** Minimal FEN
   `8/6kp/8/6P1/8/8/8/K7 b - - 0 1`, move `h7h5`. After the double push,
   White's `g5h6` en passant explodes the black commoner on g7. The blast
   square is h6; the captured pawn on h5 is not beside the commoner, so
   `moves::exploding_capture` and `Mover::exposes` both return false while
   `EffectContext::exploding_capture_after` returns the witness. This is a
   prover-side rules gap, not a movegen disagreement; plan 1 does not change
   prover source.
2. **The Zobrist tables are independent.** Standard start: ours
   `0x463b96181691fc9c`, movegen `0x5ab84d1cfc77f79e`. The legal-en-passant
   gating and each engine's own child-key contract agree on all 26,869
   positions, but the numeric values cannot match without an upstream
   compatible table or an adapter-side re-keying decision. The committed
   harness records the numeric difference and asserts the semantics.

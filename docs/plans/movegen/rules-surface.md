# `movegen` — rules surface to preserve

Read from `parity` at closure (plan 5). The prover may replace these modules
with `atomic-movegen`; `check.rs` stays on `shakmaty`. A difference is a
finding, not a convenience.

| Contract | Current call sites | Invariant `movegen` must preserve | Side |
| --- | --- | --- | --- |
| FEN acceptance | `lib.rs::position`, `from_setup`, `flaws`; `board.rs::from_fen` (test only) | Lichess-style: drop bad castling rights, an impossible en passant square, excess material; reject empty/missing king/too many kings/pawns on backrank/opposite or impossible check. | prover |
| Identity and hash | `lib.rs::hash`; `board.rs::from_atomic`, `full_hash`, `legal_ep_hash`, `ep_hash`, `hash`, `child_base`, `effect` | Key = board, side, castling rights, and en passant only when an en passant capture is legal. Clocks are excluded. Hash equals shakmaty's `zobrist_hash(EnPassantMode::Legal)`. | prover |
| Move encoding | `board.rs::Move` (`bits`, `from_bits`, `from`, `to`, `promotion`, `is_en_passant`, `is_castle`), `to_shakmaty` | Stable 16-bit move: from, to, kind; castling king-to-rook; en passant and four promotions distinct. Proof order depends on it. | prover |
| Move generation | `board.rs::pseudo_legal`, `pawn_moves`, `castling_move`, `legal`, `is_pseudo_legal`; `moves.rs::Mover`, `legal` | Every legal move exactly once; standard castling geometry; promotions N/B/R/Q only; en passant only when legal; no king capture beyond the game-ending explosion. Order may change only with the proof/checker move order. | prover |
| Effects | `board.rs::Effect`, `effect`, `play`, `apply`, `blast`, `occupied_after`; `moves.rs::effect`, `full_effect` | Child hash, 50-move counter, occupancy, capture flag, blast casualties, promotion, castling rook move. A blast removes non-pawns in the 3×3 zone; the capturing pawn dies at ground zero; pawns otherwise survive. | prover |
| Check and legality | `moves.rs::Mover` (`tested`, `touching`, `check_squares`, `uncovering`, `gives_check`, `exposes`, `connectable`), `board.rs::checked_after`, `in_check` | Touching kings are never in check. A side without a king is not in check. Pins apply only when the king is present and the kings do not touch. An en-passant destination beside the commoner is an immediate explosion; a double push leaving such a square exposes the king. A move that explodes the enemy king ends the game even if it exposes the mover's king (the blast takes both). | prover |
| Terminality | `moves.rs::has_legal_move`, `replies`, `cannot_win`; `format.rs::judge`; `dfpn.rs` repetition and 50-move handling; `lib.rs::ending`, `status` | Exploded king loses; no legal moves is mate if in check else stalemate; **the checker's insufficient-material rule is authoritative**; 50-move and repetition are draws, never wins. | prover |
| Insufficient material | `moves.rs::cannot_win` (prover) and `shakmaty::Position::is_insufficient_material` (checker) | Keep shakmaty's broader atomic rule in the prover. `atomic-movegen` deliberately has no such table (K+B/K+N/K+R vs K can win there), so **do not adopt its material verdict**. | both, written twice |
| Proof order | `format.rs::number`, `is_capture`, `won_at_once`, `Walk`; checker `check.rs` walks defences in square order | Compact proof writes only attacker moves; checks first, then from-square, to-square, promotion. The checker generates every defence. A rules rewrite must not change this order. | prover writes; checker reads |
| Proof checking | `check.rs` only | shakmaty only; no prover module import (test at `check.rs` bottom). Tablebase verdicts remain unsupported. | checker |

`main.rs`, `lines.rs`, `walk.rs` and `format.rs` may keep shakmaty for
FEN/UCI/SAN display even if the search uses `atomic-movegen`.

## Candidate dependency

`golmman/atomic_movegen` `af3faf0dd80f48cc7fa0cb737f4944bca8f6bf85` (v2.9.0,
MIT) covers FEN/`Setup`, 16-bit moves, legal generation, `EffectContext`,
legal-en-passant hashing, outcomes, terminality, checkers and pins; W1-W3
landed in this pin. Plan 1's harness is in
[`../../../tests/differential.rs`](../../../tests/differential.rs); plan 2
fixed its en-passant finding, so `moves.rs::exploding_capture` and
`Mover::exposes` now see an EP destination beside the commoner (the captured
pawn on the other square is not beside it) and the full sweep is green. The two
crates' numeric Zobrist tables still differ although the schema and legal-EP
gating agree; plan 3 must decide keep-our-hash, translate, or wait for W5. The
prover-side material rule remains explicit, not a movegen result. Open upstream
gaps are tracked in [`wishlist1.md`](wishlist1.md) (W4, W5).

# Plan 2 — en-passant immediate explosions

**Item:** none in the table; [`report1.md`](report1.md) step 1 — clears `M5`,
unblocks `M1`. **Session:** execution; predecessor report 1; successor plan 3,
the rules adapter.

## Goal

Make `moves::exploding_capture(pos, side)` see an EP capture whose
*destination* is beside the victim king, and `Mover::exposes(m, effect)` return
true for a double push that leaves such a square; then re-run the differential
harness, the gates and one same-session 10 s bench row.

**Not in this plan:** no adapter, pin, hash/Zobrist-table, `check.rs`,
proof-format, move-order, search or eval change; no `threats` change.

## Diagnosis

Probe (read-only):
`cargo test --test differential rules_differential_full -- --ignored -- --nocapture`
→ 26,869 positions / 538,453 moves, **86** explosion mismatches (81 per move + 5
node level, all EP), **81** `exposes` misses (all EP), 0 elsewhere, 26,869 key
differences; two runs identical. Smoke is green: no fixture has a passed square
beside a king.

Witness: `8/6kp/8/6P1/8/8/8/K7 b - - 0 1`, `h7h5`. The child's only explosion
is `g5h6`: the passed `h6` is empty, the captured pawn is on `h5`, the blast is
at `h6`, beside `g7`. Both helpers only look for a *piece* beside the king
(`king_attacks(king) & by_color(!side)`; `exposed`/`beside_us`), never an empty
destination. A missed `exposes` at `dfpn.rs:811/820` and `:1136` (`mates`) can
claim a win the checker rejects: a soundness gap, latent only while the veto
stays green. Witness order is proof contract, so the EP branch runs **after**
the loop.

## Options considered

1. **Chosen — targeted EP branch in both helpers:** one predicate, two additive
   `Position` accessors.
2. **Change the formulation:** ask the child, `exploding_capture(&pos.play(m),
   !us)`, per legal move. Exact, but a key recompute on the hot path; est.
   10–30% wall loss. Cheapest decisive experiment: plan5's driver with the child
   query, compare evals and wall. Fallback only.
3. Adopt `exploding_capture_after` now: the adapter before its gates; rejected.

## Phases

### 0 — baseline (~20 min)

All baseline gates; record the counters above and `P5`, and keep
`target/release/470m1-prefix` (never committed).
**Stop rule:** no source edit before the baseline is recorded.

### A — patch (~2 h)

`src/board.rs`, additive: `pub fn maybe_ep_square(&self) -> Option<Square>`
(`self.ep`, field still private), and `pub fn en_passant_move(&self, from, to)
-> Move` (`Move::new(from, to, EN_PASSANT)`). `effect`, `apply`, `legal_ep_hash`,
the Zobrist table and `hash` untouched.

`src/moves.rs`:

- `fn ep_explodes(to, victim: Option<Square>, taker: Option<Square>) -> bool` =
  `king_attacks(victim).contains(to) && !king_attacks(taker).contains(to)`:
  beside the victim, away from the capturer, which the blast takes too.
- `exploding_capture`: target loop verbatim, then `or_else` EP — require
  `pos.turn() == side`, `pos.maybe_ep_square()`, a pawn taker
  (`attackers_to(to, occupied) & by_piece(side.pawn())`) and
  `ep_explodes(to, king, pos.king(side))`; return `en_passant_move(from, to)` if
  `pos.legal(m)` (exact even for odd FENs).
- `Mover::exposes`: in the quiet branch turn the three existing answers into
  `covered`, return `covered || (effect.en_passant && self.ep_exposes(m))`;
  `ep_exposes` uses `(from + to) / 2` as the passed square, the victim from
  `our_king_after(m)` filtered by `effect.occupied`, the taker `their_king`.
  `effect.en_passant` already means "an enemy pawn attacks the passed square";
  that pawn always has the EP capture, legal exactly when the blast takes our
  king and not theirs — the same predicate, so both helpers agree.
- `threats`, `eval`, `gives_check`, `plain` and the rest unchanged: `threats` is
  score-only and stays EP-blind (record the undercount; its moves.rs reference
  mirrors it, so that test stays honest).

### B — tests (~2 h)

1. `moves.rs` `en_passant_explosion`: minimal FEN, find `h7h5` in
   `pseudo_legal()`; assert `child.maybe_ep_square() == Some(h6)`,
   `exposes(h7h5, &effect)`, `exploding_capture(&child, White) == Some(g5h6)`;
   mirror `k7/8/8/8/6p1/6K1/7P/8 w - - 0 1`, `h2h4`, `g4h3`.
2. Negatives: no taker `8/6kp/8/8/8/8/8/K7 b - - 0 1`; passed square beside
   both kings `8/8/8/8/2k1p3/8/3PK3/8 w - - 0 1`, `d2d4` → false (that EP
   explodes the mover's king; `legal` rejects it).
3. Witness order: `8/6kp/5n2/6P1/4N3/8/8/K7 b - - 0 1`, `h7h5` — `f6` is a
   piece target *and* `g5h6` explodes; assert the witness still captures `f6`.
   Add the FENs to `matches_the_played_positions` starts so its per-move
   cross-check covers EP; keep `moves > 200_000`.
4. `tests/differential.rs`: add `fn ep_fens()` (the FENs above;
   `tests/fixtures/*.tsv` untouched), fed to `run_acceptance_set` in the smoke
   and full tests; add `#[test] fn en_passant_explosion_regression()` asserting
   per legal move that `exploding_capture(child, child.turn())` presence and
   `Mover::exposes` equal `EffectContext::exploding_capture_after`, and the same
   witness `to` square where both exist. `run_playouts` gains a `starts`
   parameter so the 3 × 128 game sweep keeps its counts; EP play-outs run
   separately (1 seed, 8 games, ≤ 200 plies).
   **Stop rule:** a non-EP mismatch stops the plan; never widen a filter.

### C — gates (~40 min)

fmt, clippy `--all-targets -- -D warnings`, `cargo test`, `cargo doc --no-deps`,
`make test`, `make test-full` (sweep green **and** veto), `make veto` (9/9
accepted, control rejected by both). The sweep prints `mismatches: 0` and all
three EP counters 0; `key_table_differences` stays 26,869 plus the added
positions, recorded not asserted.

### D — bench (~1 h)

`make oracle` (or reuse `/tmp/470M1-oracle`); copy plan5's driver to
`docs/plans/movegen/measurements/plan2/run_bench.py`, header only. One session,
one process at a time:

```
python3 docs/plans/movegen/measurements/plan2/run_bench.py \
  --provari /tmp/470M1-oracle/provari/target/release/provari \
  --candidate target/release/470m1 \
  --out docs/plans/movegen/measurements/plan2/results.json \
  --seconds 10 --reference-seconds 10 --memory 192 \
  --repeats 3 --seeds 0 --protocols a --d14
```

then the same with the pre-fix candidate (`--out .../prefix.json`) to show
whether the tree moved as well as the wall. Commit driver, `results.json`,
`prefix.json`, `env.json`, `README.md`; no transcripts. Append one ledger
row (`movegen plan2`, `hand, product default (seed 0)`, cap 10 s) plus a
footnote: coverage, accepted-by-both counts, lengths vs same-session mates,
`child_evals`, pre-fix diagnostic. Bar: 3/3, geomean ≤ 1.0, 9/9.

### E — docs and closure (~1 h)

`report2.md` (≤ 12 KB); update the initiative history/status, the
`rules-surface.md` EP paragraph and the `AGENTS.md` state line. Second `movegen`
plan: the owner's kill review follows.

## Gates

| Gate | Bar |
| --- | --- |
| Full sweep | `mismatches: 0`; the three EP counters 0; two runs identical |
| `make test`, `make test-full`, `make veto` | green; veto 9/9 + control |
| Checker, keys, order | `check.rs` untouched; no Zobrist/`hash` change; witness test green; `format.rs` untouched |
| Bench | 3/3, geomean ≤ 1.0, 9/9 accepted, one session |

## Deliverables and risks

Patch, tests, `measurements/plan2/`, one ledger row, `report2.md`, status lines;
no adapter, pin, key-table or checker change.

- **Witness/proof churn:** if any written proof changes its move, stop.
- **`effect.en_passant` means "an enemy pawn attacks the passed square"**, not
  "the EP capture is legal"; the child key comes from `full_effect`.
- **`maybe_ep_square` is only meaningful for the side to move:** guard with
  `pos.turn() == side`.
- **Over-reporting:** an EP beside the capturer's own king is draw/illegal;
  `ep_explodes` excludes it, the both-kings test pins it.
- **Bench regression:** report it, no search tuning; a sweep mismatch after
  phase A stops at the minimal FEN, never behind a filter.

## Estimated cost

One session: baseline 20 min, patch 2 h, tests 2 h, gates 40 min, bench 1 h,
docs 1 h. Decisive experiment: the minimal-FEN unit test plus the sweep.

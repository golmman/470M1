# Plan 3 — rename, then the `M1` rules adapter

**Item:** `M1` (adapter), with `M3` native keys, `M9` rename, `MD3` census.
**Session:** execution. **Predecessor:** [`report2.md`](report2.md) — sweep
green, 10 s row 3/3, geomean **0.972**, 9/9 accepted. **Successor:**
`report3.md`; the owner's kill review follows.

## Goal

1. Rename per `MD4`: package/lib `atomic470mi`, binary `470MI`, first.
2. Wrap `atomic_movegen::Board` behind `Position`: the prover's identity,
   movegen, legality, effects and child keys use `atomic-movegen`'s **native**
   keys (`MD2`). Keep shakmaty for parsing, the typed boundary,
   `to_atomic`/`to_shakmaty`, `Mover`'s geometry and the checker; parser
   removal is `M8`.
3. Record the `MD3` census; change no eval.
4. Gate on the differential, both checkers and a same-session 10 s bench.

Not here: `check.rs` logic (rename strings only), `.pvr` format/move-order,
search/eval/terminality, `Mover` rewrite, W5, new pin, parser removal.

## Options

1. **Chosen — `Position` embeds `Board`, keeps the typed API**, with a cheap
   native-key path and a `StateInfo` in `Mover`. Smallest blast radius; the
   differential is the oracle.
2. **Replace `Position` with `Board` at every call site now.** Less
   glue/duplication (est. 10–15%), but touches `dfpn`/`format`/`lines`/`eval`
   at once. Cheapest experiment: a 1 h `dfpn` spike on `core`; rejected
   (`M8`).
3. **Dual `--rules parity|movegen`.** Isolates a regression but allows two
   prover rules paths, against `MD1`. Cheapest experiment: phase A's spike;
   rejected.

## Phases

### 0 — rename (`MD4`, no adapter)

`atomic470m1`→`atomic470mi`, `470m1`→`470MI` in `Cargo.toml`/`Cargo.lock`,
`src/**`, `tests/**` (including `CARGO_BIN_EXE_470MI` and `Usage: 470MI`),
`scripts/veto.sh`, `Makefile`, `bench/README.md`, and the `docs/bench.md`
command with a new `470MI@<sha>` row. Do not touch `docs/plans/**`,
`docs/notes/**`, historical `470m1@<sha>` rows or `/tmp/470M1-oracle`. Gate:
fmt, clippy, `cargo test`, `make test`; `grep -ri 470m1` is empty outside those.
**Stop:** no adapter code here.

### A — adapter (~3 h)

`src/board.rs`, keeping the public API and provenance header:

- `Position { board: atomic_movegen::Board, hash: u64 }`; `hash` is the Legal
  key, cached for `dfpn`.
- `from_atomic`: `atomic_movegen::Setup` from the `Atomic` (pieces, side,
  castling bits, **stored** `maybe_ep_square`, `rule50`), then
  `Board::from_setup`; `hash = board.hash_with(Legal)`.
- Delegate the square/bitboard/piece accessors and map shakmaty types at the
  edge (`Bitboard(pub u64)`, King↔Commoner); keep `checked_after`,
  `en_passant_move`, `capture_move`, `to_shakmaty`, `board_move`.
- `pseudo_legal`: `movegen::for_each_pseudo_legal` into the `MaybeUninit`
  `Moves`, converted to our 16-bit move.
- `Position::legal` uses `board.is_legal`; `Mover::new` populates one
  `StateInfo`, and the fallback uses `board.legal(m, &state)`. If cheaper,
  derive `tested` from `state.checkers`/`state.pinned`; the differential gates
  it.
- **Child keys:** `effect(m)` (drop `base`) keeps occupancy/capture/counter,
  drops the Zobrist/rights XORs, and takes
  `hash = board.hash_after_with(their, Legal)` — incremental, cloning only on
  a double push (the parity path already played EP). Drop
  `Effect::en_passant`; `full_effect` aliases `effect`; remove `child_base`,
  `ep_hash`, `Mover::base`. `post_move_and_key_with` is *not* the hot path.
- `play`: clone `Board`, `do_move`, cache `hash = board.hash_with(Legal)`.
  `null()`: `Board` via `Setup` (side flipped, ep cleared, **rule50 + 1**).
- `to_atomic`: rebuild the `Setup` from `Board`; remove unused
  `squares`/`blast`/`occupied_after`. Keep `crate::position`, `to_atomic`,
  `to_shakmaty` (shakmaty parsing). Other modules change only for the moved
  signatures and imports.

### B — tests and differential (~2 h)

- `tests/differential.rs`: `atomic470mi`; `compare_node` asserts
  `our.hash() == board.hash_with(Legal)` and counts `key_mismatches` (0);
  retire `numeric_key_parity_is_a_recorded_divergence`, add
  `native_keys_match_movegen` (standard, corpus, `ep_fens`, play-outs); replace
  the `ep_hash` block with `Always == Legal` iff the child has no legal EP.
- `src/board.rs`: keep `matches_shakmaty` moves/placement; drop the
  `Zobrist64`/`swap_turn` asserts.
- `src/moves.rs`: `full_effect` alias; keep the EP regressions, witness order
  and `matches_the_played_positions`.
- `native_keys_are_deterministic`: same key per `(FEN, move)` in two runs.

### C — `MD3` EP-`threats` census (~1 h, read-only)

Per node and legal move, count `ep_child` (the child's side to move has an EP
explosion: `exploding_capture(child, child.turn())` is EP),
`ep_threats_verdicts` (`threats >= 2` there) and `ep_threats_difference` against
`exploding_captures_ep(child, us)` — the test helper extended with the
empty-destination EP target and the EP turn guard. Expected **0**: after our
move the EP right is the opponent's, so it can void a double threat (the
`dfpn.rs:918` `exposed` guard) but never be one of *our* next-move threats. A
verdict-relevant case is a soundness finding, not an eval tweak; no eval or
`threats` change.

### D — gates (~40 min)

fmt, clippy `--all-targets -- -D warnings`, `cargo test`, `cargo doc --no-deps`,
`make test`, `make test-full` (sweep + slow cells + veto), `make veto` (9/9,
control rejected). Sweep: `mismatches: 0`, `key_mismatches: 0`, the three EP
counters 0, two identical runs. `check.rs` differs only in rename strings; its
`uses_nothing_from_the_prover` and CLI tests stay green; all `.pvr` fixtures
pass both checkers.

### E — bench (~1 h)

Copy plan2's driver to `measurements/plan3/run_bench.py` (header only), build
`target/release/470MI`, one session, one process at a time:

```
python3 docs/plans/movegen/measurements/plan3/run_bench.py \
  --provari /tmp/470M1-oracle/provari/target/release/provari \
  --candidate target/release/470MI \
  --out docs/plans/movegen/measurements/plan3/results.json \
  --seconds 10 --reference-seconds 10 --memory 192 \
  --repeats 3 --seeds 0 --protocols a --d14
```

Commit driver, `results.json`, `env.json`, `README.md`; no transcripts. Append
one ledger row (`movegen plan3`, `hand, product default (seed 0)`, 10 s) with
coverage, 9/9, per-cell factors, mates, `child_evals`, `D14`, and the per-cell
delta to **0.972** (cross-session reference, not a factor). Bar: **3/3, geomean
<= 1.0, 9/9**; adopt native keys only if it holds. A factor > 1.0 or lost cell
is a regression finding: report it, no search tuning.

### F — docs and closure (~1 h)

`report3.md` (<= 12 KB); update the initiative status/history, `AGENTS.md`
state, `rules-surface.md`, `PROVENANCE.md` (`atomic_movegen` now used by the
prover) and the `docs/bench.md` pin row. State what remains for `M2`/`M8`.

## Risks and stop rules

- **Effect/key cost.** If the bench regresses, profile `effect`/`Mover::new`;
  fallback is `hash_and_rule50_after` plus a double-push-only Legal correction,
  reported.
- **Native keys move the tree.** Keys index the TT/filter, so
  collisions/counters/mates may differ without a rules change; the same-session
  provari factor is the only valid comparison.
- **`null()` clock.** `null_for` preserves rule50; the adapter must set
  rule50+1 (`dfpn`'s limits depend on it). Pin it in a unit test.
- **Encodings differ** (ours `from | to<<6 | kind<<12`; movegen `from<<6 | to`,
  kinds 1..3): convert at the `Board` boundary. Castling is king-to-rook in
  both.
- **Keep `MD1`'s split:** checker material, terminality (`format::judge`,
  `cannot_win`, `status`) and `Mover`'s geometry unchanged; no
  `board.terminal()`.
- **Stop** on a non-EP rules mismatch, checker disagreement, a proof changed by
  a non-key cause, or a lost bench cell.

## Cost and deliverables

Rename 1 h, adapter 3 h, tests 2 h, census 1 h, gates 40 min, bench 1 h, docs
1 h; decisive experiment: the `core` spike (10 s, seed 0) after the smoke
differential. Deliver the renamed crate/binary, the adapter and signature
touch-ups, updated tests/counters, `measurements/plan3/`, one ledger row,
`report3.md`, and the doc updates.

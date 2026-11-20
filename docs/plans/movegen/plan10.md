# Plan 10 — adopt `atomic-movegen` 2.11.0 (`W10` evasion-aware legality)

**Item:** `M14` (evasion-aware legality). **Decisions:** `MD5`, `MD6`, `MD8`;
no new decision row. **Session:** execution, owner-triggered. **Predecessors:**
[`report9.md`](report9.md), [`wishlist3.md`](wishlist3.md),
[`decisions.md`](decisions.md) (`MD8`),
[`measurements/plan9/`](measurements/plan9/README.md). **Report:**
`report10.md`. **Budget:** one sitting (~4 h).

## Goal

Pin and adopt the released `atomic-movegen` **2.11.0**, the wishlist-3
release, and wire its `W10` evasion-aware candidate context into the one place
the prover already holds a populated `StateInfo`. Target: plan 9's
**−0.7246e9** on `core` (9.271%), with identical counters/mate, a
byte-identical `.pvr` and 9/9 accepted. The measured release number decides;
the plan does not promise `MD6`.

**Not here:** no prover-side legality (`MD5`), no search/eval/tree/order/seed
change, no `.pvr` or checker change, no second release, and no
`unsafe-optimizations` decision. Report 9 records `uo` as an owner choice
(a)/(b); it is not part of 2.11.0 and is not enabled here.

## Trigger and changelog check (read-only plan session)

- Crates.io index: **2.11.0**, checksum `46b5519a627865f4288aefe4d9b738ae19b4b7cd35d6584a7ff90035c799602d`,
  published 2026-11-20T18:46:17Z. The archive's `.cargo_vcs_info` records commit
  `0aaf9583a9c5536baf860ecdee047b89b8ff087f`; pin that commit unless the
  release tag resolves to it.
- Changelog: 2.11.0 is the **wishlist-3 release**: `wishlist3.md` contains one
  ask, `W10`, and 2.11.0 implements it. It is additive and opt-in, and states
  no change to any existing legality answer, perft value, move order, FEN
  output, terminality answer, `Always`/`Legal` key or `ZOBRIST_VERSION`.
- The new API is `Board::legality_context_with_state(counts, &state)`. It reads
  only `state.checkers`, populates nothing and recomputes nothing. The mask is
  rejection-only: captures, en passant, commoner moves, castling outside check,
  quiet moves outside check and block-ray candidates still use the shared body;
  a stale state can only over-reject. The old check-blind constructor is
  unchanged.
- Upstream tests cover the perft/conformance/effect/edge corpora and seeded
  play-outs, plus slider block in/out, double, knight/pawn check, touching
  commoners, castling/EP in check, a blocking promotion and a second line.
- **Source finding:** `Mover::new` already calls `populate_state` and derives
  `Counts` (`src/moves.rs:67-70`), so adoption is one constructor call on data
  the node has. `Position::legal` (`src/board.rs:356-360`) has no state and
  stays check-blind; a per-call `checkers()` query there is not this plan.

## Pre-registered expectation

Plan 9 is the bound, not a guarantee: the crate's pass set is conservative and
setup costs 0.18–0.26 ns per node. Expect roughly 0.5–0.72e9 on `core`, with
the whole-binary number deciding. The wall row should improve but still miss
`MD6`; that is a recorded result, not a reason to tune.

## Phases

### 0 — owner trigger (~15 min)

Owner confirms 2.11.0 and the commit above. If the archive commit, checksum or
changelog differs from this record, stop and report; do not work around it.

### A — pin and provenance (~30 min)

`Cargo.toml` → `atomic-movegen = "=2.11.0"`; regenerate `Cargo.lock`; record
checksum `46b5519…` and commit `0aaf958…` (`v2.11.0`) in `PROVENANCE.md`.
Update the `wishlist3.md` status table (`W10` released and adopted). The pin
must build before any source edit. Features stay default; `unsafe-optimizations`
is untouched.

### B — adoption (~30 min)

In `src/moves.rs::Mover::new`, replace

`pos.board().legality_context(counts)`

with

`pos.board().legality_context_with_state(counts, &state)`

using the state already used for `tested`. Keep `counts` for the effect path.
No second `populate_state`, no cached state in `Position`, no new field. Update
the stale `src/board.rs::legal` comment: `Position::legal` still reads only
counts, but `Mover`'s cached context is check-aware. Add one focused `moves.rs`
test over in-check FENs (slider in/out, double, castling, EP) asserting
`Mover::legal` equals `Position::legal`; the differential is the broad witness.

### C — measured fall (~1 h)

Use a new `measurements/plan10/run_plan10.py`, derived from plan 9's driver:
build `base` from `git archive HEAD` (2.10.0) and `w10` by applying
`w10.patch` (pin, lock and the one call) in `/tmp/plan10`; run the ignored full
differential in `w10`; run three interleaved `perf stat -e instructions:u`
rounds on `core`, `easy-win` and `deep-win` with `470MI prove FEN --seconds 10
--memory 192 --seed 0 --json`, no export; export one `.pvr` per variant/cell
for sha256 and both checkers. Write `perf.txt`, `results.json`, `env.json`,
`decision.json`.

**Branch (pre-registered).** Adopt the call only if on `core` the median fall
is **≥0.31e9** (the `MD8` rule), every round has identical counters and mate,
the `.pvr` is byte-identical on all three cells, both checkers accept 9/9 and
the differential reports `mismatches: 0`, `key_mismatches: 0`, EP 87/0/0.
Anything else is a stop: revert the pin and the call, leave the tree on 2.10.0
and report the miss. A checker disagreement, a legal move rejected by the
mask, or a non-EP mismatch is a veto regardless of the instruction count.

### D — gates and the `MD6` row (~1.25 h)

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
`cargo doc --no-deps`, `make test`, `make test-full`, `make veto` (9/9 and the
control rejected), and `cargo test --release --test differential
rules_differential_full -- --ignored` (84/11 FENs, 27,088 positions, 540,383
moves/keys, 0/0, `MD3` 87/0/0).

Then one same-session `MD6` row with plan 3's driver: rebuild the plan 2
binary from `2f629fd`, interleave candidate and reference, protocol `a`, seed 0,
10 s, 192 MB, three repeats, `--d14`. Pass still requires 3/3 cells, 9/9
accepted, `D14` identical and geomean ≤ the same-session reference reading
near its 0.972 anchor. If it misses, record the row; do not retune and do not
withdraw the adopted `W10` call that already cleared Phase C.

### E — artefacts and docs (~30 min)

`measurements/plan10/`: README, driver, `w10.patch`, `perf.txt`,
`results.json`, `env.json`, `decision.json`, `bench/{results,old,env}.json`,
`core-perf.txt`; no binaries, `.pvr`, `perf.data`, transcripts or oracle
artifacts. One `movegen plan10` row in [`../../bench.md`](../../bench.md).
`report10.md` <=12 KB. `initiative.md`: close `M14`, update status and history.
`AGENTS.md`: one-line state update. No new `decisions.md` row; `MD8` names the
release route.

## Gates and stop rules

- A pin change without differential, gates, veto and a same-session row is
  incomplete. A C-phase stop reverts both pin and source.
- Only the constructor call and its immediate documentation change are in
  scope. No effect/key, search, table, eval, tree, seed, proof or checker
  change.
- Identical `child_evals`, expansions, nodes, mate and `.pvr` at seed 0 are the
  contract; any movement is a stop, not a new baseline.
- `uo` is not enabled; one measurement shape; no re-tuning to recover a miss.

## Cost and next

Deliver: the 2.11.0 pin and provenance row, the one-call adoption, a measured
`W10` fall table, green gates/veto, one ledger row and `report10.md`. Phase C is
the decisive experiment; the `MD6` row is the product measurement, not the
adoption gate. Next: `M8`, complete shakmaty removal, `check.rs` untouched. The
`uo` owner decision remains open from report 9.

Kickoff prompt:

> Read `docs/plans/movegen/plan10.md`, `report9.md`, `wishlist3.md` (`W10`),
> `decisions.md` (`MD8`) and `measurements/plan9/`. Execute plan 10: pin
> `atomic-movegen` 2.11.0 (`0aaf958`), switch `Mover::new` to
> `legality_context_with_state`, run the differential, gates and veto, measure
> the three-cell fall and the same-session `MD6` row, and write `report10.md`.
> Do not enable `unsafe-optimizations` or change search, eval, tree or checker.

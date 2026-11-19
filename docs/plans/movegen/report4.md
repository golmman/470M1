# Report 4 — adopt `atomic-movegen` 2.10.0

**Plan:** [`plan4.md`](plan4.md). **Items:** `M2`, `M6`, `M7`; decisions
`MD1`, `MD2`, `MD5`, `MD6`. **Date:** 2026-11-19. **Session:** execution,
owner-triggered. **Outcome:** 2.10.0 is pinned and adopted on every rules-path
call the plan names; the differential and both checkers are green. The wall row
is **1.099** against a same-session plan 2 binary at **0.988**, so the `MD6`
bar still does not hold; the result is reported as a near miss with a named
residual, not tuned. Measurements:
[`measurements/plan4/`](measurements/plan4/README.md).

## Phase 0 — trigger

The owner named `atomic-movegen` **2.10.0**. Its changelog records all four
wishlist-2 asks: `W6` `Counts`/`counts`/`populate_counts`, `W7`
`LegalityContext`/`legality_context`, `W8`
`ChildEffect`/`child_effect`/`child_effect_with_occupancy`, and `W9`
`null_for_advanced`. No item was missing and nothing was worked around.
`wishlist2.md` now records the landing per item.

## Phase A — pin and provenance

`Cargo.toml` pins `atomic-movegen = "=2.10.0"`; `Cargo.lock` records
`cf06388…`; `PROVENANCE.md` moves the crate pin from `af3faf0` (`v2.9.0`) to
`5083647db4f8a674e536242fcc1bca7f84c67c04` (`v2.10.0`, 2026-11-19). The crate
builds and its own tests are not vendored here; the release archive checksum
matches the index. No code changed before the pin built.

## Phase B — adoption

`src/board.rs`:

- `Move` now holds the crate's native `Move`; `bits` is the stable 16-bit wire
  encoding and `from_bits` decodes it. The per-candidate
  `native_move`/`from_native_move` translators are gone; `Move::from_native`
  and `native` are identity wrappers. The proof tables still keep a compact
  15-bit projection (`Move::proof_bits`/`from_proof_bits`), because their
  entry is a 32-bit field with a 10-bit length and a 7-bit limit; that
  projection is the only boundary and is not on the rules path. `.pvr` move
  numbers still come from `from`/`to`/promotion, so the format and its order
  did not move.
- `Position::legal` validates the native value, then asks
  `legality_context(counts).legal`, i.e. `W6` counts-only plus `W7`, with no
  `StateInfo` population and no checkers query.
- `Mover::new` still populates one full `StateInfo` because `tested` reads
  `checkers` and `pinned`; it keeps the resulting `Counts`, the
  `LegalityContext` and the parent occupancy. The fast path and the fallback
  both read that one context; no per-candidate state or view is built.
- `Effect` comes from `child_effect_with_occupancy(m, occupied, counts,
  Legal)`: child occupancy, key, `rule50` and capture in one pass on the
  parent's bitboards. `PostMove`, `PositionView` copies and the
  `is_double_push` key re-probe are gone; `ep_threat` keeps its own
  double-push test.
- `null` is `null_for_advanced(!turn)` plus its key; the `Setup`/`from_setup`
  rebuild and the separate key recompute are gone.
- `play` remains a native clone + `do_move` + `Legal` key.

`src/moves.rs` changes are mechanical: the context/occupancy fields replace
the state field on the legality fallback, and the cached occupancy replaces
two accessor calls. `dfpn.rs` and `proofs.rs` only switch their mate/proof
field to `proof_bits`/`from_proof_bits`; search order, evaluation, threats,
counters and `.pvr` writing are untouched. `check.rs` is untouched.

## Phase C — differential and tests

Two identical full sweeps after the final representation:

| Count | Plan 4 |
| --- | ---: |
| FEN accepted / rejected | 84 / 11 |
| Positions compared | 27,088 |
| Legal moves / child keys | 540,383 |
| Play-out games / plies | 392 / 26,612 |
| Immediate explosions | 72,131 |
| Decided children skipped | 6,684 |
| Material exemptions | 642 |
| Mismatches / key mismatches | **0 / 0** |
| Explosion / `exposes` / node mismatches | **0 / 0 / 0** |
| `MD3` `ep_child` / verdicts / differences | 87 / **0 / 0** |

`board::matches_shakmaty` now checks every 16-bit native encoding against the
pseudo-legal list; `moves::matches_the_played_positions`, the EP regressions
and `null_advances_the_clock` stay green. The pinned `core` proof and the
100k-node counters did not change: the bench-cell test still proves and
repeats the recorded tree (14 / 18 / 20), and all nine `.pvr` fixtures were
accepted by both checkers. The compact proof-field projection is identity on
every legal move the search writes.

## Phase D — gates

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | green |
| `cargo doc --no-deps` | green |
| `make test` | green |
| `make test-full` | green (full sweep, slow cells, veto) |
| `make veto` | green: 9/9 accepted, control rejected by both |

## Phase E — per-call and whole-binary

The ten-FEN probe (`measurements/plan4/probe/`) measures the entry points on
this host, median of three `instructions / ops` values:

| Operation | Instructions | Target | Result |
| --- | ---: | ---: | --- |
| `LegalityContext::legal` | **121.9** | ≤100 | **miss** |
| `child_effect_with_occupancy(Legal)` | **215.2** | ≤120 | **miss** |
| `populate_counts` / `counts` | 68.3 / 49.9 | counts-only | used |
| `null_for_advanced` | 35.9 | no rebuild | pass |
| native round-trip (old translator shape) | 18.6 | 0 in prover | no conversion |

The host is ~1.17× the audit's per-call scale (`Board::legal` 168.7 here vs
144 there), but even normalised `W7` is only near 104 and `W8` near 184. So
`W8` did not reach the wishlist's `≤120`; that is an item in the residual, not
something to tune in search or evaluation.

Whole `core` at seed 0, 10 s, no proof export (`perf stat`, median of three):

| Binary | Instructions | Wall | Mate | `child_evals` |
| --- | ---: | ---: | ---: | ---: |
| plan 2 `470m1` | **6.569e9** | 480 ms | 17 | 3,823,720 |
| 2.10.0 `470MI` | **8.004e9** | 571 ms | 14 | 3,896,430 |
| plan 3 (recorded) | 9.026e9 | 639 ms | 14 | 3,896,430 |

The adoption removes 1.02e9 instructions from plan 3 (−11.3%). Against plan 2
the remaining gap is 1.44e9 (1.219×); the audit's optimistic crate-led ceiling
was ~7.35e9 (1.12×), so the release plus adoption is still short of the bound.

## Phase F — bench (input to plan 5)

Protocol `a`, seed 0, 10 s, 192 MB, three repeats, same session. Candidate
2.10.0: **1.225 / 1.160 / 0.935**, geomean **1.099**, 3/3 and 9/9 accepted,
`D14` identical. Same-session plan 2 binary: **0.996 / 1.015 / 0.955**,
geomean **0.988**. Candidate/old per cell is 1.230 / 1.134 / 0.981, geomean
**1.113** — close to the audit ceiling, still not `MD6`. The 2.10.0 tree is
14 / 18 / 20 attacker moves and 3,896,430 / 904,368 / 44,828,946 child
evaluations; the old binary is 17 / 15 / 19. No ledger row is appended here:
plan 5 owns the same-session rebuild and the row.

## Problems and scope changes

- The planned per-call acceptance misses `W7` and `W8`; both are reported and
  passed to plan 5. No solver tuning was attempted.
- `Move` stores the native value, so `bits()` is now the 16-bit wire code, not
  the old 15-bit code. The proof table keeps its own 15-bit projection, which
  kept its 12-byte entries and the recorded counters; unit-test fixtures that
  used arbitrary bit patterns now use valid native moves.
- `Position::legal` deliberately rebuilds a cheap context per call; the hot
  candidate loop uses `Mover`'s cached context. This kept `Position` at
  `{ board, hash }` and avoids the `StateInfo`-in-`Position` negative result.
- No `.pvr`, search, evaluation, threat, checker or oracle change.

## Open parts, missing tests and next steps

- `W8` needs another upstream shape if the bar is to be reached: the child
  query is still ~215 instructions per candidate on this host. `W7` is close.
- Plan 5 must rebuild plan 2 from `2f629fd`, interleave both binaries, record
  the ledger row and decide pass / near miss / regression. This report's
  1.099 is a plan-4 input, not a closure.
- No new tests are missing for the adopted surface: the native-encoding sweep,
  differential, fixtures, veto and counter pins cover it. A future release
  needs the same cycle.

**Handoff residual to plan 5:** legality 121.9 (target 100), effect+key 215.2
(target 120), `core` 8.004e9 vs 6.569e9 plan 2, tree 14 vs 17 attacker moves,
`Mover` accessor traffic and full-state `populate_state` still present.

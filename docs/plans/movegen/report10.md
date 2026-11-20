# Report 10 — adopt `atomic-movegen` 2.11.0 (`W10`)

**Plan:** [`plan10.md`](plan10.md). **Item:** `M14` (closed). **Decisions:**
`MD5`, `MD6`, `MD8`; no new decision row. **Date:** 2026-11-20.
**Session:** execution, owner-triggered. **Tree:** `470MI@a79f68f` + this
plan's working tree. **Outcome:** **adopted**. The `W10`
`legality_context_with_state` call falls **0.6686e9** instructions on `core`
(8.554%) against the 0.31e9 rule, with identical counters/mate, a
byte-identical `.pvr` on all three cells, 9/9 accepted by both checkers and a
0/0 differential. The same-session `MD6` row is 3/3 and 9/9 with `D14`
identical, geomean **1.018** against the plan 2 binary's **0.991**
(candidate/old **1.028**): still a miss, but `core` narrows from 1.190× to
**1.088×** plan 2. Measurements:
[`measurements/plan10/`](measurements/plan10/README.md).

## Phase A — pin and provenance

Phase 0 matched the plan record exactly: crates.io 2.11.0, checksum
`46b5519a627865f4288aefe4d9b738ae19b4b7cd35d6584a7ff90035c799602d`,
published 2026-11-20T18:46:17Z; the archive's `.cargo_vcs_info.json` records
`0aaf9583a9c5536baf860ecdee047b89b8ff087f`. `Cargo.toml` pins `=2.11.0`;
`Cargo.lock` carries the same checksum; `PROVENANCE.md` moves the source row
from 2.10.0 (`5083647`) to 2.11.0 (`0aaf958`, `v2.11.0`). Features stay
default; `unsafe-optimizations` is untouched. The pin built before any source
edit.

## Phase B — the one call

`src/moves.rs::Mover::new` already populated `StateInfo` and derived `Counts`
for the tested/pinned fields. The only functional change is

```rust
let context = pos.board().legality_context_with_state(counts, &state);
```

No second `populate_state`, no cached state in `Position`, no new field.
`src/board.rs::legal`'s comment now says that its own query is check-blind
while `Mover`'s cached context is check-aware. One focused test,
`moves::tests::evasion_context_matches_position_legal`, covers a slider block
ray, a knight check, a double check, castling in check and en passant in check
and asserts `Mover::legal == Position::legal` for every pseudo-legal move. The
full differential is the broad witness.

## Phase C — measured fall

`run_plan10.py` built `base` from `git archive HEAD` (2.10.0) and `w10` by
applying `w10.patch` under `/tmp/plan10`; `base` is sha256 `b7c16c94…`, the
same binary plans 8 and 9 measured. Three interleaved `perf stat -e
instructions:u` rounds, `prove FEN --seconds 10 --memory 192 --seed 0 --json`,
no export:

| Cell | base median | `w10` median | fall | fall % | wall (ms) |
| --- | ---: | ---: | ---: | ---: | --- |
| `core` | 7,816,219,420 | 7,147,608,959 | **668,610,461** | 8.554 | 559 → 529 |
| `easy-win` | 1,640,875,516 | 1,560,746,666 | 80,128,850 | 4.883 | 136 → 132 |
| `deep-win` | 82,738,904,016 | 77,783,667,935 | 4,955,236,081 | 5.989 | 5690 → 5482 |

`core` pair falls are 668,610,558 / 668,610,461 / 668,610,217. The rule is
`core` median fall ≥ 0.31e9; the release branch is taken. The pre-registered
expectation was 0.5–0.72e9, and the crate-side mask lands at 0.6686e9,
below plan 9's 0.7246e9 prototype (the crate adds a per-node setup and a
destination test) and inside the expected band. No tuning was done.

Correctness, all cells: counters and mate identical in every round; `.pvr`
byte-identical (`core` `714723211c7a0246…`, `easy-win`
`32ffb30d96ccb172…`, `deep-win` `50f062de2a67b1dd…`); our checker and
`provari check` accept 9/9; the full differential reports
`mismatches: 0`, `key_mismatches: 0`, EP 87/0/0, 27,088 positions and 540,383
moves/keys. `decision.json` records the release branch.

## Phase D — gates, veto and the `MD6` row

All green: `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`;
`cargo test` (51 unit tests plus the integration suite, 0 doc tests);
`cargo doc --no-deps`;
`make test`; `make test-full`; `make veto` (9 fixtures accepted, the control
rejected); and the release differential

```sh
cargo test --release --test differential rules_differential_full -- --ignored
```

with 84/11 FENs, 27,088 positions, 540,383 moves/keys, 0/0 and `MD3` 87/0/0.

The `MD6` row used plan 3's driver, protocol `a`, seed 0, 10 s, 192 MB, three
repeats, `--d14`; the plan 2 binary was rebuilt from `2f629fd` in
`/tmp/plan10/plan2` and is sha256 `71fff646…`, bit-identical to plan 7's
rebuild. Candidate binary `2eb218a6…`.

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.511 s | 0.578 s | **1.131** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.132 s | 0.143 s | **1.083** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.527 s | 5.627 s | **0.862** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.018** | | | **9/9** |

Same-session plan 2 binary: 0.998 / 1.015 / 0.960, geomean **0.991** (anchor
0.972), 9/9, `D14` identical. Candidate/old is **1.133 / 1.067 / 0.898**,
geomean **1.028** (plan 7: 1.070), so `MD6` is **not held**. The row is
recorded; the adopted call is not withdrawn. `core-perf.txt` gives the
interleaved fall against plan 2 directly: **7,147,608,629** vs
**6,569,377,149**, a 1.088× gap (was 1.190×).

## Phase E — artefacts and docs

`measurements/plan10/` holds the README, driver, `w10.patch`, `perf.txt`,
`results.json`, `env.json`, `decision.json`, `core-perf.txt` and
`bench/{results,old,env}.json`. No binary, `.pvr`, `perf.data`, transcript or
oracle artifact is committed. [`../../bench.md`](../../bench.md) gains one
`movegen plan10` row and pin line; [`initiative.md`](initiative.md) closes
`M14`, updates the pin prerequisite, status and history; `AGENTS.md` gets the
one-line state update; [`wishlist3.md`](wishlist3.md) records `W10` released
and adopted. `decisions.md` is unchanged: `MD8` already names the release
route.

## Problems, scope changes and what was not done

- **Host label.** The driver's `bench_host`/`provisional` flags are a
  `socket.gethostname()` equality and read `false`/`true`: the container's UTS
  name is `36f35886a6b3`, not `c90face26537`. Host identity rests on the
  platform string, the unchanged `base` binary sha256 `b7c16c94…` and the
  `core` reading 7,816,219,420, which reproduce plan 9's
  `7,816,219,508`. This is the same procedural deviation report 9 recorded; it
  is visible in `decision.json` and `env.json`.
- **Two binaries for one source.** The C-phase `w10` binary is `62986012…`;
  the product binary measured in the `MD6` row is `2eb218a6…`. They differ
  because the product tree also carries the documentation comment (which
  shifts compiled line metadata) and the `#[cfg(test)]` test; the release code
  path, counters, mate and `.pvr` are identical, and `core-perf.txt` measures
  the product binary at the same instruction count.
- **`env.json` overwrite.** The plan 3 driver writes `env.json` for any output
  whose stem does not contain `results-`, so the old-binary pass overwrote the
  candidate environment record. The committed `bench/env.json` reconstructs
  the candidate fields and records both binaries and hashes; the note is
  explicit. No other artifact was affected.
- **Crate target was a prototype bound.** Plan 9's −0.7246e9 was a prover-side
  diagnostic; the crate's released shape is 0.6686e9. That is inside the
  plan's 0.5–0.72e9 expectation, so it is a result, not a regression.
- **`uo` remains open.** `unsafe-optimizations` was measured in plan 9 and is
  not enabled, not re-measured and not decided here. It would take another
  ~0.10e9 from `core` if the owner chooses option (a), still short of `MD6`.
- **No `Position::legal` change.** It remains check-blind; only `Mover`'s
  cached context is check-aware. A per-call `checkers()` query there is not
  this plan.
- **Missing tests.** There is no test for the host-label identity and no
  prover-side test that the mask's rejection set equals the crate oracle's;
  the focused test compares against `Position::legal`, and the crate's own
  release tests carry the oracle comparison. The full differential covers the
  prover's uses.

## Distance to the goal

`core` is **7.1476e9** vs plan 2's **6.5694e9**, **1.088×** (gap 0.578e9), down
from 1.190×. `MD6` is still a miss: geomean 1.018 against 0.991, and even
`uo`'s ~0.10e9 would leave ~1.072×. `M8`, complete shakmaty removal, is the
only open item; `check.rs` stays untouched. The `uo` owner decision from
report 9 remains open.

## Next steps and options

1. **`M8`, complete shakmaty removal (recommended).** The only path to the
   initiative's done definition; independent of the remaining performance
   gap. Plan the parser/display/proof-path replacement without touching
   `check.rs`.
2. **`uo` owner decision.** If (a), enable `unsafe-optimizations` with the
   provenance note and the upstream feature-test gate; expect ~1.3% and no
   tree change. If (b), close it.
3. **Kill review.** `movegen` still has none after plans 4, 6, 8 or 10; run
   the fixed prompt before the next plan.

Option 1's kickoff prompt:

> Read `docs/plans/movegen/initiative.md`, `report10.md`, `report9.md`,
> `decisions.md` (`MD1`, `MD5`, `MD6`, `MD8`) and `measurements/plan10/`. Plan 11:
> complete `M8`, removing shakmaty from the prover (parser, display, proof
> paths, line output) while `check.rs` stays on shakmaty; keep the differential
> green, gates/veto green, and run the same-session `MD6` row. Do not enable
> `unsafe-optimizations` without the owner's decision.

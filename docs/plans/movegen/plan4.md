# Plan 4 — the `M1` per-node regression: measured rollback or `W6`

**Items:** `M1` (adapter hot path), `M2`, `M6`; decisions `MD1`, `MD2`.
**Session:** plan (read-only probes only). **Predecessor:** [`report3.md`](report3.md)
— adapter green, 10 s row 3/3, geomean **1.206**, same-session plan 2 binary
**0.987**. **Successor:** `report4.md`.

## Goal

Return the 10 s row to 3/3, 9/9 accepted, geomean **<= 1.0** with no search,
eval, proof-format or checker change. This plan decides **which hot path** to
keep; it does not tune the solver.

**Not here:** no eval/threats/search change, no `.pvr` change, no checker
change, no path dependency, no new `atomic-movegen` pin without an
owner-triggered plan.

## Diagnosis (measured this session)

Host `c90face26537`, one process at a time, stored plan 2 binary `470m1` vs
plan 3 `470MI`. `core` 30x for the clock profile; single-run `perf stat` for
instructions/cycles.

| Metric | plan 2 | plan 3 | Delta |
| --- | ---: | ---: | ---: |
| task-clock (30 runs) | 15.06 s | 20.41 s | **+35.5%** |
| instructions | 7.06e9 | 10.17e9 | **+44%** |
| cycles | 2.43e9 | 3.23e9 | +33% |
| IPC | 2.91 | 3.15 | +0.24 |
| instructions / `child_evals` | 1,846 | 2,610 | **+41%** |

The tree grew only 2.4% (342,263 → 350,430 nodes), so the loss is per node.
Same pattern on `easy-win` and `deep-win` (about +30% instructions/eval
each), so it is not one cell.

**No single hotspot.** Instruction share by bucket, absolute e9:

| Bucket | plan 2 | plan 3 | Delta |
| --- | ---: | ---: | ---: |
| `pseudo_legal` (+hand `pawn_moves_to`) | 0.63 | 0.91 | +0.28 |
| legality (`legal`/`checked_after` vs `legal_with_counts`) | 0.94 | 1.57 | +0.63 |
| `effect` + child key | 0.35 | 0.61 | +0.26 |
| `checkers` + `populate_state` | 0 | 0.53 | +0.53 |
| `native_move` conversions | 0 | 0.22 | +0.22 |
| `play`/`do_move` + `memmove` | 0.22 | 0.36 | +0.14 |
| `null`/`from_setup_parts` | 0.02 | 0.14 | +0.12 |
| `exposes`/`plain`/`gives_check`/`replies` | 0.67 | 1.03 | +0.36 |
| other (`attackers_to`, `is_pseudo_legal`, layout) | — | — | ~+0.5 |

Per expanded node (counter probe, 350,430 nodes): `pseudo_legal` 1.86 calls,
`Mover::new` 1.74, `Mover::legal` 43.7 (**34% fast, 66% full `board.legal`**),
`effect` 13.6, `play` 4.0, `in_check` 1.45 (690 instructions/call),
`populate_state` 1.74.

The three named functions are the largest rules-layer share but only
**1.17e9 of the 3.1e9** gap:

- **`pseudo_legal`** — 1,391 instructions/call vs 964: the native generator
  plus `from_native_move`, and it is called 1.86x/node because `open_group`
  re-builds the node constants.
- **`legal`** — 66% of candidates reach `legal_with_counts` (~103 instructions
  vs ~62 for plan 2's occupancy `checked_after`). Movegen's own
  `for_each_legal_with_state` ladder was prototyped: **10.16e9, no gain**.
- **`effect`** — our blast/occupancy derivation plus `hash_and_rule50_after`'s
  separate classify and key (~128 instructions/call vs ~75); two child
  derivations where one should do.

Prototypes (HEAD + patch in `/tmp`, never committed): occupancy `legal_fast`
alone **9.88e9** (−2.7%); adding `in_check` reuse from `children`, a
`play_with_hash`, and inlined conversions **9.68e9** (−4.7%), `core` 618 vs
649 ms (−4.8%), `easy` −3%, `deep` −2.4%. **The whole cheap bundle is ~5%,
not the 20–26% the 10 s row needs.** The residual is `atomic_movegen::Board`'s
general-purpose per-candidate cost, not a missing fast path.

**Conclusion.** The smallest *measured* change that reaches `<= 1.0` is to
keep the plan 2 hot path (0.987 same session). Micro-optimising `M1` cannot get
there; claiming it can would be an untested assumption.

## Options

1. **Chosen — hot-path rollback, movegen as oracle (`MD1` amendment).**
   Restore plan 2's provari-derived `board.rs`/`moves.rs` under the new name:
   cached colour/role bitboards, hand generator, occupancy legality, Zobrist
   key. `shakmaty` stays parser + checker only; `atomic-movegen` stays the
   differential oracle and is still used for FEN acceptance. The prover does
   **not** use movegen's `Board` for generation/legality/effect. Measured
   gain: 1.206 → **0.987** (stored binary, same session). Cost 4–6 h, low
   risk, licence unchanged (`PROVENANCE.md` already covers both). Requires a
   new dated `MD1`/`MD2` row: `MD1` becomes "no shakmaty in the prover's
   rules path; the provari-derived hot board stays, movegen is the oracle and
   the geometry source". `M8` then swaps the remaining shakmaty geometry and
   parser for movegen's public `attacks`/FEN APIs — a mechanical change, not a
   Board adoption.
2. **Full replacement, `W6` upstream.** Public view-based rules API (no
   `Board` write on the hot path), batch `legal + effect + key` from one
   derivation, null with an advanced clock; plus `M2` native move storage and
   shared node constants. Est. 10–20%, **may still miss 1.0**; two plans plus
   an owner-triggered release. Cheapest experiment: a local crate patch in
   `/tmp` (not a committed path dependency), `core` 10 s; reject if < 10%.
3. **Hybrid:** rollback the generator/legality, keep native `Legal` keys. Not
   recommended: the key is not the regression (`M3` diagnostic was 0.987 with
   the old key) and it would keep a `Board` per node.

## Phases (Path 1)

### 0 — owner decision (~15 min)

Owner resolves `MD1`: complete movegen replacement is **not** compatible with
the no-regression gate at 2.9.0. Either amend `MD1` (chosen) or fund Path 2.
No code before this row.

### A — rollback (~3 h)

Restore `2f629fd:src/board.rs` and `:src/moves.rs`, keeping the `atomic470mi`
name and the plan 3 signature touch-ups in `walk`/`lines`/`format`. `Position`
returns to cached bitboards + its own Zobrist. Keep the movegen differential
(as an independent oracle, not the prover's rules), the EP census and the EP
regressions. Key equality becomes a recorded divergence again (`MD2` row).

### B — tests (~2 h)

`tests/differential.rs`: rules-only comparisons stay; native-key equality is
marked a recorded divergence. `board::matches_shakmaty` and `moves` tests
return with their plan 2 shape; the pinned `dfpn` core proof/counters return
to the plan 2 values (17 moves). `.pvr` fixtures must not change: proof keys
are not stored. `check.rs` untouched.

### C — gates (~40 min)

`cargo fmt --check`, clippy `--all-targets -D warnings`, `cargo test`,
`cargo doc`, `make test`, `make test-full`, `make veto` (9/9 + control
rejected). Differential: 0 mismatches, EP counters 0/0/0, two identical runs.

### D — bench and decision (~1 h)

Plan 3's driver, one session, protocol `a`, 10 s, 192 MB, 3 repeats, seed 0.
Bar: **3/3, 9/9, geomean <= 1.0**. Commit driver/results/env/README under
`measurements/plan4/`, no raw transcripts. If the row misses, stop and report;
do not tune search/eval to hide it.

### Path 2 (only if `MD1` is kept)

`W6` wishlist + upstream release, then `M2` + shared node constants +
`EffectContext`. Accept only if `core` per-node instructions are within **+5%**
of plan 2 and the 10 s row holds; otherwise stop and report.

## Risks and stop rules

- **Rollback is a scope change:** it reverses `MD1`'s "all rules on movegen".
  A new dated decision row is mandatory; no silent reversal.
- **Key parity:** plan 2 keys differ from movegen's; that is recorded, not a
  correctness gap (proofs carry no keys).
- **Do not tune search/eval.** A rules-layer cost is not a solver result.
- **Stop** on a non-EP rules mismatch, checker disagreement, a proof changed
  by a non-key cause, or a lost cell. Path 2 stops on the +5% instruction bar.
- No third micro-optimisation round: two measured attempts are enough.

## Cost and deliverables

Decision 0.25 h, rollback 3 h, tests 2 h, gates 0.7 h, bench 1 h, docs 1 h;
decisive experiment is the `core` bench of the restored binary against the
same-session reference (the stored 0.987 binary already bounds it). Deliver
the amended decision row, restored hot path, updated differential/tests,
`measurements/plan4/`, one ledger row, `report4.md`; then `M8` (parser/display)
can proceed from a no-regression base.

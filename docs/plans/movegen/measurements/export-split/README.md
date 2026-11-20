# Export-split measurements — the proof walk on `core`

**Date:** 2026-11-20. **Host:** bench host (`36f35886a6b3`, platform
`Linux-7.2.13-200.fc44.x86_64`). **Pins:** plan 12 product (`8d807f21…`),
plan 2 product (`71fff646…`), provari v0.4.0 `bceb1ee` (`7fc4c201…`),
`atomic-movegen` 2.11.0 at `0aaf958` (unchanged). **Not a ledger row:** this is
the counted split named as the smallest next experiment in
[`kill-review1.md`](../../kill-review1.md) §5. No `src/`, pin or checker change.

## Files

- `run_export_split.py` — the clean driver: 3 interleaved rounds of
  {with `--proofs`, without} under `perf stat -e instructions:u`, for the plan
  12 product, the plan 2 product and the oracle; plus the oracle with
  `SKIP_WALK`. Writes `results.json`'s `clean_instructions`.
- `instrument.patch` — applied to a **copy** of the tree (never to `src/`):
  thread-local counters in `judge`, `prove`'s candidate loop, `wins_in_two`,
  `capture_wins`, `attacker_to_move`, `defender_to_move`, `judge`, and in
  `Position::{play,legal_moves,pseudo_legal,legal,in_check}` and
  `moves::{has_legal_move,cannot_win}`; unique-call sets; a probe of the
  search's proof table; a fast-length variant. Dumps one JSON line to stderr
  around `format::encode`.
- `reuse.patch` — the one byte-identical micro-optimisation tried: keep the
  candidate-loop child and reuse it for `wins_in_two` instead of replaying it.
- `results.json`, `env.json` — parsed numbers, derived attribution and env.
- The proofs are not committed; the plan 12 `core` proof is
  `714723211c7a0246…` (1009 positions, 14 moves) and the plan 2 / provari one
  is `2d33c6268132a159…` (1021 positions, 17 moves).

## Result

Clean instruction deltas (with `--proofs` minus without), `core`, median of 3:

| Binary | without | with | export delta | proof |
| --- | ---: | ---: | ---: | --- |
| plan 12 (movegen) | 7,061,619,917 | 7,831,655,667 | **770,035,750** | 1009 pos |
| plan 2 (shakmaty) | 6,569,377,084 | 7,055,759,475 | 486,382,391 | 1021 pos |
| provari v0.4.0 | 7,014,722,480 | 7,016,068,764 | 1,346,284 | 1021 pos |

**The oracle always walks.** Its `--proofs` only controls whether the bytes
are written: `mate` is the walk's length in both arms. Verified with the
instrumented provari (the counter dump appears without `--proofs`). Isolating
the walk with `SKIP_WALK` on a clean build gives **491,509,701** instructions,
so `provari`'s 1.35e6 with/without delta is the file write alone. The plan 12
walk is **1.58×** plan 2's and **1.57×** the oracle's; plan 2 matches the
oracle (486e6 vs 492e6). The export regression is the rules layer's per-call
cost on the walk's call mix, not format logic.

## Counters (`core`, plan 12 walk)

| Category | Count | Note |
| --- | ---: | --- |
| `prove` calls | 9,617 | only 1,009 positions are written |
| legal moves scanned | 359,688 | first loop: play + `in_check` each |
| checking candidates | 29,937 | each replayed for `wins_in_two` |
| `wins_in_two` calls | 13,485 | 2,502 succeed; 56,498 replies total |
| `judge` calls | 42,844 | 346 `Failed`, rest `Open` |
| `legal_moves` calls | 42,844 | 72.4% from `wins_in_two` |
| `play` calls | 452,726 | 79.5% from the candidate loop |
| legality checks | 1,594,553 | `Position::legal` per candidate |
| checker `won_positions` | 9,617 | equals `prove`: the proof tree itself |

## Is any of it re-derivable?

**Proof-table answers: no.** The table holds a proof for 6,688 of the 9,617
prove calls and misses 2,929; of the hits, 5,804 lengths equal the walk's and
884 differ (887 length units, almost all ±1). On the decision that matters,
`wins_in_two`, the table is not an oracle: of 2,502 actual successes **0** are
labelled length 2 by the table, and all **311** table-length-2 cases are false
positives. Using the table would change the written moves/lengths, so it cannot
be byte-identical.

**Positions already proven in the walk: ~0.08e9 at the very most.** The exact
repeat counts are 185 proves, 515 `wins_in_two`, 4,886 `legal_moves`, 3,533
`judge`, and 30,003 `play` (the checking replays). Even returning every repeat
for free is <0.08e9. The measured byte-identical `reuse.patch` variant keeps
the `.pvr` sha256 and all counters but *adds* 55.9e6 instructions: the cache
bookkeeping costs more than the 30,003 replays it saves.

Derived attribution (symbol deltas ÷ these call counts; a model, not a second
measurement): `wins_in_two` ≈ 0.29e9 (38%), candidate scan ≈ 0.22e9 (28%),
strategy-tree write ≈ 0.09e9 (13%), deep prove ≈ 0.03e9, judge/other ≈ 0.14e9.
This is the per-candidate rules work (MD7), not format logic.

## What this cannot see

Per-call costs are derived from the kill-review's percentile symbol delta, not
from a call-level instruction counter; the phase split is an estimate whose
total is the measured 0.770e9. The `SKIP_WALK` oracle build is diagnostic (its
proof is empty) and differs from the pinned binary by one branch. No cycle,
cache or branch counters were taken.

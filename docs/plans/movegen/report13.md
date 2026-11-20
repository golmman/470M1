# Report 13 — export-walk split; `movegen` closes with the dated miss

**Item:** none (follow-up to [`kill-review1.md`](kill-review1.md) §5).
**Plan:** none — this is the counted split the review named, not a plan.
**Measurements:** [`measurements/export-split/`](measurements/export-split/README.md).
**Budget:** one sitting. **Tree:** `470MI@e3c237e`; scratch copies only, no
`src/` edit, no pin change, no ledger row.

## Decision

The pre-registered rule was: if **≥0.31e9** of the **0.770e9** `core` export
delta is re-derivable work (proof-table answers, or positions already proven in
the walk), propose one encoder plan; otherwise the cost is the `MD7` residual
and `movegen` closes with the dated miss.

**The re-derivable share is at most ~0.08e9, and the one byte-identical
implementation measured is slower.** No encoder plan. The 0.770e9 is the
per-candidate rules work: `wins_in_two` and the candidate loop re-derive the
format's canonical move, and the proof table does not hold that answer.

Recommendation: close `movegen` on `MD1`'s done definition (prover replaced,
checker independent, differential 0/0, `.pvr` byte-identical) and keep `MD6` as
the dated miss with this decomposition. Do not open a search plan for it.

## What was measured

Clean `core` instruction deltas (with `--proofs` minus without, median of 3,
one process at a time):

| Binary | without | with | export delta | proof |
| --- | ---: | ---: | ---: | --- |
| plan 12 (movegen) | 7,061,619,917 | 7,831,655,667 | **770,035,750** | 1009 pos |
| plan 2 (shakmaty) | 6,569,377,084 | 7,055,759,475 | 486,382,391 | 1021 pos |
| provari v0.4.0 | 7,014,722,480 | 7,016,068,764 | 1,346,284 | 1021 pos |

The measured proof is byte-identical to plan 12's
(`714723211c7a0246…`, 1009 positions, 14 moves) with identical
`child_evals` 3,896,430 and `mate` 14. The probe binaries are not products, so
`9/9` was not re-run; the proof is the one already accepted 9/9 by
`470MI check` and `provari check`, and no product byte changed. Both checkers
read it back as **`won_positions: 9617`** — exactly the walk's `prove` count.
The 1,009 written entries are the move-bearing nodes; the 9,617 are the
distinct `(position, counter)` won attacker nodes of the proof tree. The walk
is visiting the proof it writes, not re-proving an off-line extra.

## Correction: the oracle does not skip the walk

`kill-review1` priced the candidate's export against a with/without oracle
delta of +4.7 ms and concluded the oracle's walk was free. It is not: provari
computes `mate` from `format::walk` in both arms, and `--proofs` only decides
whether the bytes are written. The instrumented oracle dumps the same counters
**without** `--proofs`. A `SKIP_WALK` short-circuit on a clean build isolates
the oracle's walk at **491,509,701** instructions; its 1.35e6 with/without
delta is the file write alone.

So the real comparison is walk to walk: plan 12 **0.770e9**, plan 2
**0.486e9**, oracle **0.492e9**. The movegen walk is **1.58×** plan 2's and
**1.57×** the oracle's, and plan 2 matches the oracle. The export gap is the
rules layer's per-call cost on the walk's call mix, not format logic and not an
algorithmic difference — the walk is the copied provari walk, and the candidate
writes a *shorter* proof (1009 vs 1021 positions).

The ledger row already charges both sides a walk, so it is walk-for-walk. The
review's 0.944 "search-only" comparison removed the candidate's walk but left
the oracle's in, so it is not a clean search-only decomposition; it flatters
the candidate. The row's letter (1.001 product, 1.013 `MD6`) is unchanged.

## The two re-derivable classes

**Proof-table answers: not usable byte-identically.** The instrumented walk
asks the search's proof table for every attacker position. It holds a proof for
6,688 of the 9,617 `prove` calls and misses 2,929; of the hits 5,804 lengths
equal the walk's and 884 differ (887 length units, almost all ±1). On the
decision the walk actually makes, `wins_in_two`, it is not an oracle: of 2,502
actual successes **0** are labelled length 2 by the table, and all **311**
table-length-2 cases are false positives. The table stores the search's proof,
which is not the format's canonical answer (a check that wins in two, or the
first mate-in-one in the format's move order). Substituting it changes the
written moves or the header length, so it cannot keep the `.pvr` identical.

**Positions already proven in the walk: ~0.08e9 at the very most.** Exact
repeats in the 0.770e9 walk: 185 `prove`, 515 `wins_in_two`, 4,886
`legal_moves`, 3,533 `judge`, and 30,003 `play` — the checking candidates
replayed after the candidate loop. The independent checker's 9,617
`won_positions` equals the 9,617 `prove` calls, so the proves are the proof
tree itself; the only within-walk repeats are the micro-repeats above. Even
returning every repeat for free (ignoring the lookup) is below 0.08e9. The
one byte-identical change tried, `reuse.patch` (keep the candidate child and
pass it to `wins_in_two`), keeps the `.pvr` sha256 and every counter but
**adds 55.9e6 instructions**: moving the children into and out of a map costs
more than the 30,003 replays saved.

## Where the 0.770e9 goes

Exact call counts, with per-call costs derived from `kill-review1`'s symbol
delta (a model whose total is the measured 0.770e9):

| Phase | ≈ instructions | Share | Re-derivable? |
| --- | ---: | ---: | --- |
| `wins_in_two` pattern | 0.290e9 | 38% | **no** (table agrees 0/2,502) |
| `prove` candidate scan | 0.216e9 | 28% | no (needs every legal move and its check) |
| strategy-tree write (defender children) | 0.095e9 | 13% | no (table misses 30%, lengths differ) |
| deep `prove` | 0.034e9 | 4% | no |
| judge and unattributed | 0.136e9 | 17% | intra-walk repeats only |

Caller split: `wins_in_two` owns 30,999 of 42,844 `legal_moves` calls (72.4%);
the candidate loop owns 359,688 of 452,726 `play` calls (79.5%) and 29,937
checking replays. The cost is the rules layer's per-candidate work, exactly the
`MD7` residual, at a call mix the search hot path does not isolate.

## Problems and open parts

- **The table is not a canonical-walk oracle.** It stores search proofs, not
  the format's first-in-order mate or its check-wins-in-two decision. No
  encoder can read it without changing the file. A search that stored the
  canonical walk result would be a search change (`MD1` non-goals) and would
  grow the 12-byte entry, not a one-plan encoder change.
- **A walk memo is a loss, not a lever.** The measured byte-identical memo for
  the play class is negative; the other classes are smaller. There is no
  ≥0.31e9 re-derivable block.
- **The per-call costs are derived, not counted per call.** The phase split
  multiplies exact counts by percentile symbol deltas. It closes to ~0.75e9 of
  the measured 0.770e9; the residual is un-attributed.
- **No product change was made.** `src/`, the checker, the pin and
  `PROVENANCE.md` are untouched. The scratch patches are diagnostic and live
  only under `measurements/export-split/`.
- **The `PROVENANCE.md` tag label** (`v2.11.0`, which upstream has not tagged)
  is still a label-only fix for the owner; this report does not change it.

## Distance to the goal

`MD1` and the checker goal remain met. `MD6` is **missed** and now fully
decomposed: the `core` product is 1.075× plan 2 search-only and 1.110× with the
walk; the walk itself is 1.58× plan 2 / 1.57× the oracle, and it is the rules
layer, not a re-derivable redundancy. A perfect intra-walk memo would recover
<0.08e9, inside the instrument's own spread. The letter still reads 1.001
product / 1.013 `MD6`.

## Next steps and options

1. **Close `movegen`** on `MD1`'s done definition, append no ledger row, keep
   `MD6` dated as a miss with this decomposition, and re-anchor `MD6` at the
   next row. This is the recommendation.
2. **Record the search-only geomean with the corrected oracle arm** as a
   diagnostic column (the oracle always walks), and say in the ledger that the
   row's gap is the rules layer's walk cost.
3. **Leave the export alone.** The only remaining lever inside the prover is a
   per-call rules optimisation (`legal_moves` rebuilds a legality context per
   candidate), which is `MD7` work, not an encoder plan, and is not promised.

Option 1's kickoff prompt:

> Close the `movegen` initiative on `MD1`'s done definition. Read
> `docs/plans/movegen/initiative.md`, `decisions.md`, `report13.md` and
> `measurements/export-split/`. Update the initiative status to closed, keep
> `MD6` a dated miss with the export decomposition, record that the oracle
> always walks, append no ledger row, and change no source or pin.

# Plan 9 — evasion mask and `unsafe-optimizations`: measure on the bench host

**Item:** new `M14` (evasion-aware legality screen). **Decisions:** `MD5`,
`MD6`, `MD7`; a dated `MD8` row only on the release branch. **Predecessors:**
[`report8.md`](report8.md), [`decisions.md`](decisions.md) (`MD7`),
[`hot-path-audit.md`](hot-path-audit.md). **Report:** `report9.md`.
**Budget:** one sitting (~3 h): A 1.5 h on the bench host, B 1 h, C 0.5 h.

## Goal

Measure two levers that `MD7` did not test, on `core`, `easy-win` and
`deep-win`, each as three interleaved `perf stat -e instructions:u` pairs
against the unchanged tree on `c90face26537`:

- **`ev`:** a per-node block mask. When the side to move is in check by a
  single slider and the kings are apart, a quiet non-king move must land
  between the king and the checker. Any other quiet non-king move is rejected
  without calling the oracle; double or non-slider check leaves the mask empty.
  Castling in check is rejected. Patch: [`ev.patch`](measurements/plan9/ev.patch),
  the `/tmp/mg-ev` shape.
- **`uo`:** `atomic-movegen` 2.10.0's `unsafe-optimizations` feature, which
  drops the bounds checks from the Zobrist and magic table loads. Patch:
  [`uo.patch`](measurements/plan9/uo.patch). There is no API or semantics change.

`ev` is prover-side legality, which `MD5` forbids adopting. Here it is a
diagnostic bound for a crate ask, and it is never committed to `src/`.

## Why this reopens a closed question

`MD7` rests on plan 8's census: 82.0% of the 10.01M fallback candidates are in
check, and the ladder could only defer them to the oracle. The plan-session
census (`/tmp/evasion-probe`, an instrumented `Mover::legal`, seed 0) counts
what an evasion mask settles. Call counts do not depend on the architecture.

| `core` quantity | Count |
| --- | ---: |
| in-check oracle calls | 8,220,516 |
| — quiet non-king | 6,128,196 |
| — **rejected by the mask** | **5,882,026** (0 of them legal) |
| — castling / EP | 116,128 (0 legal) |
| `Mover::new` in check / single-slider check / double check | 337,911 / 307,515 / 1,135 |

On `easy-win` the mask rejects 761,648 of 1,038,152 in-check calls, and on
`deep-win` 45,072,205 of 61,347,947. Every rejected call is illegal under the
oracle on all three cells. On `deep-win`, 21 castling/EP calls in check are
legal; the probe does not split the two (hypothesis: EP, passed to the oracle).

**Pre-registered projection.** The upper bound is `5.88M × 120.9` ≈ **0.71e9**:
each rejected call saves the whole-binary oracle cost, and the mask test and the
per-node `between` are about 0.01e9. Plan 8's lesson is that this projection
shape was about 2× optimistic, which gives an expected **~0.35e9**. That is
just above the 0.31e9 rule, so only the measurement decides. Earlier aarch64
wall runs are not an input.

## Phase A — measurement (owner, bench host, ~1.5 h)

The driver is [`run_plan9.py`](measurements/plan9/run_plan9.py). It needs a
clean `src/`/`tests/`/`Cargo.*`, `perf`, cargo and optionally the pinned
provari at `/tmp/470M1-oracle`. Run:

```sh
python3 docs/plans/movegen/measurements/plan9/run_plan9.py
```

It does the following, read-only for the repo, with scratch trees in
`/tmp/plan9`:

1. `git archive HEAD` → `base`, `ev` (+`ev.patch`), `uo` (+`uo.patch`);
   `cargo build --release --locked` each.
2. `rules_differential_full` (release, ignored test) in `ev` and `uo`. This
   test exercises `Mover::legal`, so the mask itself is covered.
3. Three rounds. Per round and cell the order is `base`, `ev`, `uo`, each run
   as `perf stat -e instructions:u -x, -- 470MI prove FEN --seconds 10 --memory
   192 --seed 0 --json` with no export. The pairs are `(base_r, ev_r)` and
   `(base_r, uo_r)`.
4. One `--proofs` export per variant and cell, sha256, our checker, and
   `provari check` if present.
5. Writes `perf.txt`, `results.json`, `env.json` and `decision.json` next to
   itself. `decision.json` is `provisional` unless the host is
   `c90face26537`/x86_64.

The plumbing was validated with `--no-perf --rounds 1` on the aarch64 session
host, which has no PMU, so that run has no instruction numbers.

## Phase B — decision (pre-registered)

| Lever | Rule | Branch |
| --- | --- | --- |
| `ev` | `core` median fall **≥ 0.31e9**, counters/mate identical in every run on all 3 cells, `.pvr` byte-identical on all 3, both checkers accept, differential `mismatches: 0` | **release** |
| `ev` | anything else | **acceptance**: `MD7` stands, no `wishlist3` |
| `uo` | report falls, `fall_pct`, spreads and correctness on all 3 cells | **owner decision**, no adoption here |

A measurement that cannot be read on the bench host makes the session stop.
A provisional number does not branch. The 0.31e9 rule is unchanged from plan
8, and the measured fall decides it, not the projection.

**Release** (execution session):

- `wishlist3.md` gets **`W10`, an evasion-aware `LegalityContext`**. When the
  board is in check by one slider with the kings apart, `legality_context`
  stores `between(king, checker)`. `legal(m)` returns `false` for a quiet
  non-king move outside it and for castling in check. In any other check it
  returns `false` for every quiet non-king move. Everything else goes to the
  oracle. The ask carries the measured `core` fall, the census, the semantics
  above, and a crate-side test against `legal_in_view` over in-check positions.
  The ask covers behaviour and cost only and is written from the crate's own
  structures; no code is copied from this repository.
- `decisions.md` gets a dated **`MD8`** that reopens `MD7` for this one lever.
  It records the measured fall, the census, and that adoption needs an
  owner-triggered release plan with the `MD6` same-session row. `MD7`'s text
  is not edited, and the `initiative.md` table gains an `MD8` line.

**Acceptance:** a dated note under `MD7`'s evidence in `report9.md` only, with
no new decision row.

**`uo` record** for both branches, in `report9.md` and `decision.json`. The
neutral reading is: a fall is *material* if it exceeds 3× the larger spread on
every cell and is ≥ 0.5%. The owner options are:

- (a) enable the feature in `Cargo.toml`, with a `PROVENANCE.md` note and a
  `cargo test --features unsafe-optimizations` gate upstream;
- (b) keep the default features.

AGENTS' "measured win, documented, guarded" rule is applied to a dependency
feature for the first time, so the owner makes this call.

## Phase C — artefacts and docs (~0.5 h)

`measurements/plan9/`: README, driver, patches, `perf.txt`, `results.json`,
`env.json`, `decision.json`, census. No binaries, `.pvr` or `perf.data`.

`initiative.md` gets `M14`, one status addition and one history line;
`AGENTS.md` gets a one-line state update. The tree is unchanged, so there is no
pin, no `PROVENANCE.md` change and no ledger row.

## Gates

- `make test` green before and after; no edit to `src/`, `Cargo.*`,
  `PROVENANCE.md` or `check.rs`.
- In the scratch trees, the differential, both checkers on 9 proofs, and
  identical counters.
- A checker disagreement, a mask rejection the oracle calls legal, or a
  non-EP rules mismatch is a veto. That veto ends `ev` regardless of the
  instruction count.

## Stop rules

- No re-tuning of `ev` after Phase A: one shape, one measurement.
- No bench geomean and no `MD6` claim: this plan screens a crate ask.
- Search, table, eval, tree, `.pvr` and checker are off-limits.

## Risks

- **Crate placement may cost more than the prototype.** The prototype reuses
  `quiet()`'s `occupied.contains(to)`, which the crate must recompute. W10
  states the prototype's fall as its target, not as a promise.
- **Atomic semantics.** Captures, king moves and EP always go to the oracle;
  only quiet non-king moves and castling are short-circuited.
- **Overdue kill review.** `movegen` has no kill review after plans 4, 6 or 8.
  The owner should schedule one before any release plan.

## Next

- **Release:** plan 10, owner-triggered: upstream `W10`, then pin,
  `PROVENANCE.md`, differential, gates, veto and a same-session `MD6` row.
- **Acceptance:** `M8`, shakmaty removal, as in report 8 option 1.

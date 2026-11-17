# Plan 3 — P2: parity measurement and gap closure

**Item:** 3 (P2 parity gate). **Status:** proposed. **Session type:** execution.
**Predecessor:** [`plan2.md`](plan2.md). **Successor:** the `movegen`
initiative, after `P5` is recorded.

## Goal

Close the remaining wall gap to provari and reach `P4`/`P5`: all three bench
cells proved at 10 s, geomean wall `<= 1.0` against `provari --guide hand`,
every claim accepted by our checker and `provari check`. If the port is
already at parity after plan2, this plan is a measurement and ledger plan, not
a tuning plan.

## Context

- Read [`initiative.md`](initiative.md), [`decisions.md`](decisions.md),
  [`gotchas.md`](gotchas.md), `report2.md` and the plan2 measurements.
- The reference is re-measured in the same session. The bench set, metric and
  reading rules are in [`../../bench.md`](../../bench.md).
- The candidate is the plan2 binary. No new search formulation, no
  tablebases, no NNUE/model, no web, no UCI, no portfolio (D7).
- atomi's measured levers are the first place to look if the port is slower:
  HOTPATH-V2 (tree-neutral guide closures), NULL-PIPE (null-reply probe
  pipelining), proof-retention share, and guide/EF-PN tuning. Port only what
  the measurement justifies, with provenance.

## Phases

### Phase 0 — same-session baseline and diagnosis

1. Re-measure the reference and the plan2 candidate on all three cells at
   10 s and 120 s, seeds 0–2, three repeats, one process at a time.
2. Record wall, `child_evals`, `expansions`, proof length, reference `mate`
   and D14 verdicts.
3. Compute the per-node and tree terms separately. Do not quote a tree ratio
   from a different proof length without saying so.
4. Identify the dominant gap: tree size, per-node cost, proof-table pressure,
   or a cell-specific guide effect.

**Stop rule:** if the candidate already reaches `P5`, skip to Phase C.

### Phase A — port measured levers (only if Phase 0 justifies it)

Candidate levers, in order of expected value from atomi's record:

1. **HOTPATH-V2** — tree-neutral guide closures: existence-based one-ply
   explosion test, null board without a FEN round-trip, null reply test on the
   legal stream. Gate: tree identical, wall reduction on the product stack.
2. **NULL-PIPE** — `null_win` pass 1 keyed, prefetched and probed in chunks.
   Gate: exact by construction, tree identical, wall reduction.
3. **Proof-retention share** — if the proof table refuses a deep proof, test
   a larger share of the memory label. Gate: all three cells claimed, D14
   accepted, no wall regression on the easy cells.
4. **Guide/EF-PN tuning** — only with a held-out set and a pre-registered
   gate. A single-cell win is not a default.

Each lever is a named stack with one gate. A failed gate closes the lever
with the evidence; it does not become a default. A mechanism becomes a
default only on a measured wall-time win on the product `--guide hand` stack.

### Phase B — 10 s matrix and D14

1. Run the three cells at 10 s/192 MB, seeds 0–2, three repeats, same session
   as the reference.
2. For every claimed win, run the untimed proof export and check it with our
   checker and `provari check`. The timed and untimed counters must match.
3. Record the ledger row: cap, coverage, geomean, proof lengths, pins.
4. If a cell is censored, record `>= cap / reference`; do not convert it.

### Phase C — close and hand off

1. If `P5` is reached, record it in the initiative and the ledger, run the
   full gates, and write the closure block.
2. If `P5` is not reached, record the absolute distance and take the
   continue/re-scope/close decision at the kill review. Do not silently raise
   a cap or drop a cell.
3. If `P5` is reached, the next initiative is `movegen`; do not start it in
   this plan.

## Gates

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` green.
- `make test`, `make test-full` and `make veto` green.
- `P4`: all three cells proved at 10 s, 3/3 seeds, every proof accepted by
  both checkers.
- `P5`: `P4` plus geomean wall `<= 1.0` against the same-session
  `provari --guide hand` reference.
- The ledger row records `(cap, coverage, geomean)` together and the proof
  lengths beside the reference `mate`.
- No default change without a measured wall-time win on the product stack.

## Deliverables

- Same-session baseline and diagnosis.
- Any adopted lever with its gate and provenance.
- `measurements/plan3/` with parsed results and `env.json`.
- A ledger row if a claim is made.
- `report3.md` with the rung reached, the absolute distance, and the next
  step.
- Updated initiative/index status.

## Risks and stop rules

- If a lever's gate fails, close it; do not keep tuning it in the same plan.
- If the reference is unavailable, the result is `INCONCLUSIVE`; do not
  substitute a different version.
- If a proof is rejected, stop and fix correctness before any speed work.
- If the gap is dominated by a missing mechanism that is out of scope (e.g.
  tablebases or a new search formulation), take it to the kill review rather
  than adding it silently.

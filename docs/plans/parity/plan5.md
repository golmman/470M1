# Plan 5 — Movegen handoff

**Item:** 5 (Movegen handoff). **Status:** proposed; execution needs the
owner's go. **Session type:** execution.
**Predecessor:** [`plan4.md`](plan4.md), [`report4.md`](report4.md),
[`kill-review1.md`](kill-review1.md). **Successor:** kill review 2, then
`movegen` plan 1.

## Goal

Close `parity` with a recorded `P5` closure row and open `movegen` with a
documented rules surface and a pinned dependency. This is a handoff, not a
tuning plan: no search, table, proof, checker or rules change, and no new
dependency in this plan.

**Precondition (D11).** The owner approves closing `parity`. If not, execute
only Phase 0 and report; do not close the initiative or open `movegen`.

## Context

- `P5` is met on the product default (seed 0). Plan4's same-session 10 s
  seed-0 factors are 0.965 / 1.012 / 0.931, geomean **0.965**, coverage 3/3,
  9/9 accepted by both checkers, `D14` counters identical. The three-seed row
  (1.016) is a regression row, not the product row.
- Kill review 1: close `parity`, open `movegen`, spend no plan on margin.
- `movegen` is a placeholder initiative with candidate items M1–M7 and
  prerequisites: `P5` recorded, the prover's rules surface documented, a
  pinned `atomic-movegen`, and a differential harness.
- D11: `movegen` starts only after `P5` is recorded; the checker stays on
  shakmaty and shares no code with the prover.
- Read [`initiative.md`](initiative.md), [`decisions.md`](decisions.md),
  [`gotchas.md`](gotchas.md), [`report4.md`](report4.md),
  [`kill-review1.md`](kill-review1.md),
  [`../movegen/initiative.md`](../movegen/initiative.md) and
  [`../../bench.md`](../../bench.md).

## Phases

### Phase 0 — read-only audit

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`, `make test`, `make test-full`, `make veto`, `cargo doc`.
2. Verify the `P5` evidence: plan4's seed-0 rows, 3/3 coverage, 9/9 accepted,
   `D14` identical; confirm no open `parity` correctness item.
3. Inventory the prover's shakmaty rules surface, naming each call site and
   the contract `movegen` must preserve:
   - FEN parsing and Lichess-style acceptance (`lib.rs::position`,
     `from_setup`, `flaws`);
   - identity and hash: board, side, castling, legal en passant, clocks
     excluded (`lib.rs::hash`, `board.rs::hash`, `ep_hash`, `full_hash`);
   - 16-bit move representation, castling king-to-rook, en passant,
     promotions (`board.rs::Move`);
   - legal move generation, ordering and effects (`board.rs::pseudo_legal`,
     `legal`, `Effect`; `moves.rs::Mover`);
   - terminality: exploded/mated king, stalemate, insufficient material,
     50-move and repetition (`moves.rs::has_legal_move`, `format.rs::judge`,
     `dfpn.rs`);
   - check/legality: touching kings, pins, explosion (`moves.rs`);
   - proof/checker contract: compact move order and the shakmaty-only
     checker (`format.rs`, `check.rs`).
4. Check `golmman/atomic_movegen`: current HEAD, licence (MIT), API coverage
   for the contracts above, and whether it can be a GPL-compatible
   dependency. Choose a candidate pin; do not add it to `Cargo.toml` yet.
5. Confirm the checker-independence test still passes and `check.rs` imports
   no prover module.

**Stop rule:** no source change before the audit is recorded.

### Phase A — `P5` closure row

1. Copy plan4's driver to `measurements/plan5/run_bench.py` unchanged apart
   from its header. Run the product default (seed 0) at 10 s / 192 MB, three
   cells, three repeats, same session as the reference, one process at a
   time, with `--d14`.
2. For every claim, export the proof and check it with `470m1 check` and
   `provari check`; require timed and untimed counters identical.
3. Append one ledger row: `plan5`, product default seed 0, cap 10 s,
   coverage 3/3, geomean, proof lengths beside the same-session reference
   `mate`, `child_evals`, pins. Mark it **`P5` closure, product default**.
   Do not rewrite plan3's or plan4's rows.
4. If the geomean is `> 1.0` or a cell is censored, do **not** claim `P5`:
   record the result and stop for the owner/kill review. Do not raise the cap,
   drop a cell, or re-run until a better number appears.

**Stop rule:** no closure claim without both checkers and `D14`.

### Phase B — close `parity`

1. Update [`initiative.md`](initiative.md): status closed, item 5 done, final
   `P5` row, no active items; keep the goal and rungs as history.
2. Update [`README.md`](README.md), [`history.md`](history.md),
   [`../README.md`](../README.md), `AGENTS.md` current state and `README.md`
   if needed.
3. Record the closure pin as a placeholder for the owner's commit; never
   invent a sha.
4. No code change, no new dependency, no copied-material provenance row.

### Phase C — open `movegen`

1. Update [`../movegen/initiative.md`](../movegen/initiative.md): status open,
   prerequisites met or explicitly remaining, first plan next.
2. Write a concise rules-surface table (<= 4 KB) at
   `docs/plans/movegen/rules-surface.md` from Phase 0. It names each shakmaty
   call site, the invariant, and whether it is prover-only or checker-only.
3. Record the chosen `atomic-movegen` pin and licence in `PROVENANCE.md`
   source pins with use "movegen dependency (planned)"; do not add the
   dependency to `Cargo.toml` in this plan.
4. Do not write `movegen` code or `movegen/plan1.md` here. The next session
   writes plan 1 from the handoff brief; its first phase is the differential
   harness.
5. Write `report5.md` with tools, problems, open parts, missing tests, the
   handoff brief and the smallest next experiment.

## Gates

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `cargo doc --no-deps` green.
- `make test`, `make test-full` and `make veto` green.
- `P5` closure row: 3/3 coverage, geomean `<= 1.0`, 9/9 accepted by both
  checkers, `D14` identical.
- No source change to search, table, proof, checker or rules; no new
  dependency; checker independence test green.
- `parity` closed and `movegen` open with a rules surface and a candidate pin.
- No `movegen` code and no `movegen/plan1.md`.

## Deliverables

- `measurements/plan5/` with the driver copy, `env.json`, `results.json` and
  a README.
- One ledger closure row at 10 s, marked product default.
- `docs/plans/movegen/rules-surface.md` and the candidate pin record.
- Updated status docs and `report5.md`.
- No code, no dependency, no `movegen` plan.

## Risks and stop rules

- **Host drift.** If the seed-0 row reads `> 1.0`, record it and stop; do not
  re-run for a better number. The closure decision goes to the owner/kill
  review.
- **Oracle unavailable.** The result is `INCONCLUSIVE`; never substitute or
  skip the external check.
- **Open correctness item.** If the audit finds one, stop and fix it before
  closing `parity`.
- **Unsuitable movegen.** If `atomic-movegen`'s licence or API coverage is
  inadequate, record it and do not open `movegen`; the owner decides.
- **Docs budget.** If docs outgrow the budget, cut docs, not tests.

## Next steps after this plan

1. Owner commits the plan5 tree and records the pin in `docs/bench.md`.
2. Run kill review 2 (fixed prompt) over plans 4–5.
3. If the review agrees, write `docs/plans/movegen/plan1.md`; its first phase
   is the differential harness, not a prover rewrite.

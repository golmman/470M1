# Report 5 — `P5` closure and movegen handoff

**Plan:** [`plan5.md`](plan5.md). **Item:** 5. **Date:** 2026-11-17.
**Outcome:** all gates green. The product default (seed 0) reads geomean
**0.945** at 10 s, coverage 3/3, 9/9 proofs accepted by both checkers, `D14`
counters identical. `parity` is closed; `movegen` is open with a documented
rules surface and a pinned candidate dependency. No search, table, proof,
checker, rules, proof-format or dependency change. The execution request for
this plan is taken as the owner's D11 go.

## Phase 0 — read-only audit

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | 51 fast tests green (44 lib, 2 checker, 3 CLI, 2 prover) |
| `make test` | green (lint, licence, fast tests) |
| `make test-full` | green; ignored 10 s bench-cell test 1/1, veto 9/9 + 1/1 |
| `make veto` | 9/9 accepted, 1/1 control rejected by both checkers |
| `cargo doc --no-deps` | green |
| Checker independence | `check.rs` imports no crate module; its bottom test still reads the source |

`P5` evidence was re-read from plan4: seed 0 factors 0.965 / 1.012 / 0.931,
geomean 0.965; 3/3 coverage; 9/9 accepted; `D14` identical. No open `parity`
correctness item was found.

The prover's shakmaty surface was inventoried by call site (parsing, identity
and hash, 16-bit moves, generation/effects, check/legality, terminality,
proof order, checker). The resulting table is
[`../movegen/rules-surface.md`](../movegen/rules-surface.md), kept under 4 KB.
The dominant contracts for the handoff are:

- shakmaty's Lichess-style FEN acceptance stays the parser authority;
- identity is board, side, castling, legal en passant only, clocks excluded;
- the compact proof's move order is part of the format;
- **the checker's insufficient-material rule is authoritative**. shakmaty's
  atomic rule is broader than `atomic-movegen`'s deliberate no-table rule
  (K+B/K+N/K+R vs K can win there), so a movegen replacement must keep the
  prover's `moves.rs::cannot_win` rule explicitly.

`golmman/atomic_movegen` was read at HEAD
`6e04488fd737af17b2edbe101038f3f69754f3c6` (v2.8.0, MIT). It covers FEN and
`Setup`, stable 16-bit moves, legal generation, per-position `EffectContext`,
legal-en-passant hashing, outcomes/terminality, checkers and pins. It is a
zero-dependency, GPL-compatible candidate and is now recorded in
`PROVENANCE.md` as "movegen dependency (planned)"; it was not added to
`Cargo.toml`.

## Phase A — `P5` closure row

Driver: plan4's `run_bench.py` copied to
`measurements/plan5/run_bench.py`, header changed only. One interleaved
same-session pass, protocol a, seed 0, three repeats, 10 s / 192 MB, `--d14`.

| Cell | Ref wall | Cand wall | Factor | Ref mate | Cand mate | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.358 s | 0.335 s | 0.936 | 17 | 17 | 3/3 |
| `easy-win` | 0.078 s | 0.075 s | 0.962 | 15 | 15 | 3/3 |
| `deep-win` | 4.584 s | 4.301 s | 0.938 | 22 | 22 | 3/3 |

**Coverage 3/3, geomean 0.945, cap 10 s.** Every candidate proof was accepted
by `470m1 check` and `provari check` (9/9). `D14` timed and untimed counters
are identical for all three cells:

| Cell | Mate | Nodes | Child evals | Expansions |
| --- | ---: | ---: | ---: | ---: |
| `core` | 17 | 342,289 | 3,823,765 | 189,605 |
| `easy-win` | 15 | 72,523 | 897,937 | 52,251 |
| `deep-win` | 22 | 4,574,912 | 58,229,418 | 3,198,022 |

The row is appended to [`../../bench.md`](../../bench.md) as **`P5` closure,
product default**. Plan3's and plan4's rows were not rewritten. Evidence:
[`measurements/plan5/`](measurements/plan5/). The host is the same
`8f5ca2f2750a`; the candidate sha256 is unchanged from plan4.

## Phase B — `parity` closed

Updated [`initiative.md`](initiative.md) (status closed, item 5 done, no active
items), [`README.md`](README.md), [`history.md`](history.md),
[`../README.md`](../README.md), `AGENTS.md` and the root `README.md`. The
closure pin is left as `470m1@<sha>` / "owner commits the plan5 tree"; no sha
was invented. No code, dependency or provenance copy row changed.

## Phase C — `movegen` open

Updated [`../movegen/initiative.md`](../movegen/initiative.md): status open,
prerequisites met except the differential harness, first plan next. Wrote
[`../movegen/rules-surface.md`](../movegen/rules-surface.md). Added the
`atomic-movegen` source pin to `PROVENANCE.md`. No `movegen` code and no
`movegen/plan1.md` were written.

## Gates after the change

| Gate | Result |
| --- | --- |
| `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | green (no source change) |
| `cargo test`, `make test`, `make test-full`, `make veto` | green |
| `cargo doc --no-deps` | green |
| `P5` closure | 3/3, geomean 0.945, 9/9 both checkers, `D14` identical |
| Checker independence | green; `check.rs` has no prover import |
| No source/dependency change | yes; no `Cargo.toml`/`Cargo.lock` change |
| Handoff | `parity` closed, `movegen` open, rules surface + candidate pin |

## Problems and findings

- **`atomic-movegen`'s material verdict differs by design.** It has no
  insufficient-material table; shakmaty (and the checker) do. This is not a
  movegen bug, but a prover-side rule that plan 1 must preserve and the
  differential harness must exempt explicitly.
- **FEN acceptance is not yet equivalent.** `atomic-movegen` has opt-in
  validation; `parity` uses shakmaty's Lichess-style drops. Plan 1 should keep
  shakmaty for parsing or prove the acceptance sets equal, rather than assume
  it.
- **Proof move order is a second contract.** `atomic-movegen` has its own
  generation order, which differs from the compact proof's order. A movegen
  adapter must not leak that order into the writer or checker.
- **The 10 s instrument is still drift-limited.** This session's 0.945 is
  below plan4's 0.965 seed-0 reading on the same host; the code and counters
  are unchanged, so this is host drift, not a speed effect.

## Missing tests / open parts

- No differential harness yet: fixtures, corpus and random play-outs comparing
  legal moves, outcomes, hashes, en passant and child terminality.
- No proof-level A/B test that the same search tree under both rules
  implementations emits a proof accepted by both checkers.
- No `atomic-movegen` dependency is present, so the licence gate has not yet
  seen it in `Cargo.toml`.
- No automated check that the rules-surface document stays in step with the
  call sites; plan 1 should add the harness before the adapter.

## Handoff brief for `movegen` plan 1

1. Add `atomic-movegen` at `6e04488` (v2.8.0) and run the licence gate.
2. Build the differential harness first: each fixture, each corpus FEN and
   seeded random play-outs; compare legal moves as UCI sets, outcomes,
   terminal reasons, legal-en-passant hashes, and child keys. Treat the
   insufficient-material difference as an explicit expected divergence.
3. Only then add the adapter behind the prover's `Position`/`Mover`
   interface; keep `format.rs`, `check.rs` and the checker's shakmaty path
   unchanged.
4. Gate on zero unexplained disagreements, `make test-full`, both checkers on
   the fixtures, and a same-session 10 s bench row. A regression is a finding,
   not a silent adoption.

## Smallest next experiment

Run the differential harness on the nine veto fixtures and the three bench
FENs only, comparing legal move sets, outcome, legal-en-passant hash and child
key against shakmaty. Expected: zero differences except the known
insufficient-material class; cost ~1 h, no prover change.

## Next steps

1. Owner commits the plan5 tree and records the pin in `docs/bench.md`.
2. Run kill review 2 (fixed prompt) over plans 4–5.
3. If the review agrees, write `docs/plans/movegen/plan1.md`; its first phase
   is the differential harness, not a prover rewrite.

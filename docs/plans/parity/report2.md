# Report 2 — P1 search core

**Plan:** [`plan2.md`](plan2.md). **Item:** 2. **Date:** 2026-11-17.
**Outcome:** gates green; `P3` at 120 s; 10 s candidate matrix geomean 0.968;
one ledger row at 120 s.

## What landed

- `src/table.rs` — 4-slot clusters, filter, least-work replacement,
  `Open`/`Refuted`/`Mate`; single-threaded.
- `src/proofs.rs` — full-key proof table, `(move, length, limit)`, never
  evicts, refuses at nine tenths full.
- `src/dfpn.rs` — DF-PN with 1+ε, per-frame child numbers, move classes,
  EF-PN hand scaling, quiet grouping, simulation, null-threat replay and
  history. Tablebase, network, provability-model, progress and sampling code
  removed.
- `src/search.rs` — exhaustive win-in-N baseline and `read_proof`, tablebase
  removed.
- `src/eval.rs` — hand evaluation; `src/clock.rs` — the search clock.
- `prove FEN [--seconds S] [--memory MB] [--guide hand|none] [--proofs PATH]
  [--json] [--seed N] [--evals N]`. `--evals` is a deterministic child-eval
  limit; `--proofs` writes the compact proof after the search, so collection
  is trajectory-neutral.
- `tests/prover.rs` — every known-win fixture is proved and its proof accepted
  by the independent checker; every never-prove negative is refused.
- `tests/fixtures/proofs/deep-win.pvr` — written by this prover, accepted by
  both checkers.
- `docs/plans/parity/measurements/plan2/` — driver, parsed 120 s and 10 s
  results, `env.json`, `env-10s.json`, README.

## Scope change: provari `v0.1.0` search modules

The plan named provari `v0.4.0` (`bceb1ee`). Its `dfpn.rs`, `table.rs` and
`proofs.rs` are entangled with threads, the network, the provability model and
tablebases; stripping them in one sitting would have rewritten most of the
search. The same repository's `v0.1.0` has the same single-threaded DF-PN,
table and proof-table architecture without those layers. I copied `v0.1.0`'s
`table.rs`, `proofs.rs`, `dfpn.rs`, `search.rs`, `eval.rs` and `clock.rs`,
stripped the remaining tablebase/model/network/progress/sampling code, and
recorded the new pin in `PROVENANCE.md`. The search reproduces provari
`v0.4.0`'s node counts exactly on all three bench cells (`core` 342,289,
`easy-win` 72,523, `deep-win` 4,574,912), so the older single-threaded core is
behaviourally the same search for this plan. Plan3 may port the `v0.4.0`
ordered proof table if proof-table pressure shows up.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | 44 tests green (40 unit, 4 integration) |
| `make test` | green, including the licence gate |
| `make test-full` | green |
| `make veto` | 9/9 proofs accepted by both checkers; 1/1 control rejected by both |
| Known-win fixtures | all proved; every proof accepted by our checker |
| Never-prove negatives | all refused |
| Timed vs untimed counters | same search; proof writing happens after `solve` and does not touch the search |
| `P1`/`P2` | reached; `P3` also reached at 120 s |
| `P4`/`P5` | candidate 10 s matrix: all three, 9/9, geomean 0.968; plan3 owns the gate |

## Measurements

Same session, host `8f5ca2f2750a`, oracle `provari 0.4.0` at `bceb1ee`.
Reference: `provari prove FEN --seconds 10 --memory 192 --guide hand --json`.
Candidate: `470m1 prove FEN --seconds CAP --memory 192 --guide hand --seed S
--proofs TMP --json`, seeds 0–2, three repeats.

### 120 s cap (plan2 row)

| Cell | Ref wall | Cand wall | Factor | Ref mate | Cand mate | Child evals | Expansions | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.350 s | 0.382 s | 1.091 | 17 | 17 | 3,823,775 | 189,606 | 9/9 |
| `easy-win` | 0.079 s | 0.078 s | 0.987 | 15 | 15 | 897,937 | 52,251 | 9/9 |
| `deep-win` | 4.584 s | 4.065 s | 0.887 | 22 | 19 | 38,205,943 | 2,098,415 | 9/9 |

Geomean 0.985, coverage 3/3. Ledger row appended at 120 s.

### 10 s cap (plan3 evidence)

| Cell | Ref wall | Cand wall | Factor | Ref mate | Cand mate | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.365 s | 0.383 s | 1.049 | 17 | 17 | 9/9 |
| `easy-win` | 0.080 s | 0.078 s | 0.975 | 15 | 15 | 9/9 |
| `deep-win` | 4.902 s | 4.349 s | 0.887 | 22 | 22 | 9/9 |

Geomean 0.968, coverage 3/3. This is a candidate `P5` matrix, not the plan2
gate; plan3 must re-measure and decide.

## Problems and findings

- **Memory split depends on the cap.** `proofs_needed(seconds)` gives the
  proof table more room at 120 s than at 10 s. That is why `deep-win` finds a
  19-move proof at 120 s and a 22-move proof at 10 s. The 10 s row is the
  like-for-like parity comparison; the 120 s row is the plan's gate.
- **`refuted` differs on terminal draws.** Our `prove` reports
  `refuted: true` for a root that is already a draw (two kings, insufficient
  material); provari reports `found: false` without `refuted`. Both refuse the
  win. This is an output-shape difference, not a soundness difference.
- **No `--attacker`.** The fixtures and bench cells are all side-to-move wins,
  so the option was left out as the plan allows.
- **`--evals` is a child-eval limit.** It stops the search deterministically
  after N `evaluate` calls; it is not a node limit.

## Differential against provari

Small set, same FEN and hand guide:

- `core`, `easy-win`, `deep-win`: both find a win; our proof lengths 17/15/22
  at 10 s match the reference mates 17/15/22; both checkers accept.
- `veto.tsv` prove/rule50 rows: all found; every proof accepted by both
  checkers.
- `never_prove.tsv` subset (`only_two_kings_*`, `rook_alone_vs_safe_king`,
  `mate_in_1_black`, `dec24`, `dec28`, `dec36`): both refuse the win. Our
  `refuted` flag is set on the terminal draws where provari leaves it unset.

## Missing tests / open parts

- No slow fuzz over a stored proof corpus.
- No test that the `v0.4.0` ordered proof table is faster under pressure; the
  `v0.1.0` table is the one shipped.
- No `--attacker` path; no JSONL proof writer in `prove` (compact only).
- The 10 s matrix is evidence, not a gate; plan3 must re-measure it.

## Next steps

1. Execute [`plan3.md`](plan3.md): same-session 10 s matrix, per-node/tree
   diagnosis, D14, and the `P4`/`P5` decision.
2. If proof-table pressure appears, port the `v0.4.0` ordered proof table.
3. Keep the checker on shakmaty and independent; do not add tablebases.

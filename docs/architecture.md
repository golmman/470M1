# Architecture

How `470MI` is built and how it must grow. Rules for writing the code are in
[`guidelines.md`](guidelines.md); the decisions behind this shape are in
[`decisions.md`](decisions.md). `as-built` describes the tree today,
`target` is binding for new code and is reached by `foundation`.

## Principles

1. **Two independent rules implementations.** The prover runs on
   `atomic-movegen`; the checker runs on shakmaty and shares no code with the
   prover (`D4`). A rules bug cannot hide in both.
2. **Layers depend downward only.** Rules <- search <- proof output <- store
   <- application. No cycles, no upward calls; an upper layer reaches a lower
   one through a narrow interface.
3. **Policy at the edge.** Only the application layer parses flags, reads
   config files and the environment, prints, writes files and picks exit
   codes. Libraries return values and typed errors.
4. **The hot path is closed.** Node expansion uses static dispatch, no
   allocation per node, no logging and no I/O; it calls `atomic-movegen`
   directly (`MD5`). Seams sit around the hot path, not inside it.
5. **Determinism at one thread** (`PD2`). Observers, proof collection and
   configuration never change the trajectory.

## Layers (target)

| Layer | Responsibility | Modules | May depend on |
| --- | --- | --- | --- |
| rules | position, moves, effects, keys, terminality | `board`, `types`, `moves`, status/ending | `atomic-movegen` |
| search | proof-number search, tables, guides, budgets, progress events | `dfpn`, `table`, `proofs`, `eval`/`guide`, `clock` | rules |
| proof | proof walk, `.pvr`/JSONL encoding, line display | `walk`, `format`, `lines` | rules, `ProofSource` |
| store | durable proof database (`proof-db`) | `store` | rules, proof |
| app | CLI, config, logging, errors, exit codes, bench driver | `main`, `cli`, `config`, `log`, `error`, `bench` | all of the above, checker |
| checker | independent verification | `check` | shakmaty, serde_json, zstd |

**Crate boundary (target).** A Cargo workspace makes the boundaries
compile-time facts: `atomic470mi` (prover library: rules, search, proof,
store), `atomic470mi-check` (checker library, shakmaty only) and the `470MI`
binary (app), which depends on both. The source-reading independence test
stays as a second guard.

## Seams

Each seam is a small trait or enum owned by the lower layer and implemented
or wired from above. Hot-path seams are generic parameters (monomorphised),
never `dyn`.

| Seam | Purpose | Shape |
| --- | --- | --- |
| `Guide` | move ordering and initial numbers: none, hand, model | trait, search generic over it; replaces `dfpn::Guide` branches |
| `Budget` | when to stop: deadline, evaluation limit, external stop | value checked at the existing clock ticks; shared stop flag for threads |
| `Observer` | progress events for logs | called only at clock ticks with a `Progress` snapshot; search never formats text |
| `ProofSource` | winning move per position for encoding | trait over the proof table, the store or a file (`format::encode` already takes a closure) |
| `ProofTable` | proven entries `(key, limit) -> (move, length, limit)` | single-threaded today; shared variant for `parallel`; persistent backing for `proof-db` |
| `Config` | typed, validated settings per command | built by app (`PD9`), passed by value; no globals |
| errors | typed failure per layer | enum per layer; app maps to exit codes (`PD10`) |

## Data flow — `prove`

```
flags + config file -> ProveConfig -> Position (rules)
  -> Dfpn<G: Guide>(Table, ProofTable, Budget, Observer)   [search]
  -> result -> stdout ; Progress -> Observer -> log -> stderr
  -> ProofSource -> format::encode -> .pvr                  [proof]
  -> claim only after check (ours) and provari check accept [app, checker]
```

## Growth paths

- **`guided`**: a model guide is a new `Guide` implementation plus inference
  code in the search layer; `--guide hand` stays byte-identical.
- **`parallel`**: shared `Table`/`ProofTable` and a shared stop flag, after
  provari's shared-table design (GPL, provenance). The 1-thread path is the
  same code with a single worker and keeps identical counters.
- **`proof-db`**: `store` persists `ProofTable` entries (full keys, limits,
  lengths), appends with atomic commits, compacts, serves `ProofSource`
  queries, and exports `.pvr`. Warm runs are a separate measurement.
- **Openings (`nf3-d6`, `start`)**: long runs combine `parallel` and
  `proof-db`: resumable search seeded from the store, periodic checkpoints,
  progress logs, and proofs exported per subtree and checked.

## As-built map (2026-11-28)

| Module | Size | Layer | Conformance |
| --- | ---: | --- | --- |
| `dfpn.rs` | 76 KB | search | too large: driver, numbers, hand guide, simulation, null replay, memory and mates in one file |
| `check.rs` | 57 KB | checker | too large; entry is `check_paths` + `CheckOptions`, no parsing or printing |
| `moves.rs` | 36 KB | rules | too large: `Mover`, explosion facts and material rules |
| `format.rs` | 28 KB | proof | large; walk and encoding mixed; `save` moved to `app::write` |
| `app/` | — | app | conforming: `cli` (clap), `values`, `config`, `error`, `write`, `commands/{prove,check,line,bench}` |
| `table.rs`, `proofs.rs`, `lines.rs`, `board.rs` | 17–18 KB | — | — |
| `main.rs` | ≤ 10 KB | app | entry only: parse, dispatch, print the error, exit |
| `eval.rs`, `search.rs`, `types.rs`, `lib.rs`, `walk.rs`, `clock.rs` | ≤ 10 KB | — | conforming; `search.rs` is a reference baseline, unused by the CLI |

Cross-cutting gaps: library `String` errors are retired — every layer has
its own error enum with `Display` and `std::error::Error` (plan 3). No
progress logging (provari's was removed at the copy). Eleven panic-like
sites remain outside tests, all `expect("invariant: ...")`: `board` 3,
`moves` 6, `check` 1, `app/commands/bench` 1. The twelfth was the
panicking `Dfpn::new`, which plan 3 removed; the old "~90" counted test
code.

**Indicative splits** (the `foundation` plans derive the final cut from the
code):

- `dfpn/`: `search` (solve, mid, children), `numbers` (values, combine,
  thresholds), `guide` (band, evaluate), `simulate` (simulation, null-threat
  replay), `memory` (lookup, store, filter, proofs, split), `mates`.
- `check/`: `compact`, `jsonl`, `rules` (shakmaty helpers), `walk`; argument
  parsing moves to app.
- `moves/`: `mover`, `explosion`, `material`.
- `main.rs` -> `cli`, `config`, `log`, `error`, `commands/{prove,check,line,bench}`.

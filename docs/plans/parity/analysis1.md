# Analysis 1 — prior-art map for `parity`

Read-only differential written before plan1. It records what the three source
repositories contain, what `parity` takes, and what it deliberately defers.
It is not a measurement record and makes no performance claim.

## provari `v0.4.0` (`bceb1ee`)

provari is the parity reference. Its crate is a library plus a CLI; the
browser build is a separate target.

| Module | Role | Parity use |
| --- | --- | --- |
| `lib.rs` | FEN parsing, Lichess-style position acceptance, `ending` | copy/adapt |
| `board.rs` | bitboard/mailbox position, 16-bit moves, `Effect` precomputation, hashes equal to shakmaty | copy/adapt |
| `moves.rs` | per-position pins, check squares, attack maps, legality and one-ply explosion facts | copy/adapt |
| `eval.rs` | hand-crafted atomic evaluation (material, targets, fuses, pressure, checkers) | copy/adapt |
| `table.rs` | bounded TT: 4-slot clusters, filter, least-work replacement, `Open`/`Refuted`/`Mate` | copy/adapt |
| `proofs.rs` | full-key proof table, `(move, length, limit)`, never evicts, refuses at load limit | copy/adapt |
| `dfpn.rs` | DF-PN, 1+ε, hand guide, move classes, EF-PN, simulation, null-threat replay, history, threads | copy/adapt; threads deferred |
| `search.rs` | exhaustive win-in-N baseline | optional; not needed for parity |
| `format.rs` | compact `.pvr` v1 writer/reader, JSONL | copy/adapt |
| `walk.rs` | proof walk, length/tree measurement | copy/adapt |
| `lines.rs` | `line` display and PGN tree | copy/adapt |
| `check.rs` | independent checker, shakmaty only, no prover code | copy/adapt |
| `text.rs` | human status/result text | copy/adapt as needed |
| `clock.rs` | native/wasm clock | native only |
| `tablebase.rs` | Syzygy verdicts | **excluded** |
| `nnue.rs`, `model.rs` | network and provability model | **excluded** |
| `uci.rs`, `web.rs` | UCI and browser | **excluded** |

### Search pipeline

1. Parse the root FEN with shakmaty; accept it as Lichess does.
2. Build a `Position` and a `Mover` for the side to move.
3. For each child, compute an `Effect` (hash, counter, occupancy, capture)
   without playing the move; build the child only to descend.
4. Score children with move classes, EF-PN hand evaluation, history and
   simulation; keep the attacker's quiet moves as one delayed group.
5. Descend with DF-PN thresholds; store `Open`/`Refuted`/`Mate` in the TT and
   every proof in the full-key proof table.
6. On a root proof, walk the proof table and write the compact `.pvr` file.
7. Check the file with the independent checker and with `provari check`.

### Proof and checker contract

- The compact format writes only the attacker's moves; the checker generates
  every defence.
- Move order is checks first, then from-square, to-square, promotion.
- The header carries root, attacker (only when not the side to move),
  tablebase limit (0 for none), length and position count.
- The checker uses shakmaty only and shares no code with the prover. This is
  the property that makes the external veto meaningful.

## atomi (`a4802e7`)

atomi is the previous from-scratch attacker-only prover. It reached `T_3`
(all three cells at 120 s) and closed its per-node route to `T_4` by
measurement. Its durable contributions to `parity` are process and measured
levers, not code that must be copied.

| Contribution | Use in `parity` |
| --- | --- |
| Initiative / decisions / gotchas / history / bench structure | adopted as the documentation model |
| Stack-level gating and both-direction ablations | adopted as the working agreement |
| Bench set, corpus, reading rules, ledger conventions | adopted in `docs/bench.md` |
| HOTPATH-V2 (tree-neutral guide closures) | candidate lever in plan3 |
| NULL-PIPE (null-reply probe pipelining) | candidate lever in plan3 |
| Proof-retention share | candidate lever in plan3 if the proof table refuses |
| Guide/EF-PN tuning and held-out gates | candidate lever in plan3 |
| Gotchas: `Open` saturation, proofs not in TT, lazy stalemate, en passant key | adopted in `gotchas.md` |

## atomic_solver (`14abcf2`)

atomic_solver is the legacy general solver. Its `winprover` initiative
measured the cost of porting provari levers into a general Win/Loss/Draw
solver and plateaued. `parity` does not start from that solver; it takes the
fixtures, bench driver and test discipline.

| Contribution | Use in `parity` |
| --- | --- |
| `corpus-v1`, decisive fixtures, never-prove negatives | copy/adapt in plan1/plan2 |
| Bench driver and parsed-result layout | copy/adapt in plan1 |
| Test tiers, slow-test convention, deterministic budget discipline | adopted in `AGENTS.md` |
| Proof-tree / proofdb machinery | **excluded**; proofs are files |
| General Win/Loss/Draw search | **excluded**; parity is attacker-only DF-PN |

## What parity takes, and when

| Mechanism | Source | Milestone | Note |
| --- | --- | --- | --- |
| FEN/rules/ending | provari | P0 | shakmaty authority |
| compact `.pvr` + JSONL | provari | P0 | external checker contract |
| independent checker | provari | P0 | shakmaty only |
| walk/line | provari | P0 | no tablebase branches |
| board/moves/effect | provari | P1 | differential against shakmaty |
| TT + proof table | provari | P1 | full-key proofs, no eviction |
| DF-PN + hand guide | provari | P1 | no threads, no model |
| proof writer | provari | P1 | fails closed |
| bench harness | atomic_solver/atomi | P0/P1 | reference side first |
| HOTPATH-V2 / NULL-PIPE / proof share | atomi | P2 | only if measurement justifies |
| `atomic-movegen` replacement | movegen initiative | after P5 | checker stays on shakmaty |

## Open questions for plan1/plan2

1. Does the compact format's zstd framing need the exact provari level and
   checksum for `provari check` to accept? (Plan1 tests this.)
2. Which provari modules can be copied verbatim after removing tablebase,
   NNUE/model, web and UCI, and which need adaptation? (Plan1 records the
   provenance rows.)
3. Does the checker's `valid: null` path for tablebase proofs need a clear
   message when the header says `tablebases > 0`? (Plan1 decides.)
4. Which atomi measured lever is the first candidate if plan2 is slower than
   provari? (Plan3 Phase 0 decides from the diagnosis, not from this list.)

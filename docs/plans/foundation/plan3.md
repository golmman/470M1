# Plan 3 — F2 (library): typed errors and the invariant audit

**Item:** F2 (library half). **Session:** plan → execution. **Status:** proposed
(2026-11-28). **Successor:** [`report3.md`](report3.md).

## Goal

Retire every library `Result<_, String>` — 50 return sites in seven files — into
typed error enums per layer, each with `Display` and `std::error::Error`; finish
F2's panic audit: outside tests every `unwrap`/`expect`/`panic!`/`unreachable!`
is an `expect("invariant: ...")` with a stated invariant, and the panicking
`Dfpn::new` goes. Keep every user-visible message, `valid: null` verdict, exit
code, counter and `.pvr` byte unchanged. **No search, proof, checker-rule or
default change.**

## Probes (read-only, 2026-11-28)

| Fact | Value |
| --- | --- |
| Base | `96c40da` (`finish foundation plan2`), clean tree |
| `Result<_, String>` | 50 lines in `board` 1, `check` 19, `format` 15, `lib` 1, `lines` 10, `search` 2, `walk` 2; `src/app/` is already typed |
| Panic-like, non-test | 12: `board` 3, `moves` 6, `check` 1, `dfpn` 1, `app/commands/bench` 1 (already an invariant). `architecture.md`'s "~90" is stale; report 3 corrects it |
| Null verdict | `check::verdict` decides `valid: null` by `error.starts_with("cannot check")`; a variant must replace the prefix test |
| Locked text | tests, `check` verdicts and `line` errors match current messages (`bad FEN`, `illegal position`, `the proof has no move for`, `a line repeats`, `cannot check`, ...) |
| Counters | plan 2 product = movegen plan 12: `core` 14 / 350,430 / 3,896,430 / 191,557; `easy-win` 18 / 68,692 / 904,368 / 53,237; `deep-win` 20 / 3,553,009 / 44,828,946 / 2,487,550 |

## Error model

One enum per layer; `String` may remain only as a leaf detail inside a typed
variant, never as a function's error type. `Display` reproduces today's text
byte for byte.

| Module | Type | Variants (groups) |
| --- | --- | --- |
| rules | `board::FenError` | `Syntax(FenLoadError)`; `Illegal(Vec<Flaw>)` with `Flaw` = empty board, missing/too many king, pawn on back rank, opposite check, impossible check, unreachable. `lib::position` delegates; `Position::from_fen` keeps its message |
| proof | `walk::Error` | `Repeat`, `NoMove` |
| proof | `format::Error` | `Read { path, source }`, `Json`, `Zstd`, `Header`, `Position`, `Move`, `Walk`, `Tablebases`, `Mismatch` |
| proof | `lines::Error` | `Walk`, `NoMove { fen }`, `Illegal { token }`, `NotProof`, `Over`, `EndsWithoutWin { fen }`, `NoWin` |
| search | `search::Error` | `RootOver`, `NoMove { fen }`; `read_proof`'s emit closure returns it too |
| checker | `check::Error` | entry only: `Options`, `Thread`, `Incomplete` |
| checker | `check::Rejection` | per proof: `Invalid`, `Unchecked`; `verdict` switches on this, not on text |
| app | `app::Error` | fields become typed (`Position(FenError)`, `Proof(format::Error)`, `Checker(check::Error)`, `Moves(lines::Error)`); `Display`/`source` forward, so text and exit codes stay |

The checker keeps its own enums and imports nothing from the prover; the
independence test stays. No new dependency (`thiserror`, `anyhow`).

## Invariant audit

| Site | Disposition |
| --- | --- |
| `board.rs` 99, 119, 335 | promotion role, proof-bit kind, legal move: `expect("invariant: ...")`; add a `proof_bits`/`from_proof_bits` round trip over legal moves |
| `moves.rs` 213, 240, 344, 375, 412, 440 | a legal move's from-square holds a piece: one invariant message |
| `check.rs` 405 | Atomic has no drops or null moves: `expect("invariant: ...")` |
| `dfpn.rs` 242 | delete `Dfpn::new`; the seven test call sites use `try_new(...).expect("the test's memory")` |
| `app/commands/bench.rs` 24 | already `invariant: config::bench requires out`; keep |

## Steps

1. Baseline: `git archive 96c40da | tar -x -C /tmp/470M1-base`, release build,
   sha256, guard counters.
2. `board::FenError` + `lib`/`format` callers; then `walk`, `format`, `lines`,
   `search`; compile and test after each module.
3. `check::Error` + `check::Rejection`; preserve every verdict key, message and
   null rule.
4. App conversions and `source()`; `tests/cli.rs`, `tests/config.rs` and
   `tests/checker.rs` unchanged except where they matched `String`.
5. Invariant audit and the source guard; remove the panicking constructor.
6. Docs: `architecture.md` as-built gap (String errors done; 12 panic sites,
   not ~90), `initiative.md`, `plans/README.md`, then
   [`report3.md`](report3.md) and `measurements/plan3/`.
7. Gates and measurement; no ledger row (a refactor guard row, as plan 2).

## Tests (pre-registered)

- **Display snapshots:** one test per enum variant prints its exact current
  text; a changed message fails.
- **Sources:** an I/O error keeps its `source()` chain through `format::Error`
  and `app::Error`.
- **Checker verdicts:** malformed proof → `valid:false`; tablebase proof and a
  too-many-positions proof → `valid:null`; bad worker/seconds options →
  `check::Error` (exit 3). Same keys, text and order.
- **Source guard** (`tests/errors.rs`, non-test prefix before
  `#[cfg(test)] mod tests`): no `Result<..., String>`; every
  `unwrap(`/`expect(`/`panic!`/`unreachable!` is an `expect("invariant: ...")`.
- **No-panic sweep and exit codes:** the plan 2 cases still pass; `line` on an
  invalid proof is 1, usage 2, runtime 3.
- **Neutrality:** counters and `.pvr` bytes as in the gates.

## Gates (pre-registered; correctness first)

1. **Correctness.** `make test-full` passes (fast and ignored tests,
   `unsafe-feature`, veto: 9/9 accepted, control rejected; differential 0/0).
   The guard `.pvr` are byte-identical to
   `measurements/plan2/guard/proofs/candidate/*.pvr` and accepted by both
   checkers. A disagreement is a veto.
2. **Counters and determinism.** The three guard cells give exactly the plan 2
   product counters above, across repeats, completed searches only (report 1's
   gate-2 finding).
3. **Tests.** `make test` and `make doc` are clean; the source guard passes; no
   `Result<_, String>` outside tests; every non-test panic-like site is an
   invariant `expect`. The report maps each enum and exit path to its tests.
4. **Measurement.** Guard set, `--baseline-bin` row, 5 interleaved repeats,
   10 s, 192 MB, 1 thread. **Both binaries are copied into one fresh directory
   on the same filesystem before the row** (plan 2's location finding); record
   both sha256s and that directory. Pass if cand/base geomean is `<= 1.01`;
   otherwise one same-session rerun decides, and a second miss is blocking.
   `core` cachegrind `Ir` `<= +0.5%` vs base, one run each, no `--seconds`.
   Diagnostics: mate-in-1 `Ir`, median of 20 startup walls.
5. **Scope.** Diffs are limited to error types and propagation, app error
   conversions, invariant messages, the `dfpn` constructor's removal, tests and
   docs. No change to search decisions, tables, proof encoding, checker rules,
   defaults, flags or output shapes; no `Cargo.toml` change.

## Non-goals and risks

Out of scope: F5 module splits, F4 logging, the F6 checker crate, F7 closure, a
new dependency, and any proof-format or checker-rule change.

- **Message drift** breaks drivers: Display snapshots and the CLI tests.
- **The checker could inherit a prover type:** its enums stay local; the
  independence test guards it.
- **A source guard can be brittle:** it reads only the non-test prefix and
  allows exactly the invariant form.
- **An error type can move inlining:** gate 4 measures `core`; counters pin the
  trajectory.
- **`Dfpn::new` is a public API break:** only in-repo tests call it; no
  external user.

**Session (~2.5 h):** baseline 15, rules/proof enums 70, checker 25, app 20,
tests and guard 35, measurement 15, report 10.

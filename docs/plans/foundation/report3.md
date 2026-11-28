# Report 3 — F2 (library): typed errors and the invariant audit

**Item:** F2 (library half). **Plan:** [`plan3.md`](plan3.md)
(2026-11-28). **Session:** execution, 2026-11-28. **Status:** complete,
owner commits.

## Goal and verdict

Retire every library `Result<_, String>` into a typed error enum per layer,
with `Display` and `std::error::Error`; finish the panic audit so every
non-test `unwrap`/`expect`/`panic!`/`unreachable!` is an
`expect("invariant: ...")`; remove the panicking `Dfpn::new`. Keep every
user-visible message, `valid: null` rule, exit code, counter and `.pvr` byte
unchanged.

**Met.** There is no `Result<_, String>` outside tests; the source guard
passes. Every one of the three guard cells has the plan 2 counters and a
byte-identical `.pvr`, both checkers accept all three (9/9 fixtures plus the
control in the veto), and the geomean is 0.9782 (gate `<= 1.01`) with `core`
`Ir` −0.000001% (gate `<= +0.5%`).

## What was built

| Layer | Type | Variants as built |
| --- | --- | --- |
| rules | `board::FenError` | `Syntax(FenLoadError)`, `Illegal(Vec<Flaw>)`; `Flaw` = empty board, missing/too many king (by colour), pawn on back rank, opposite check, impossible check, unreachable |
| rules | `board::FenError` delegation | `lib::position` returns it; `Position::from_fen` keeps its message |
| proof | `walk::Error` | `Repeat`, `NoMove` |
| proof | `format::Error` | `Read { path, source }`, `Json`, `Zstd`, `Header`, `Position`, `Move`, `Walk`, `Tablebases`, `Mismatch` |
| proof | `lines::Error` | `Walk`, `NoMove { fen }`, `Illegal { token }`, `NotProof { played, token }`, `Over`, `NoWin` |
| search | `search::Error` | `RootOver`, `NoMove { fen }`; `read_proof`'s emit closure returns it |
| checker | `check::Error` | entry only: `Options`, `Thread`, `Incomplete` |
| checker | `check::Rejection` | per proof: `Invalid`, `Unchecked` |
| app | `app::Error` | `Position(FenError)`, `Proof(format::Error)`, `Checker(check::Error)`, `Moves { played, source: lines::Error }`; `Display` and `source` forward |

The checker keeps its own enums and imports nothing from the prover; the
independence test passes. No new dependency.

Two deliberate shapes differ from the plan's table, both to keep text and
context exact:

- `lines::Error` has no `EndsWithoutWin`: no current `lines` message
  corresponds to it. The two "line ends without the attacker winning"
  messages are proof-walk messages and live in `format::Error::Walk`. `NoWin`
  is the current `"the proof ends without a win"`.
- `app::Error::Moves` carries `played` beside the `lines::Error`, because the
  user-visible prefix is `the moves "<played>": `; a bare tuple would have
  lost it.

`check::verdict` now matches `Rejection::Unchecked` instead of testing the
`"cannot check"` prefix, so every limit and tablebase message still produces
`valid: null` and every proof defect `valid: false`.

## Invariant audit

| Site | Result |
| --- | --- |
| `board.rs` promotion role, proof-bit kind, legal move | three `expect("invariant: ...")`; a `proof_bits`/`from_proof_bits` round trip was added to `moves_and_effects_agree` over every legal move |
| `moves.rs` six from-square lookups | one message, `invariant: a legal move's from-square holds a piece` |
| `check.rs` move number | `invariant: Atomic has no drops or null moves` |
| `dfpn.rs` `new` | removed; the six unit call sites and `tests/prover.rs` use `try_new(...).expect("the test's memory")` |
| `app/commands/bench.rs` | unchanged: `invariant: config::bench requires out` |

Eleven sites remain outside tests, all stated invariants. `architecture.md`
now records that and says the old "~90" counted test code.

To make the source guard honest, test-only helpers that sat in the non-test
prefix moved inside `#[cfg(test)]`: `format`'s `state`, `SMALL` and
`small_defended` (the test module is now `pub(crate)`), `eval`'s
`terms_reference`, and `proofs::Proofs::sized`. No production code moved.

## Tests

`tests/errors.rs` is new. It holds:

- Display snapshots for every enum variant and every `Flaw` (plus a two-flaw
  join), and the `Read` path format;
- an I/O `source()` chain through `format::Error::Read`;
- the checker split: `b"{}"` → `valid:false` and `"no root"`; a tablebase
  header and a header over `MOST_POSITIONS` → `valid:null` with the exact
  text; a real fixture → `valid:true`; the JSON key order in each case;
- `check_paths` with zero workers and with no proofs → `check::Error::Options`;
- `position` delegating to the rules-layer messages;
- the source guard: every `src/**/*.rs` non-test prefix has no
  `Result<..., String>` and no bare `unwrap(`, `panic!`, `unreachable!`; every
  `expect(` starts with `"invariant: "`.

`src/app/error.rs` gained two unit tests: the proof error forwards its I/O
source through `app::Error`, and the `the moves "<played>": ` prefix survives.

`tests/cli.rs`, `tests/config.rs` and `tests/checker.rs` needed no change.

## Gates

1. **Correctness — passed.** `make test-full` passed (fast and ignored tests,
   `unsafe-feature`, veto: 9/9 fixtures accepted, control rejected;
   `test-full.txt`); `rules_differential_full` asserts 0 mismatches, 0 key
   mismatches and 0 ep-threat differences. The three guard `.pvr` are byte-identical to
   `measurements/plan2/guard/proofs/candidate/*.pvr` (`guard/cmp.txt`) and
   accepted by `470MI check` and `provari check` (`guard/claims.json`). No
   disagreement.
2. **Counters and determinism — passed.** Every completed search in the row
   (5 base + 5 cand per cell) has identical `mate`/`nodes`/`child_evals`/
   `expansions`: `core` 14 / 350,430 / 3,896,430 / 191,557; `1eb80b92f92c`
   18 / 68,692 / 904,368 / 53,237; `db6a741c723f` 20 / 3,553,009 /
   44,828,946 / 2,487,550 — the plan 2 product. All searches completed.
3. **Tests — passed.** `make test` and `make doc` are clean; the source guard
   passes; no `Result<_, String>` outside tests; every non-test panic-like
   site is an invariant `expect`. The option-to-test map is below.
4. **Measurement — passed on the pre-registered row.** Guard, 5 interleaved
   repeats, 10 s, 192 MB, 1 thread, both binaries copied into
   `/tmp/470M1-plan3-bin` on one filesystem (plan 2's location finding):

   | Cell | base [s] | cand [s] | Factor |
   | --- | ---: | ---: | ---: |
   | `1eb80b92f92c` | 0.1254 | 0.1157 | 0.9226 |
   | `core` | 0.5128 | 0.5144 | 1.0031 |
   | `db6a741c723f` | 5.2058 | 5.2646 | 1.0113 |
   | **Geomean** | | | **0.9782** pass |

   `core` cachegrind `Ir`: base 5,980,229,410, cand 5,980,229,347,
   **−0.000001%** (gate `<= +0.5%`), same counters. Diagnostics: mate-in-1
   `Ir` −0.0005%; median of 20 startup walls 12.825 → 11.810 ms.
5. **Scope — passed.** No `Cargo.toml`/`Cargo.lock` change. Diffs are error
   types and propagation, app error conversions, invariant messages, the
   `Dfpn::new` removal, tests and docs. No search decision, table, proof
   encoding, checker rule, default, flag or output-shape change
   (`scope.txt`).

## Problems and findings

- **Message drift was caught before the row.** The first `board::FenError`
  printed `FenLoadError`'s `invalid FEN syntax: ` prefix, while the baseline
  printed only the inner `FEN too short: …`. A CLI parity sweep of the base
  and candidate binaries over malformed FENs and proofs found it; `Display`
  now unwraps the inner syntax error, and the sweep is clean over 120
  generated proof mutations (`check` and `line` stdout, stderr and exit).
- **The plan's "bad worker/seconds → exit 3" is not reachable through the
  CLI.** `clap`'s value parsers reject those as usage errors (exit 2), as they
  did in plan 2. `check_paths` itself now returns `check::Error::Options`, and
  the library test pins that. No exit code changed.
- **Cachegrind `Ir` shifted by ~1.7k from plan 2's recorded base** on the
  same bytes (5,980,229,410 vs 5,980,062,197) because the host is a different
  container; the within-row delta is what the gate measures and it is
  −0.000001%.
- **Plan 2's same-filesystem rule is still not enforced by the driver.** It
  was followed manually here; the driver accepts any `--baseline-bin`. This
  remains an open decision from plan 2.

## Option-to-test map

| Rule | Tests |
| --- | --- |
| every enum variant's Display | `errors.rs::display_snapshots_are_unchanged` |
| FEN delegation and flaw words | `errors.rs::display_snapshots_are_unchanged`, `position_delegates_to_the_rules_error`; `lib::tests::reads_fens_as_lichess_does` |
| I/O `source` chain | `errors.rs::an_io_error_keeps_its_source`, `app::error::tests::forwards_a_proof_error_and_its_source` |
| `Moves` prefix | `app::error::tests::keeps_the_moves_prefix` |
| checker false/null/true split, keys and order | `errors.rs::the_checker_keeps_its_verdict_split`; `check::tests::rejects_damaged_compact_proofs`, `keeps_to_its_limits` |
| checker entry errors | `errors.rs::bad_check_options_are_an_entry_error`, `check::tests::refuses_zero_workers` |
| no `String` errors, invariant expects | `errors.rs::the_source_guard_holds` |
| exit codes 0–3, no panic | `cli.rs::exit_codes_0_to_3`, `garbage_never_panics`, `a_closed_stdout_is_an_error_not_a_panic` |
| proof-bit round trip | `board::tests::moves_and_effects_agree` |
| `Dfpn::new` removal | `dfpn` unit tests and `tests/prover.rs` compile against `try_new` |

## Open parts and missing tests

- Progress logging (F4), module splits (F5), the checker crate (F6) and the
  closure audits (F7) are untouched.
- No test forces a checking-thread start or join failure; the typed
  `check::Error::Thread` path is exercised only by construction.
- The source guard reads the non-test prefix at the first `#[cfg(test)]`; it
  trusts the convention that all later code is test code. Moving a production
  item below a test helper would hide it.
- No test drives `app::Error` through every CLI error path; the CLI tests
  cover the user-visible ones.

## Next steps

1. Owner commits the plan 3 tree.
2. Owner decides the same-filesystem rule for `--baseline-bin` rows (driver
   check, `bench/README.md` line), carried over from plan 2.
3. `F4` (logging): `Observer`/`Progress`, stderr logger, levels, and the
   trajectory-neutrality test.

Evidence: `measurements/plan3/` (`README.md` indexes it).

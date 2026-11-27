# Plan 2 — F3 + F2 (edge): configuration, exit codes, atomic writes

**Items:** F3 (Configuration) and the app half of F2 (exit codes, atomic
writes). **Session:** plan → execution. **Status:** proposed (2026-11-27).
**Successor:** [`report2.md`](report2.md).

## Goal

Replace the hand parsers in `main.rs` and `check::run` with `clap` + `serde`
+ `toml` (`PD9`): `--config FILE`, one typed validated config per command,
precedence default < file < flag, effective config printed with every result;
map errors to exit codes 0–3 in the binary only; make every write atomic. FR1
met, FR2 met at the edge. **No search, proof-byte, checker-rule or default
change.**

## Regrouping (scope note for `initiative.md`)

F2 splits: library error enums (`lib::position`, `format`, `lines`, `walk`,
`search`, `check`) and the `expect` invariant audit are **plan 3**; until then
the app wraps library `String`s in a typed variant per call site. `check`'s
argument parsing moves to the app here (from F6): clap owns every flag.

## Probes (read-only, 2026-11-27)

| Fact | Value |
| --- | --- |
| Base | `f7ba48a`; counters = movegen plan 12 (`child_evals` 3,896,430 / 904,368 / 44,828,946, mate 14/18/20) |
| Crates | `clap` 4.6.8 (2026-11-16), `toml` 1.2.1 (2026-11-06), both MIT OR Apache-2.0; `serde` 1.0.229 + `serde_derive` already locked |
| Instructions | `perf` has no `instructions:u`; `valgrind --tool=cachegrind --cache-sim=no`: `core` 5,980,060,924 `Ir` twice, same counters, ~13 s (not ledger units) |
| Writes | `format::save` atomic but in the library; `line --pgn` `fs::write`; `bench --out` streamed |
| Traps | `prove`'s default `--seconds` is **absent**, and `proofs_needed(seconds, …)` reads it; `println!` panics on a closed pipe; the driver reads the **last** JSON line of `check`; `line` exits 2 on an invalid proof |

## Design

**Modules** (binary crate, `mod app;` from `main.rs`; the library never sees
clap or toml): `app/cli.rs` (clap derive), `app/config.rs` (values, file,
resolve), `app/error.rs`, `app/write.rs`, `app/commands/{prove,check,line,bench}.rs`.
`main.rs` parses, dispatches, prints the error, exits. Files stay under 10 KB.

**Values.** `Seconds`, `Megabytes`, `Workers`, `Repeats`, `Evals`, `Until`,
`Guide`: each has one validator used by clap (`FromStr`) and serde
(`try_from`), with today's messages and ranges.

**Flags.** clap fields are `Option<T>` with no clap defaults (absent is
visible); `--config` is global; booleans only switch on.

**File.** Tables `[prove]`, `[check]`, `[line]`, `[bench]`; key = flag name with
`_` for `-`; `deny_unknown_fields`; no implicit file.
`tests/fixtures/config/defaults.toml` spells out every default.

**Defaults** live in each config's `Default` and equal today's:

| Command | Defaults |
| --- | --- |
| prove | seconds absent (unbounded), memory 192, guide hand, seed 0, evals absent, proofs absent, json false |
| check | workers 1, seconds absent, compact_only false |
| line | pgn absent, until absent |
| bench | out required, reference `$PROVARI` else `provari`, seconds 10, memory 192, guide hand (passed through), repeats 3 |

**Effective config.** A `config` object joins the `prove --json` line, every
`check` verdict line (no extra line: the driver reads the last) and `line`'s
result; `bench --out` starts with a `{"config":…}` row; text `prove` adds a
stdout line `config: key=value …`.

**Checker entry.** `check::run(args)` becomes `check::check_paths(&[PathBuf],
CheckOptions) -> Result<Vec<Value>, …>`: no parsing, no printing;
`CheckOptions` lives in `check.rs` (independence holds); no rule or verdict
change.

**Exit codes** (help text and `guidelines.md` table):

| Code | Cases |
| --- | --- |
| 0 | success, no win found |
| 1 | `check` saw a proof not shown valid (`null` included, unchanged); `line` on an invalid proof |
| 2 | clap errors, bad values, unreadable/bad/unknown-key config, bad FEN, an output path whose parent is missing or that is a directory (checked **before** the search) |
| 3 | I/O on inputs or outputs, failed memory reservation (`try_new`), `bench` reference spawn, checker thread spawn, a proof walk that fails after a win, a closed stdout (no panic) |

**Writes.** `app::write::atomic(path, bytes)`: unique temp file in the target
directory, sync, rename; removed on failure. Serves `--proofs`, `--pgn`,
`--out` (bench rows buffered); `format::save` leaves the library.

## Steps

1. Baseline: `git archive f7ba48a | tar -x -C /tmp/470M1-base`, release
   build, sha256, guard counters.
2. `cargo add clap --no-default-features -F std,derive,help,usage,error-context`
   and `cargo add toml`. Every new `Cargo.lock` entry must be at least 7 days
   old on crates.io (`cargo update --precise` otherwise) and pass
   `scripts/licence.sh`. List them in the report.
3. Implement the design. `tests/cli.rs` and a new `tests/config.rs`.
4. Docs: help text; the `guidelines.md` exit table (3 is no longer
   "`foundation`"); the `architecture.md` as-built map; the regrouping in
   `initiative.md`; `bench/README.md` (`--baseline-bin`).
5. Driver: `run_bench.py --side cand --baseline-bin OLD` interleaves
   `base,cand` per cell; factor cand/base.
6. Gates and measurements, then the report.

## Tests (pre-registered)

- **Per option:** a table-driven test per command: each flag and TOML key with
  a good value (seen in `config`) and a bad one (exit 2, naming flag or key).
- **Precedence:** per command, file 64 + flag 128 → 128; file only → 64;
  neither → default.
- **File errors:** unknown key or table, bad syntax (line named), missing
  file: exit 2.
- **Defaults snapshot:** the effective config with no flags and no file equals
  the table above.
- **Trajectory neutrality:** `prove CORE --evals 200000 --json` gives identical
  `nodes`/`child_evals`/`expansions`/`root` with no config, `defaults.toml`,
  and flags spelling the defaults.
- **Exit codes 0–3:** one case each, including `line` on a control (1), a
  missing `--proofs` parent (2, before the search), `bench --reference
  /nonexistent` (3), and a closed stdout (3, no `panicked`).
- **No panic sweep:** garbage FENs, proofs and TOML exit in {1,2,3} without
  `panicked`.
- **Atomic write units:** target replaced, no temp left, a failed write keeps
  the old bytes.

## Gates (pre-registered; correctness first)

1. **Correctness.** `make test-full` passes (veto, differential 0/0, slow bench
   cells). The new release binary's guard proofs are byte-identical to
   `measurements/plan1/guard/proofs/candidate/*.pvr` and accepted by both
   checkers. Any disagreement is a veto.
2. **Defaults and determinism.** Guard `mate`/`nodes`/`child_evals`/
   `expansions` are identical between the old and new binaries and across
   repeats. Only completed searches count (report1's gate-2 finding). The
   neutrality test passes.
3. **Tests.** `make test` is clean. The report maps each option to its tests.
   No new `Result<_, String>` in `src/app/`.
4. **Measurement.** Guard set, `--baseline-bin` row, 5 interleaved repeats, 10
   s, 192 MB, 1 thread. Pass if the cand/base geomean is `<= 1.01`. Otherwise
   one same-session rerun decides; a second miss is a blocking finding.
   `core` cachegrind `Ir` `<= +0.5%` vs base, one run each, no `--seconds`
   (a ~38x slowdown hits 10 s). Diagnostic: mate-in-1 `Ir` and
   median of 20 walls (startup; 20–55 ms matched cells).
5. **Scope.** No diff in `src/{dfpn,table,proofs,eval,moves,board,types,walk,
   search,clock}.rs`; `format.rs` only loses `save`; `check.rs` only its entry.

## Non-goals and risks

Out of scope: library error enums, the `expect` audit and module splits (plan
3, F5); logging flags `-q`/`-v`/`--log` (F4); `--threads`; new defaults; a
ledger row; any change to `check`'s verdict or `null` semantics.

- clap accepts other syntax (`--x=v`; extra positionals rejected). The
  driver, `veto.sh` and the `Makefile` use only documented forms; the gate-4
  row is their smoke run.
- A new binary can move code layout with identical counters: gate 4.

**Session (~3 h, min):** baseline and crates 20, app 80, tests 40, driver and
docs 20, gates 20, then [`report2.md`](report2.md) and the status line.

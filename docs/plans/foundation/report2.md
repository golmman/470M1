# Report 2 — F3 + F2 (edge): configuration, exit codes, atomic writes

**Items:** F3 and the app half of F2. **Plan:** [`plan2.md`](plan2.md)
(2026-11-27). **Session:** execution, 2026-11-27. **Status:** complete, owner
commits.

## Goal and verdict

Replace the hand parsers with `clap` + `serde` + `toml`, add `--config FILE`
with one typed, validated config per command (default < file < flag), print
the effective config with every result, map errors to exit codes 0–3 in the
binary only, make every write atomic. No search, proof-byte, checker-rule or
default change.

**Met.** FR1 is met; FR2 is met at the edge (library `String` errors stay for
plan 3, as regrouped). All five gates passed; gate 4 passed on the
pre-registered rerun after a 1.0114 first row, and a diagnostic traces most
of that miss to where the binary file sits, not to the code.

## What was built

- `src/app/` (binary crate only; the library sees no clap or toml):
  `cli.rs` (clap derive, `Option` flags, no clap defaults, global
  `--config`), `values.rs` (`Seconds`, `Megabytes`, `Workers`, `Repeats`,
  `Evals`, `Until`, `Guide`: one validator each, used by `FromStr` and
  serde), `config.rs` (file tables `[prove]`/`[check]`/`[line]`/`[bench]`
  with `deny_unknown_fields`, per-command configs whose `Default` holds
  today's values, resolution), `error.rs` (one enum, `exit_code`),
  `write.rs` (`atomic`, `check_target`), `commands/{prove,check,line,bench}`.
  Every file is under 10 KB (largest `values.rs` 9.2 KB); `main.rs` is 1.1 KB.
- `check::run` became `check::check_paths(&[PathBuf], CheckOptions) ->
  Result<Vec<Value>, String>`: no parsing, no printing, defensive option
  checks; verdict lines unchanged. `format::save` left the library.
- Effective `config`: a key of the `prove --json` line, of every `check`
  verdict line and of `line`'s result; the first `bench --out` row; text
  `prove` prints `config: k=v …` before its result line.
- Exit codes: 1 adds `line` on an invalid proof (was 2); 3 covers I/O,
  `try_new` memory, the bench reference spawn, checker threads, a failed
  walk after a win and a closed stdout (all were 2 or a panic). Output paths
  (`--proofs`, `--pgn`, `--out`) whose parent is missing or that are a
  directory fail with 2 before any search or spawn.
- Writes: a unique temp file in the target directory, `sync_all`, rename,
  removed on failure; `bench --out` rows are buffered, so a failed spawn
  leaves no file (it used to leave a truncated one).
- `tests/fixtures/config/defaults.toml` spells every present default; absent
  ones are comments.
- Driver: `run_bench.py --side cand --baseline-bin OLD` interleaves
  `base,cand`, books cand/base, records `expansions` and per-cell
  `counters_identical`; existing modes only gain additive summary keys.
- Docs: help text and its exit table; `guidelines.md` exit table;
  `architecture.md` as-built map (2026-11-27); the regrouping note in
  `initiative.md`; `bench/README.md` (`--baseline-bin`).

## Crates

All new `Cargo.lock` entries are MIT and/or Apache-2.0 and were created on or
before 2026-11-20; `scripts/licence.sh` passes. `cargo add` first resolved
clap 4.6.10, toml 1.2.3, toml_parser 1.1.7 and hashbrown 0.17.2 (created
2026-11-21..25); they were pinned back with `cargo update --precise`.

| Crate | Version | Created |
| --- | --- | --- |
| clap / clap_derive / clap_lex | 4.6.8 / 4.6.8 / 1.1.2 | 2026-11-16 |
| clap_builder | 4.6.7 | 2026-09-14 |
| anstyle | 1.0.14 | 2026-03-13 |
| heck | 0.5.0 | 2024-03-12 |
| toml | 1.2.1 | 2026-11-06 |
| toml_datetime / toml_parser / toml_writer | 1.1.3 / 1.1.5 / 1.2.0 | 2026-11-06 / 10-09 / 11-04 |
| serde_spanned | 1.1.3 | 2026-11-06 |
| winnow | 1.0.5 | 2026-11-05 |
| indexmap / hashbrown / equivalent | 2.14.2 / 0.17.1 / 1.0.3 | 2026-09-05 / 05-09 / 10-13 |

`serde` (already locked at 1.0.229) became a direct dependency for `derive`;
the plan's step 2 named only clap and toml.

## Gates

1. **Correctness — passed.** `make test-full` passed (fast and ignored tests,
   `unsafe-feature` gate, veto: 9/9 fixtures accepted, the control
   rejected; `test-full.txt`). The three guard `.pvr` from the new release
   binary are byte-identical to plan1's `guard/proofs/candidate/`
   (`guard/cmp.txt`) and accepted by `470MI check` and `provari check` in
   both rows (`claims.json`). No disagreement.
2. **Defaults and determinism — passed.** Every completed search, 5 base + 5
   cand per cell in each row, has identical `mate`/`nodes`/`child_evals`/
   `expansions`: `core` 14 / 350,430 / 3,896,430 / 191,557, `easy-win` 18 /
   68,692 / 904,368 / 53,237, `deep-win` 20 / 3,553,009 / 44,828,946 /
   2,487,550 — the movegen plan 12 counters. All searches
   completed (30 per row). `config_never_moves_the_search` passes.
3. **Tests — passed.** `make test` and `make doc` are clean; no
   `Result<_, String>` in `src/app/`; the only non-test `expect` is
   `"invariant: config::bench requires out"`. Option-to-test map below.
4. **Measurement — passed on the rerun.** Guard, 5 interleaved repeats,
   10 s, 192 MB, 1 thread:

   | Row | `core` | `easy-win` | `deep-win` | Geomean |
   | --- | ---: | ---: | ---: | ---: |
   | `guard` (pre-registered) | 0.9948 | 1.0518 | 0.9887 | **1.0114** miss |
   | `guard-rerun` (decides) | 0.9852 | 1.0446 | 0.9946 | **1.0078** pass |
   | `guard-tmpfs` (diagnostic) | 0.9851 | 1.0052 | 0.9875 | 0.9926 |

   Medians in the decisive row: base 0.3643 / 0.0785 / 3.8214 s, cand
   0.3589 / 0.0820 / 3.8007 s; spreads ≤ 4.4%. `core` cachegrind `Ir`:
   base 5,980,062,197, cand 5,980,231,156, **+0.0028%** (gate `<= +0.5%`),
   same counters. Diagnostics: mate-in-1 `Ir` +1.20% (13,210,984 →
   13,369,626, i.e. +158,642 instructions, the clap/toml/config edge);
   startup median of 20 walls 5.455 → 7.575 ms with cand on overlay, 5.130
   → 5.315 ms (+3.6%) with both on tmpfs.
5. **Scope — passed.** No diff in `src/{dfpn,table,proofs,eval,moves,board,
   types,walk,search,clock}.rs` (nor `lines.rs`, `lib.rs`); `format.rs` only
   lost `save` and two imports; `check.rs` only its entry, its doc lines and
   one unit test (`scope.txt`).

## Problems and findings

- **The guard row's base and cand sat on different filesystems.** The base
  was built in `/tmp` (tmpfs), the candidate in `target/` (the container's
  overlay). `perf stat -r 30 --version`: cand on overlay 1.48 ms with 10
  major faults per run, the same bytes on tmpfs 0.47 ms with 0; the base
  copied onto overlay 1.07 ms with 4. That is ~1–2 ms per process charged to
  cand only, which is most of the 1–5% on the 80 ms `easy-win` cell and the
  1.0114 first row. With both on tmpfs the row reads 0.9926 and startup
  +0.19 ms. **Finding:** a `--baseline-bin` row must run both binaries from
  one filesystem (copy both to one directory and record it); the driver
  should refuse or warn otherwise. The pre-registered gate is judged on the
  pre-registered rows; the tmpfs row is a diagnostic, not a substitute.
- **The edge has a real, small startup cost:** +158,642 instructions and
  +0.19 ms on a mate-in-1 when filesystems match; the binary grew 3.55 →
  4.15 MB. Invisible at `core` (+0.003% `Ir`).
- **Flaky test fixed in session.** `a_failed_write_keeps_the_old_bytes` once
  saw a `.tmp` from a parallel test: the unit tests shared `/tmp`
  (`CARGO_TARGET_TMPDIR` is not set for a binary's unit tests). Each write
  test now has its own directory. A test-only change made after the
  measurements; the release binary is unaffected.

## Behaviour changes (all at the edge, none in search or verdicts)

- clap rejects extra positionals (silently ignored before) and accepts
  `--flag=value`; `-h` gives the subcommand's help.
- `--evals` above `i64::MAX` is now refused (values come through `i64` so
  negatives get the range message); `--seed` keeps the full `u64`.
- `bench` without `--out` is 2 (as before); a reference that cannot start is
  now 3 and writes nothing. `bench --guide` stays a pass-through string.
- `line` maps every `lines::show` error to 2 (the MOVES given), and a
  decode failure after a valid check to 3; plan 3's typed errors may split
  the first.
- `check` still reports an unreadable proof as a `valid:false` line (exit
  1), unchanged.

## Option-to-test map

| Option / rule | Tests |
| --- | --- |
| every flag, good value in `config` | `cli.rs::{prove,check,line,bench}_flags_land_in_config` |
| every flag, bad value exits 2 naming it (incl. `--tablebase`, `--json=yes`, `--compact-only=yes`, bare `--config`/`--guide`, directory outputs) | `cli.rs::bad_flag_values_are_usage_errors`, `exit_codes_0_to_3` |
| every TOML key, good and bad | `config.rs::every_toml_key_lands_in_config`, `bad_toml_values_are_usage_errors` |
| precedence 64 / 128 / default | `config.rs::flag_beats_file_beats_default` |
| unknown key/table, bad syntax (line named), missing file | `config.rs::config_file_errors_are_usage_errors` |
| defaults snapshot (no flags, and `defaults.toml`) | `config.rs::defaults_match_the_table`, `prove_defaults_with_no_flags_at_all` |
| trajectory neutrality | `config.rs::config_never_moves_the_search` |
| exit codes 0–3, closed stdout | `cli.rs::exit_codes_0_to_3`, `a_closed_stdout_is_an_error_not_a_panic` |
| no-panic sweep | `cli.rs::garbage_never_panics` |
| atomic writes | `app::write` unit tests |
| values | `app::values` unit tests; `check::tests::refuses_zero_workers` |

## Open parts and missing tests

- Library error enums and the `expect` audit: plan 3. Progress logging: F4.
- No test runs `bench` against a real provari (the fake references are
  `/bin/true` and a script on `PATH`); the gate-4 row is the driver's smoke.
- No test drives a write failure through the binary (only the unit seam).
- The driver's new mode has no test of its own, like the rest of the driver.

## Next steps

1. Owner commits the plan 2 tree.
2. Owner decides the same-filesystem rule for `--baseline-bin` rows (driver
   check, `bench/README.md` line).
3. Plan 3: library error enums (`position`, `format`, `lines`, `walk`,
   `search`, `check`) and the `expect` invariant audit.

Evidence: `measurements/plan2/` (`README.md` indexes it).

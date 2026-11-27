# Code guidelines

How code is written in `470MI`. The shape it must fit is
[`architecture.md`](architecture.md); correctness traps are in
[`gotchas.md`](gotchas.md). A rule here may be broken only with a one-line
justification in the code and a finding in the next code audit.

## Tooling

- Rust 2024. `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and
  `cargo doc --no-deps` are clean on every change (`make test`).
- Dependencies are GPL-compatible (`scripts/licence.sh`), at least seven days
  old, and pinned `=x.y.z` when they touch rules, search or proof bytes.

## Design (SOLID in Rust)

| Principle | Rule here |
| --- | --- |
| Single responsibility | one module, one reason to change; its `//!` header says what it owns in one sentence |
| Open/closed | extend at a seam (`Guide`, `ProofTable`, `Observer`, ...), not with a new flag tested deep inside the search |
| Liskov | every implementation of a seam passes the seam's shared contract tests |
| Interface segregation | small traits; functions take the narrowest type they need (`&Position`, not the search) |
| Dependency inversion | libraries depend on traits and values; the app wires concrete types; no globals, no `std::env`, no printing in libraries |

- **Size.** Source files stay under ~10 KB of non-test code. Larger needs a
  header line saying why; "copied upstream, kept whole" is not a reason past
  the next code audit.
- **Coupling.** `pub(crate)` by default; `pub` only for the binary, the
  integration tests or the docs. Struct fields are private unless the type is
  plain data. No module cycles.
- **Cohesion.** Keep a value's rules next to the value: methods on the type,
  not free helpers in another module.
- **Copied code.** After a copy, the file is ours (`D10`): restructure it to
  this architecture in the same plan or record the gap in the backlog.

## Hot path

- Static dispatch (generics or enums); no `dyn`, allocation, logging or I/O
  per node.
- Call `atomic-movegen` directly; a needed rules-layer change is a
  `wishlistN.md` item, not a local adapter (`MD5`, `M7`).
- Any hot-path change reports `instructions:u` on `core` and the counters at
  1 thread; identical counters, or the tree change is the result.
- `unsafe` only for a measured win, with a `// SAFETY:` comment and a guard
  test or gate.

## Errors

- One error enum per layer, implementing `Display` and `std::error::Error`;
  propagate with `?`. No new `Result<_, String>`.
- No `unwrap`/`expect`/`panic!` outside tests, except a proven invariant
  written `expect("invariant: ...")`. User input never panics.
- Messages say what failed, why, and what to do: `error: --memory is
  1..=1048576 MB, not 0`.
- Writes are atomic: a temporary file in the target directory, then rename.
- Exit codes are mapped only in the binary:

| Code | Meaning |
| --- | --- |
| 0 | success (including "no win found") |
| 1 | a checked proof is invalid, or `line` on an invalid proof |
| 2 | usage or configuration error |
| 3 | runtime failure: I/O, memory, a failed sub-process or thread, a closed stdout |

## Configuration

- One `clap` struct and one `serde` struct per command; both resolve into a
  validated config of typed values (`Seconds`, `Megabytes`, `Threads`).
- Defaults live in one place and equal the current product behaviour; a
  changed default is a bench event.
- Precedence default < `--config FILE` < flag. Unknown keys are errors.
- `--json` results carry the effective configuration.

## Logging and output

- stdout: the result only (text, or one JSON line with `--json`). stderr:
  logs (`PD3`).
- Libraries never print; the search emits `Progress` to an `Observer`.
- Line format: `<elapsed> <level> <command>: <message>`, e.g.
  `12.0s info prove: 4.1M nodes 340k/s root 31/1288 table 62% proofs 3%`.
- Levels: `error`, `warn`, `info` (default; progress every 1 s, a final
  summary), `debug`. `-q` keeps errors only.
- Logging is trajectory-neutral: counters are identical with logging off and
  at `debug`, pinned by a test.

## Tests (pyramid)

| Level | Where | Runs in |
| --- | --- | --- |
| unit | `#[cfg(test)] mod tests` beside the code; pure, fast | `cargo test` |
| contract | one shared suite per seam, run against every implementation | `cargo test` |
| integration | `tests/`: CLI through the binary, fixtures, differential | `cargo test` |
| slow | `#[ignore = "slow: ..."]`, e.g. 10 s bench cells | `make test-full` |
| veto | our checker and `provari check` on proof fixtures | `make veto` |
| bench | measurement, not a test | `make bench`, `docs/bench.md` |

- A bug fix starts with a failing test.
- Every flag and config key has a CLI test, including its error path.
- Determinism fixtures pin counters at 1 thread; threaded tests assert
  checker acceptance, not counters.
- `cargo test` needs no network, oracle or tablebases.

## Documentation in code

- Every module starts with `//!`: responsibility, then provenance if copied.
- Public items have `///`. Comments say why, not what. No dead or
  commented-out code.

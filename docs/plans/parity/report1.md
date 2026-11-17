# Report 1 — P0 foundation

**Plan:** [`plan1.md`](plan1.md). **Item:** 1. **Date:** 2026-11-17.
**Outcome:** gates green; no search, no ledger row.

## What landed

- Rust 2024 crate `atomic470m1` with binary `470m1` (Cargo forbids a package
  name starting with a digit; the binary keeps the working name).
- Rules helpers in `src/lib.rs` (`position`, `from_setup`, `ending`, `hash`,
  `status`), plus adapted provari `board.rs` and `moves.rs`.
- Compact `.pvr` v1 read/write and JSONL conversion in `src/format.rs`;
  `docs/proof-format.md` is the adapted `FORMAT.md`.
- Independent checker in `src/check.rs`: shakmaty only, no prover imports,
  public surface `run` and `check_proof`. Tablebases are absent; a compact
  header with `tablebases > 0` is `valid: null` (unchecked).
- `line` viewer in `src/lines.rs` and `src/main.rs`; it runs the checker first
  and refuses a proof the checker rejects.
- CLI: `check`, `line`, reference-side `bench`, `prove` stub, `--version`,
  `--help`.
- `bench/corpus.json` (`corpus-v1`, digest verified), `bench/run_bench.py`,
  `bench/README.md`.
- `scripts/build_oracle.sh`, `scripts/veto.sh`, `scripts/licence.sh`,
  `Makefile` targets `test`, `test-full`, `lint`, `licence`, `oracle`, `veto`,
  `bench`.
- Fixtures: eight oracle-written `.pvr` proofs, one truncated control,
  copied `veto.tsv`/`never_prove.tsv`, integration tests.
- `PROVENANCE.md` rows for every copied file; generated proofs are marked
  independent.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | 28 tests green (26 unit, 2 integration) |
| `make test` | green, including the licence gate |
| `cargo doc --no-deps` | green |
| `make oracle` | provari `0.4.0` at `bceb1ee` built in `/tmp/470M1-oracle` |
| `make veto` | 8/8 fixtures accepted by both checkers; 1/1 control `valid: false` by both |
| Checker independence | `check::tests::uses_nothing_from_the_prover` green |
| Bench reference side | 3/3 wins on all three cells, 3 repeats each |

## Reference measurements (same session)

Command: `provari prove FEN --seconds 10 --memory 192 --guide hand --json`.

| Cell | id | median wall | mate | nodes | wins |
| --- | --- | ---: | ---: | ---: | ---: |
| `core` | `core` | 0.342 s | 17 | 342,289 | 3/3 |
| `easy-win` | `1eb80b92f92c` | 0.075 s | 15 | 72,523 | 3/3 |
| `deep-win` | `db6a741c723f` | 4.721 s | 22 | 4,574,912 | 3/3 |

Parsed rows: `measurements/plan1/results.json`; host/tool versions:
`measurements/plan1/env.json`. No ledger row: there is no candidate search.

## Problems and scope changes

- **Package name.** Cargo rejects `470m1` as a package name (leading digit).
  The package is `atomic470m1`; the binary is `470m1`.
- **`eval.rs` not copied.** Plan task 2 says “as needed by the format and
  walk”; neither uses the hand evaluation. It belongs to plan2 and is left
  there.
- **Tablebases removed, not stubbed.** The checker returns `valid: null` for
  a compact proof that asks for tables. The `line` viewer refuses it. This is
  the plan's “unchecked” rule, not a silent skip.
- **`bench` CLI scope.** The CLI `bench` runs the reference side over a whole
  corpus/JSONL file. The committed driver is the Python one, which selects the
  three fixed cells and writes the parsed measurement files.
- **Oracle build.** The pinned provari builds cleanly; no substitution was
  needed.

## Missing tests / open parts

- No slow fuzz over a stored proof corpus (the fast fuzz test remains).
- No tablebase path by design; a future initiative would need its own checker
  and fixtures.
- No candidate search, so no `prove` proof-writing test and no ledger row.
- The `bench` CLI has no cell filter; the Python driver is the supported path.

## Next steps

1. Execute [`plan2.md`](plan2.md): board/moves/effect differential, TT, proof
   table, DF-PN, hand guide, proof writer, `prove`.
2. Reuse the oracle and `make veto`; add the plan2 proof fixtures.
3. Keep the checker on shakmaty and independent; do not add tablebases.

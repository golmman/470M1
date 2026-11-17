# Plan 1 — P0: repository, rules, proof format, independent checker, bench harness

**Item:** 1 (P0 foundation). **Status:** done (see [`report1.md`](report1.md)). **Session type:** execution.
**Predecessor:** none. **Successor:** [`plan2.md`](plan2.md).

## Goal

Create the first working slice of `470M1`: a Rust crate with the parity rules
layer, provari's compact proof format, an independent checker, a `line`
viewer, a bench harness that can run the reference side, and the oracle/gate
infrastructure. No search yet; `prove` is a stub that says plan2 implements it.

The gate is correctness and infrastructure, not speed: the checker must accept
known provari proofs and reject corrupted controls, and the bench harness must
reproduce the reference side of the three bench cells.

## Context

- Read [`initiative.md`](initiative.md), [`decisions.md`](decisions.md) and
  [`gotchas.md`](gotchas.md) first.
- The parity reference is provari `v0.4.0` at
  `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`.
- The repository is GPL-3.0-or-later. provari code may be copied/adapted with
  provenance. `atomi` and `atomic_solver` are MIT and may also be copied.
- The checker must use shakmaty only and share no code with the prover (D4).
- No tablebases, NNUE/model, web or UCI (D7). Proofs carry `tablebases: 0`.

## Tasks

1. **Crate skeleton.** Create `Cargo.toml` (Rust 2024, `shakmaty` 0.30.1 with
   `variant`, `serde_json`, `zstd`), `src/lib.rs`, `src/main.rs`, and the
   module layout from `AGENTS.md`. Add a `Makefile` with `test`, `test-full`,
   `lint`, `oracle`, `veto`, `bench` targets (the last three may be stubs
   until the oracle/driver exist). Add `.gitignore` for `target/`,
   `__pycache__/`, raw logs and oracle scratch data.

2. **Rules layer.** Copy/adapt provari's `src/lib.rs` position/ending helpers
   and `src/board.rs`/`src/moves.rs`/`src/eval.rs` as needed by the format and
   walk. Keep shakmaty as the rules authority. Remove tablebase, NNUE/model,
   web and UCI code. Add a `status`/`hash` helper for the walk that uses
   `EnPassantMode::Legal`. Record every copied file in `PROVENANCE.md` with
   the source commit, path, sha256 and mode.

3. **Proof format.** Copy/adapt provari's `src/format.rs` and `FORMAT.md`.
   Implement compact `.pvr` v1 read/write and JSONL read/write. Remove
   tablebase verdicts; a proof with `tablebases > 0` is `unchecked` (the
   checker has no tables). Keep the exact move order, LEB128 indices, zstd
   frame and header fields. Add round-trip tests and a known-proof fixture.

4. **Independent checker.** Copy/adapt provari's `src/check.rs` into
   `src/check.rs`. Remove `shakmaty-syzygy` and all tablebase code. Keep the
   checker's "shares no code with the prover" property: it may use shakmaty
   only, and its only public entry points are `run` and `check_proof`. Add a
   test that reads `src/check.rs` and fails if it imports a prover module.
   Implement `check PROOF... [--workers N] [--seconds S] [--compact-only]`
   with provari's JSON-line output and exit code.

5. **Line viewer.** Copy/adapt provari's `src/walk.rs` and `src/lines.rs`.
   Implement `line PROOF [MOVES] [--pgn FILE] [--until N]` for compact and
   JSONL proofs. Remove tablebase branches. The viewer must reject a proof
   that the checker rejects.

6. **CLI.** Implement `check`, `line`, `bench` (reference side only), a
   `prove` stub that exits with a clear "search is plan2" error, `--version`
   and `--help`. Keep the option names and output shapes close to provari so
   the bench driver and future plans can reuse them.

7. **Bench harness and corpus.** Copy `corpus-v1` from atomi/atomic_solver to
   `bench/corpus.json` and verify sha256
   `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`.
   Adapt `bench/run_bench.py` to run the reference side
   (`provari prove "<FEN>" --seconds 10 --memory 192 --guide hand`), parse
   walls/nodes/mate, and write `docs/plans/parity/measurements/plan1/`.
   Add `bench/README.md` with the protocol and the corpus digest gate. Do not
   append a ledger row: the ledger holds candidate rows only.

8. **Oracle and veto.** Add `scripts/build_oracle.sh` that clones provari at
   the pinned commit into `/tmp/470M1-oracle`, builds it, and records
   `provari --version`. Add `make oracle` and `make veto`. `make veto` runs
   our checker and `provari check` on the proof fixtures and on corrupted
   controls; a missing oracle is red, not skipped. Controls must be
   `valid: false`, never `null`.

9. **Fixtures and tests.** Add known-win FENs and proofs from the legacy
   fixtures (with provenance), a corrupted-proof control, a JSONL/compact
   round-trip test, a checker-independence test, and a test that the checker
   rejects a proof with a repeated position, a 50-move failure and a missing
   move. Add `tests/` integration tests and keep unit tests in modules.

10. **Report.** Write `report1.md` with tools used, problems, open parts,
    missing tests, the measured reference values, and next steps. Update
    `docs/plans/README.md` and `docs/plans/parity/README.md` status.

## Gates

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` green.
- `make test` green (licence gate plus fast tests).
- `make oracle` builds the pinned provari and records its version.
- `make veto` green: our checker accepts the known provari proofs and
  `provari check` agrees; every corrupted control is `valid: false`.
- The checker-independence test passes.
- The bench harness reproduces the reference side of `core`, `easy-win` and
  `deep-win` at 10 s/192 MB, three repeats, and writes parsed results plus
  `env.json` under `measurements/plan1/`.
- No ledger row is appended (no candidate search yet).

## Deliverables

- Working crate with `check`, `line`, `bench` and a `prove` stub.
- `docs/proof-format.md` (or equivalent) and the format implementation.
- Independent checker and its tests.
- `bench/corpus.json`, `bench/run_bench.py`, `bench/README.md`.
- `scripts/build_oracle.sh`, `Makefile`, `PROVENANCE.md` rows.
- `measurements/plan1/` with reference values and `env.json`.
- `report1.md`.

## Risks and stop rules

- If the checker cannot accept a known provari proof, stop and fix the
  checker before any other work.
- If the corpus digest does not match, stop; do not silently use another
  corpus.
- If the oracle cannot be built, record `INCONCLUSIVE`; do not substitute a
  different provari version.
- Do not start the search core in this plan. If a task needs it, move it to
  plan2 and record the scope change in the report.

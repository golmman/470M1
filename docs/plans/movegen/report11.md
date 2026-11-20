# Report 11 — `unsafe-optimizations` on 2.11.0: audited, safe masks rejected, adopted

**Item:** `M15` (build-shape decision). **Plan:** [`plan11.md`](plan11.md).
**Measurements:** [`measurements/plan11/`](measurements/plan11/README.md).
**Decisions:** `MD9` added; `MD1` and `MD6` unchanged. **Budget:** one sitting.

## Outcome

`uo` is adopted. On the adopted 2.11.0 pin the feature falls **90,264,864**
instructions on `core` (1.263%), 20,497,022 on `easy-win` (1.313%) and
974,758,363 on `deep-win` (1.253%). The safe-Rust alternative `sm` (mask the
`ZOBRIST` and `ENTRIES` indices) is **slower than `base`** on every cell, so it
recovers none of the fall; the pre-registered rule takes the **adopt** branch,
not `W11` and not close. All witnesses hold: counters/mate identical in every
round, `.pvr` byte-identical on all three cells, both checkers 9/9, differential
`mismatches: 0`, `key_mismatches: 0`, EP 87/0/0.

`M15` closes. The product win is 1.3% and a cleaner `M8` baseline; `MD6` is
still **not held** (candidate/old geomean **1.012**, from 1.028; `core`
**7.0573e9**, **1.074×** plan 2, from 1.088×).

## Phase A — audit (read-only, 2.11.0)

Exactly three unsafe blocks, all in the MIT dependency:

| Site | Load | Invariant |
| --- | --- | --- |
| `board.rs:3694` `piece_key_raw` | `ZOBRIST.piece[sq][raw]` | `sq < 64`, `raw < 16`, `debug_assert!`-ed |
| `magic.rs:489` `bishop_attacks` | `BISHOP_TABLE[offset + idx]` | `idx < 1 << (64 - shift)`; builder allocates exactly that many |
| `magic.rs:512` `rook_attacks` | `ROOK_TABLE[offset + idx]` | same argument |

The plan's two load-bearing assumptions were tested, and one changed:

1. **The panic→UB delta does not exist for magic.** `let e = &ENTRIES[sq as
   usize]` is a safe index into `[MagicEntry; 64]` *before* the unsafe table
   load, so `Square::NONE` still panics in the feature build. Only
   `piece_key_raw` could be UB on `NONE`.
2. **The plan's prover call-site list is shakmaty's.** `src/moves.rs:91,92,
   285,286,490,491,514,517,520` call `shakmaty::attacks`, not
   `atomic_movegen::attacks`. The crate's magic is reached through `Board`
   internals; the prover's only explicit entry is `src/board.rs:339`
   (`Position::attackers_to`). `checked_after` (`src/board.rs:352`) is unused.

Call-site closure: `attackers_to` receives a move endpoint (`from()`/`to()`,
decoded with `bits & 63`), an iterated bitboard square, `Some(king)` from
`king()`, or `maybe_ep_square()` — never `NONE`. Crate-internal callers take
squares from bitboard iteration or generated moves. `cargo tree -e features`
shows only the product enables the feature.

**Guard.** Upstream `0aaf958` has **no** `.github` directory and no CI. A CI
job is not a crate change, so `measurements/plan11/upstream-ci.patch` adds a
workflow running `cargo test` and `cargo test --features
unsafe-optimizations` with fmt/clippy; the owner pushes it. The enforceable
guard in this repository is `make unsafe-feature`
(`scripts/unsafe_feature_gate.sh`), which runs the pinned crate's own suite in
the shipped shape from the registry copy Cargo builds, and is wired into
`make test-full`. This is a scope addition: the plan named upstream CI only,
and the upstream half cannot be landed from an agent session.

## Phase B — safe alternative and feature

Three interleaved `perf stat -e instructions:u` rounds, order `base`, `uo`,
`sm`, `--seconds 10 --memory 192 --seed 0 --json`:

| Cell | `uo` fall | `uo` % | `sm` fall | `sm` % | `sm`/`uo` recovery |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | **90,264,864** | 1.263 | +18,103,780 | −0.253 | **−20.1%** |
| `easy-win` | **20,497,022** | 1.313 | +3,716,666 | −0.238 | **−18.1%** |
| `deep-win` | **974,758,363** | 1.253 | +190,478,555 | −0.245 | **−19.5%** |

`sm` masks `ZOBRIST.piece[(sq as usize) & 63][(raw as usize) & 15]` and
`ENTRIES[(sq as usize) & 63]`; the flat `TABLE[offset + idx]` cannot be masked
without aliasing, so the shape is partial by construction. It removes the two
entry bounds checks and both Zobrist checks but leaves the table bounds check,
and the masks cost more than they save. The feature is material on every cell
(≥2.8× the larger spread, ≥0.5%), so the decision rule selects adopt.

The base binary is `2eb218a6…` (plan 10's product binary), the feature binary
`b287d7eb…`; the adopted product `target/release/470MI` rebuilds to the same
`b287d7eb…`. Host identity again rests on the binary hash, platform string and
`core` reading (UTS `36f35886a6b3`), recorded as provisional in
`env.json`/`decision.json`.

## Phase C — adoption

- `Cargo.toml`: `atomic-movegen = { version = "=2.11.0", features =
  ["unsafe-optimizations"] }`; lockfile and pin unchanged.
- `PROVENANCE.md`: the 2.11.0 source row names the three audited sites, the
  safe `ENTRIES` load / `NONE` argument and the guard.
- `decisions.md`: dated **`MD9`** (adopt) with the audit, the `sm` failure, the
  guard and the `MD6` result.
- `scripts/unsafe_feature_gate.sh` + `make unsafe-feature`; `make test-full`
  now runs it. `upstream-ci.patch` is prepared, not landed.
- `initiative.md`: `M15` closes, status and history updated; `AGENTS.md`
  current-state line updated; `bench.md` plan11 row, footnote and pin row.

No `src/`, search, eval, tree or checker change. `check.rs` is untouched.

## `MD6` row

Plan 3 driver, unchanged, 10 s, protocol `a`, seed 0, three repeats, product
and plan 2 binary back to back:

| Cell | Reference | Candidate | Factor | Mate / ref | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.537 s | 0.587 s | **1.093** | 14 / 17 | 3/3 |
| `easy-win` | 0.136 s | 0.145 s | **1.066** | 18 / 15 | 3/3 |
| `deep-win` | 6.591 s | 5.717 s | **0.867** | 20 / 22 | 3/3 |
| **Geomean** | | | **1.003** | | **9/9** |

Same-session old binary: 1.002 / 1.015 / 0.957, geomean **0.991** (anchor
0.972). Candidate/old **1.091 / 1.050 / 0.906**, geomean **1.012**; `D14`
counters identical; 9/9 accepted by both checkers. This is a row, not a pass.

## Gates

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
`cargo doc --no-deps`, `make test`, `make test-full` and `make veto` are green;
the shipped shape additionally ran the release differential
(`rules_differential_full`) at 0/0 and the crate's feature suite. No scratch
tree carries a repository edit; the `sm` patch is diagnostic only.

## Problems, scope changes and open parts

- **Upstream CI is not landed.** `0aaf958` has no CI; the patch is prepared and
  the local `make unsafe-feature` gate covers the pin today. Pushing the
  workflow is the owner's action and needs no pin/provenance cycle.
- **Two plan assumptions were wrong** (safe `ENTRIES` load; shakmaty call
  sites). The audit closed both, so the item was not rejected unmeasured.
- **Local guard added** beyond the plan's text, because the upstream half was
  not executable here. It runs the crate's own tests in the shipped shape.
- **`sm` is a partial recovery by construction.** The flat table load has no
  meaning-preserving mask; the experiment bounds what safe Rust can recover.
- **`MD6` remains missed.** 0.0903e9 leaves `core` at 1.074× plan 2; `M8` is
  the only open item.
- **Missing tests.** There is no test that the shipped binary uses the feature
  (it is a Cargo feature, not a runtime switch); the guard tests the same
  crate source and the same flag. There is no host-label test.

## Next steps and options

1. **`M8`, complete shakmaty removal (recommended).** The only path to the
   initiative's done definition; independent of the remaining performance gap.
2. **Push the upstream CI workflow** from `measurements/plan11/upstream-ci.patch`
   at `0aaf958`; no crate or pin change.
3. **Kill review.** `movegen` has none after plans 4, 6, 8 or 10; run the fixed
   prompt before the next plan.

Option 1's kickoff prompt:

> Read `docs/plans/movegen/initiative.md`, `report11.md`, `report10.md`,
> `decisions.md` (`MD1`, `MD5`, `MD6`, `MD9`) and `measurements/plan11/`. Plan
> 12: complete `M8`, removing shakmaty from the prover (parser, display, proof
> paths, line output) while `check.rs` stays on shakmaty; keep the differential
> green, gates/veto green, and run the same-session `MD6` row. Do not change
> the adopted `atomic-movegen` pin or the `unsafe-optimizations` feature.

# Plan 11 measurements — `unsafe-optimizations` audited, safe masks measured, adopted

**Date:** 2026-11-20. **Host:** bench host (`c90face26537`; this container's
UTS name is `36f35886a6b3`, platform `Linux-7.2.13-200.fc44.x86_64`, the same
string as plans 7/9/10). **Pins:** `atomic-movegen` 2.11.0 (`0aaf958`, MIT),
default plus the feature; provari 0.4.0 (`bceb1ee`). **Tree:** `470MI@fedf389`
plus the plan's working tree. **Outcome:** `MD9` adopts
`unsafe-optimizations`; the safe masked-index alternative is slower; `M15`
closes.

## Files

- `run_plan11.py` — builds `base` (`git archive HEAD`, default features), `uo`
  (`uo.patch`: the feature, lockfile unchanged) and `sm` (`sm.patch` applied to
  a copy of the registry crate, wired in with `[patch.crates-io]`; never
  committed) under `--scratch` (`/tmp/plan11`). It runs the ignored release
  differential in `uo` and `sm`, then three interleaved rounds of `perf stat -e
  instructions:u` on `core`, `easy-win` and `deep-win`, order `base`, `uo`,
  `sm`. Each variant exports one `.pvr` per cell for sha256 and both checkers.
  It writes `perf.txt`, `results.json`, `env.json` and `decision.json`.
  `--no-perf` checks plumbing only.
- `uo.patch` — `Cargo.toml` with `features = ["unsafe-optimizations"]` on
  `=2.11.0`. The lockfile is unchanged.
- `sm.patch` — the safe-Rust diagnostic: mask the `ZOBRIST.piece` indices with
  `& 63` / `& 15` and the safe `ENTRIES[sq]` loads with `& 63`. The flat
  `TABLE[offset + idx]` load cannot be masked without aliasing, so this is a
  partial recovery and never a committed change.
- `upstream-ci.patch` — the prepared upstream workflow: `cargo test` and
  `cargo test --features unsafe-optimizations` plus fmt/clippy. It is not
  landed at `0aaf958`; the owner pushes it.
- `bench/` — the plan 3 driver's 10 s protocol-`a` row: `results.json`
  (product, feature enabled), `old.json` (plan 2 binary rebuilt from `2f629fd`)
  and `env.json` (both sides and hashes). The old pass overwrote the driver's
  `env.json`, so the candidate fields were reconstructed as in plan 10.
- `perf.txt`, `results.json`, `env.json`, `decision.json` — Phase B evidence.

No binaries, `.pvr`, `perf.data`, raw transcripts or oracle artifacts are
committed.

## Phase A — audit on 2.11.0

The unsafe surface is exactly three blocks under the MIT dependency's feature:

| Site | Load | Invariant |
| --- | --- | --- |
| `board.rs:3694` `piece_key_raw` | `ZOBRIST.piece[sq][raw]` | `sq < 64`, `raw < 16`, both `debug_assert!`-ed |
| `magic.rs:489` `bishop_attacks` | `BISHOP_TABLE[offset + idx]` | `idx < 1 << (64 - shift)`; the builder allocates exactly that many |
| `magic.rs:512` `rook_attacks` | `ROOK_TABLE[offset + idx]` | same argument |

Two plan assumptions were corrected. First, `let e = &ENTRIES[sq as usize]`
precedes the unsafe table load and is an ordinary safe index into
`[MagicEntry; 64]`, so `Square::NONE` still panics in the feature build; the
"panic becomes UB" delta does not hold for the magic paths. Only
`piece_key_raw` could read out of bounds on `NONE`. Second, the prover's
`src/moves.rs` calls named in the plan are **shakmaty's** `attacks::*`, not the
crate's; the crate's magic is reached only through `Board` internals.

Call-site closure: `Position::attackers_to` is the prover's only explicit
entry to crate magic. Its `Square` comes from a move endpoint
(`from()`/`to()`, decoded with `bits & 63`), an iterated bitboard square, a
`Some(king)` from `king()`, or `maybe_ep_square()` — never `Square::NONE`.
`checked_after` is unused. `cargo tree -e features` shows only the product
enables the feature, and `cargo test --features unsafe-optimizations` in the
pinned crate is green (all unit, integration and doc tests).

## Phase B — measurement (`perf stat -e instructions:u`, three rounds)

| Cell | base median | `uo` median | `uo` fall | `uo` % | `sm` median | `sm` fall | `sm` % | spreads (base/`uo`/`sm`) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `core` | 7,147,609,020 | 7,057,344,156 | **90,264,864** | 1.263 | 7,165,712,800 | +18,103,780 | −0.253 | 334 / 451 / 474 |
| `easy-win` | 1,560,746,879 | 1,540,249,857 | **20,497,022** | 1.313 | 1,564,463,545 | +3,716,666 | −0.238 | 197 / 155 / 408 |
| `deep-win` | 77,783,668,246 | 76,808,909,883 | **974,758,363** | 1.253 | 77,974,146,801 | +190,478,555 | −0.245 | 1,030 / 787 / 104 |

`uo` pair falls are stable to a few thousand instructions;
`sm` pair falls are negative on every round. `sm` recovers **−20.1% /
−18.1% / −19.5%** of `uo`'s fall, against the 70% line. Both variants:
differential `mismatches: 0`, `key_mismatches: 0`, EP 87/0/0; counters/mate
identical in every round; `.pvr` byte-identical on all three cells; our
checker and `provari check` accept 9/9. `decision.json` branches **`uo`**:
`uo` is material on every cell (≥2.8× the larger spread, ≥0.5%), so `MD9`
adopts the feature.

## `MD6` row — 10 s, protocol `a`, seed 0, 192 MB, three repeats

Driver: plan 3's `run_bench.py`, unchanged. `bench/results.json` is the
product binary `b287d7eb…` with the feature; `bench/old.json` is the plan 2
binary (`71fff646…`, rebuilt bit-identical from `2f629fd`). The two passes ran
back to back.

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.537 s | 0.587 s | **1.093** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.136 s | 0.145 s | **1.066** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.591 s | 5.717 s | **0.867** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.003** | | | **9/9** |

Same-session plan 2 binary:

| Cell | Reference | Old candidate | Old factor | Old mate / ref | Old `child_evals` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.530 s | 0.531 s | **1.002** | 17 / 17 | 3,823,720 |
| `easy-win` | 0.132 s | 0.134 s | **1.015** | 15 / 15 | 897,937 |
| `deep-win` | 6.616 s | 6.330 s | **0.957** | 19 / 22 | 54,633,842 |
| **Geomean** | | | **0.991** | | | **9/9** |

Coverage 3/3, 9/9 accepted by both checkers, `D14` counters identical for
every cell. Candidate/old is **1.091 / 1.050 / 0.906**, geomean **1.012** (was
1.028 in plan 10): `MD6` is still **not held**. `core` instructions are
**7.0573e9** against plan 2's **6.5694e9**, a **1.074×** gap (was 1.088×).

## What this instrument cannot see

The pair measures instruction counts and wall separately; the 1.3% is a true
fall on every cell but does not move the `MD6` class. Host identity rests on
the product binary sha256, the platform string and the base `core` reading, not
the container's UTS name. The differential cannot see an out-of-bounds read
that happens to return correct bytes; the three-site audit and the crate's own
feature tests are the guard. The safe alternative is partial by construction:
the flat magic table load has no mask that preserves meaning.

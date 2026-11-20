# Plan 12 measurements — `M8`: shakmaty removed from the prover

**Date:** 2026-11-20. **Host:** bench host (`c90face26537`; container UTS
`36f35886a6b3`, platform `Linux-7.2.13-200.fc44.x86_64`). **Pins:**
`atomic-movegen` 2.11.0 (`0aaf958`, MIT) with the audited
`unsafe-optimizations` feature; provari 0.4.0 (`bceb1ee`). **Tree:**
`470MI@0d8ac90` plus the plan's working tree. **Outcome:** `M8` closes; no pin,
feature or checker change; instruction count is parity; `MD6` remains a
recorded miss at 1.013 candidate/old.

## Files

- `run_plan12.py` — builds the plan 11 product (`HEAD`) and the plan 2
  reference (`2f629fd`) under `--scratch`, builds the working tree, runs three
  interleaved `perf stat -e instructions:u` rounds over the three cells
  (order `head`, `cand`), then invokes the plan 3 bench driver for the
  candidate and the old binary. It writes `perf.txt`, `env.json` and `bench/`.
  `--skip-perf` / `--skip-bench` check one half.
- `perf.txt`, `env.json` — Phase C instruction evidence, with binary hashes.
- `bench/results.json` — candidate `470MI` `8d807f21…`, plan 3 driver, 10 s,
  protocol `a`, seed 0, three repeats, `--d14`.
- `bench/old.json` — the same driver and session on `470m1` `71fff646…`,
  rebuilt bit-identical from `2f629fd`.
- `bench/env.json` — both sides and hashes; the old pass overwrote the
  driver's own `env.json`, so both were reconstructed (as in plans 10/11).

No binaries, `.pvr`, `perf.data`, raw transcripts or oracle artifacts are
committed.

## Phase A/B — what left the prover

`grep -rn shakmaty src/` after the plan names only `src/check.rs` and the
comment in `src/board.rs` saying the prover does not use it. The removed
surface, by module:

| Surface | Before | After |
| --- | --- | --- |
| Parser | `lib::position`/`from_setup` on shakmaty `Setup` | `Board::from_fen_with(FenPolicy::Lichess)` |
| Display | `Fen::from_position` | `Position::to_fen` (`Board::fen_with(Legal)`) |
| Value types | shakmaty `Bitboard`, `Color`, `Piece`, `Role`, `Square`, `attacks` | `src/types.rs` re-exports of the crate's types plus line tables |
| Proof paths | `format::Strategy` on `shakmaty::Move`, `board_move`, `to_shakmaty` | our `Move` directly; `Move::from_uci` for JSON lines |
| Line output | `SanPlus`, `play_unchecked`, `fullmoves().get()` | `Board::san`, `Position::play`, `Position::fullmoves` |
| Walk/search boundary | `walk::Moves` on shakmaty `Move`, `Atomic` params | our `Move`/`Position`; `dfpn` already searched our `Position` |

`src/check.rs` is untouched. `tests/differential.rs` keeps shakmaty as the
independent oracle; it now rebuilds the shakmaty side from FEN through a local
`shak_position` helper, so the prover exports no conversion method at all.

The first removal measured **+2.50%** instructions on `core`. Two prover-side
fixes, both outside the crate, closed it: an `#[inline(always)]` dispatch for
`attacks_from` (the crate's `#[inline]` helper was not inlined in the
`eval`/`Mover` loops) and a direct bit-scan `Squares` iterator. No crate or pin
change.

## Phase C — instructions (`perf stat -e instructions:u`, 3 rounds)

| Cell | plan 11 product (`0d8ac90`, `b287d7eb…`) | M8 product (`8d807f21…`) | fall | % | spreads |
| --- | ---: | ---: | ---: | ---: | --- |
| `core` | 7,057,344,364 | 7,061,619,735 | −4,275,371 | −0.061 | 266 / 284 |
| `easy-win` | 1,540,250,238 | 1,536,250,632 | +3,999,606 | +0.260 | 164 / 217 |
| `deep-win` | 76,808,909,651 | 76,684,453,244 | +124,456,407 | +0.162 | 939 / 862 |

Counters and mate are identical for head and candidate in every round on all
three cells. `core` pays 0.061% (4.3M, about 12 instructions per node) for the
removal; the other two cells fall. This is parity, not a material regression.

## `MD6` row — 10 s, protocol `a`, seed 0, 192 MB, three repeats

| Cell | Reference | Candidate | Factor | Mate / ref | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.516 s | 0.576 s | **1.116** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.135 s | 0.143 s | **1.059** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.614 s | 5.614 s | **0.849** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.001** | | | **9/9** |

Same-session plan 2 binary (`470m1` `71fff646…`):

| Cell | Reference | Old candidate | Old factor | Old mate / ref | Old `child_evals` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.516 s | 0.515 s | **0.998** | 17 / 17 | 3,823,720 |
| `easy-win` | 0.135 s | 0.136 s | **1.007** | 15 / 15 | 897,937 |
| `deep-win` | 6.656 s | 6.397 s | **0.961** | 19 / 22 | 54,633,842 |
| **Geomean** | | | **0.988** | | | **9/9** |

Candidate/old is **1.118 / 1.052 / 0.883**, geomean **1.013** (plan 11:
1.012). Coverage 3/3, 9/9 accepted by both checkers, `D14` timed and untimed
counters identical on every cell. `MD6` is still **not held**; the row is a
record, not a pass.

## Correctness witnesses

The release full differential is unchanged: 84/11 FENs, **27,088** positions,
**540,383** moves/keys, `mismatches: 0`, `key_mismatches: 0`, EP **87/0/0**,
explosions 72,131 with 0 mismatches, material exempt 642, 392 games / 26,612
play-out nodes. The three cell proofs are byte-identical to plans 10/11:

| Cell | `.pvr` sha256 prefix | Mate | `child_evals` |
| --- | --- | ---: | ---: |
| `core` | `714723211c7a0246…` | 14 | 3,896,430 |
| `easy-win` | `32ffb30d96ccb172…` | 18 | 904,368 |
| `deep-win` | `50f062de2a67b1dd…` | 20 | 44,828,946 |

`470MI check` and `provari check` accept all three (9/9). `make test`,
`make test-full` and `make veto` are green; `make test-full` includes the
crate feature gate, the full differential and the nine proof fixtures.

## What this instrument cannot see

Instruction parity is not a wall parity claim: `core`'s 0.061% is below the
`MD6` class, and the wall row is the authority. The differential compares rules,
not the value/attack vocabulary at instruction level; an aliasing or table bug
would have to show as a counter, mate or `.pvr` difference, and none did. The
test suite still uses shakmaty as the oracle, so the independent implementation
is preserved where it matters; no host-label test exists, and the container's
UTS name is not the owner's host label.

# Plan 11 — `unsafe-optimizations` on 2.11.0: audit, safe alternative, decide

**Item:** new `M15` (build-shape decision). **Decisions:** `MD1`, `MD6`; a dated
`MD9` row only on the adopt or safe-ask branch. **Predecessors:**
[`report9.md`](report9.md) §`uo`, [`report10.md`](report10.md),
[`measurements/plan9/`](measurements/plan9/README.md). **Report:** `report11.md`.
**Budget:** one sitting (~3 h).

## Goal

Turn plan 9's `uo` datum into a decision on the **adopted pin** 2.11.0, closing
the three things plan 9 left open: the unsafe **audit**, a **safe-Rust
alternative** that changes the formulation, and the **guard** AGENTS'
"documented, guarded" rule requires.

Not a re-run: the number is one pin old (`W10` changed the legality path),
nobody audited the three blocks against *our* call sites, "guarded" was never
specified, and the safe alternative was never measured — so the owner is
choosing between "unsafe" and "nothing" when a third shape may exist.

**Not here:** no `M8`, no search/eval/tree/checker change, no committed crate
source change, no second release, no claim that `uo` closes `MD6`.

## The unsafe surface (read-only census, 2.11.0)

Exactly **three** blocks under `feature = "unsafe-optimizations"`, all in the
MIT dependency; `src/lib.rs:306` is `forbid(unsafe_code)` without the feature.

| Site | Load | Invariant |
| --- | --- | --- |
| `board.rs:3700` `piece_key_raw` | `ZOBRIST.piece[sq][raw]` | `sq < 64`, `raw < 16`, both `debug_assert!`-ed |
| `magic.rs:497` `bishop_attacks` | `BISHOP_TABLE[offset + idx]` | `idx < 1 << (64 - shift)`; builder allocates exactly that many |
| `magic.rs:518` `rook_attacks` | `ROOK_TABLE[offset + idx]` | same argument |

Our prover has no `unsafe` of its own and no `forbid(unsafe_code)`; the crate's
flag does not touch that, and the surface is small enough to audit by hand.

**The one semantic delta, and it is not visible in a differential.** Without
the feature `bishop_attacks(Square::NONE)` **panics**; with it, that call is out
of bounds and **UB**. A differential cannot see it on inputs we never send, so
the plan audits the call sites instead of trusting the green harness:
`src/moves.rs:91,92,285,286,490,491,514,517,520` and `src/board.rs:339,352`
(`attackers_to`). Phase A records per site where the `Square` comes from —
bitboard iteration or an `Option<Square>` filtered at `src/board.rs:329-333`. A
site that cannot be closed rejects `uo` outright.

## The safe alternative (`sm`) — the formulation change

Masking the index is safe Rust and makes the check provably dead:

```rust
ROOK_ENTRIES[(sq as usize) & 63]
ZOBRIST.piece[(sq as usize) & 63][(raw as usize) & 15]
```

`sm` applies those two masks in a scratch tree. The flat table load
`TABLE[offset + idx]` **cannot** be masked without changing meaning (a mask
would silently alias), so `sm` recovers the entry and Zobrist checks only — a
**partial** recovery. That bound is the experiment: if the two masked loads
carry the bulk of the 1.3%, `uo` becomes unnecessary and `forbid(unsafe_code)`
stays intact for the whole dependency tree.

`uo` is 0.103e9 on `core`; `sm` recovers between 0 and all of it, so the
experiment is one build and one round per cell.

## Decision rules (owner, 2026-11-21)

The owner took the safe-alternative-first route: measure `sm` in the same
session as `uo`, adopt whichever shape is justified, and stop leaving `uo`
undecided. No threshold debate: the 70% line is a build decision, not a
statistical one.

| Branch | Rule | Outcome |
| --- | --- | --- |
| `sm` | recovers **≥ 70%** of `uo`'s fall on every cell, with the same correctness witnesses | `wishlist4` `W11` (masked indices); `uo` closes for good |
| else | `uo`'s fall is still **≥ 0.5%** on every cell and the witnesses hold | **adopt** the flag |
| else | less than that, a witness fails, or Phase A cannot close a call site | **close** `uo` with the finding |

The witnesses are counters and mate identical in every round, `.pvr`
byte-identical, both checkers 9/9 and differential `mismatches: 0`. `0.31e9`
stays the *release* bar (upstream implement, pin, `PROVENANCE.md`, `MD6` row);
`uo` costs none of that.

`MD6` is untouched either way: 0.103e9 leaves `core` near **7.05e9**, still
**1.07×** plan 2's 6.5694e9. The adopt branch is a 1.3% product win and a
cleaner `M8` baseline, not progress on parity.

**Adopt is not free.** `MD6` requires a same-session row, so adoption owes a
full `MD6` row (plan 3 driver, 3/3, 9/9, `D14`) and a `bench.md` row — report 9
has neither for `uo`.

## Phases

**0 — owner trigger (~10 min).** **Met (2026-11-21):** option 3 of the review —
safe alternative first, `uo` adopted only if `sm` cannot carry the win.

**A — audit (~45 min, read-only).** The site table, the call-site `Square`
provenance table, `cargo tree -e features` (nothing else pulls the feature in),
and the guard: does the upstream repo at `0aaf958` run `cargo test --features
unsafe-optimizations` in CI? **A CI job is not a crate change**, so adding one
needs no pin or provenance cycle. Cite it if it exists; if not, the adopt branch
adds it before `Cargo.toml` changes.

**B — measure (~1 h, owner, bench host).** `run_plan11.py` is
[`run_plan9.py`](measurements/plan9/run_plan9.py) with variants `base`, `uo`
(`features = ["unsafe-optimizations"]`, lockfile unchanged) and `sm`
(diagnostic patch, never committed). Three interleaved
`perf stat -e instructions:u` rounds per cell, order `base`, `uo`, `sm`,
`--seconds 10 --memory 192 --seed 0 --json`; the ignored release differential in
`uo` and `sm`; one `.pvr` export per variant and cell with sha256, our checker
and `provari check` when `/tmp/470M1-oracle` exists. Writes `perf.txt`,
`results.json`, `env.json`, `decision.json` under
`measurements/plan11/`. Plan 9's host-label caveat is recorded again, not
worked around: identity is the `base` binary sha256 `b7c16c94…`, the platform
string and the `core` reading.

**C — branch (~1 h).**

- *Adopt:* `atomic-movegen = { version = "=2.11.0", features =
  ["unsafe-optimizations"] }`; `PROVENANCE.md` source row gains the audited
  three sites and the `NONE` argument; upstream CI gate in place; dated `MD9`;
  `MD6` row and `bench.md` row. Every gate then covers the shipped shape.
- *`sm`:* `wishlist4.md` `W11` with the four-line change, the masked-index
  argument, the measured fall and the crate's exhaustive magic tests as the
  oracle; dated `MD9` closing `uo` to a release ask. No pin, no `Cargo.toml`.
- *Close:* dated `MD9` (option b), `uo` recorded as measured and declined.
- *Reject:* Phase A found an unreachable-by-argument gap; `uo` closes with the
  finding and no measurement is run.

**D — artefacts and docs (~15 min).** `measurements/plan11/`: README, driver,
patches, `perf.txt`, `results.json`, `env.json`, `decision.json`, plus the
`MD6`/bench files on the adopt branch. No binaries, `.pvr`, `perf.data`,
transcripts or oracle artifacts. `initiative.md` gets `M15`, a status line and a
history line; `AGENTS.md` the one-line state update.

## Gates

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `cargo doc --no-deps`, `make test`, `make test-full`, `make veto` green before
  and after; scratch trees carry no repository edits.
- The adopt branch additionally runs clippy, tests and the release differential
  (`cargo test --release --test differential rules_differential_full --
  --ignored`) **in the shipped shape**.
- `check.rs` untouched; the checker stays the independent veto.

## Risks

| Risk | Mitigation |
| --- | --- |
| A green differential hides the `NONE` panic→UB change | Phase A call-site audit; a gap rejects the item unmeasured |
| Identical counters by luck (an OOB read returning correct bytes) | Not excluded by data; the invariant argument and the crate's exhaustive magic tests are the guard, so the audit is load-bearing |
| A sub-0.31e9 adoption erodes the release rule | `MD9` records that `uo` was adopted only after the safe shape was measured and failed |
| `sm` patched into the dependency and forgotten | Diagnostic only; promoting it needs `wishlist4` and a release plan |

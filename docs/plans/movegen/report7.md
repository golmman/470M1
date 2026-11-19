# Report 7 — eval geometry: one per-node pass (`M12`)

**Plan:** [`plan7.md`](plan7.md). **Item:** `M12` (new). **Decisions:**
`MD1`, `MD5`, `MD6`; no new decision row. **Date:** 2026-11-19.
**Session:** execution. **Tree:** `470MI@e4588cb` + `M12`, published with the
owner's commit. **Outcome:** `M12` adopted. `core` instructions fall
**0.1879e9** (−2.35%) with identical counters, mate and `.pvr`; the 10 s
geomean is **1.056** against a same-session plan 2 binary at **0.987**
(candidate/old 1.070, from 1.107). `MD6` is still not held, as pre-registered.
Measurements: [`measurements/plan7/`](measurements/plan7/README.md).

## Phase A — the pre-registered budget

The unchanged tree at `core` is **8.0040859e9** instructions (three
interleaved runs, spread 703) against the plan 2 binary's **6.5693769e9**
(spread 100): the 1.4347e9 gap is unchanged, so the session is comparable to
plan 6. The `Dfpn::evaluate` annotate excerpts in both binaries are recorded
in `measurements/plan7/perf-annotate.txt`; they are evidence for the next
decision, not an input to this one.

The counted build (`counts.patch`, applied temporarily, never committed) keeps
the search identical — `child_evals` 3,896,430, `expansions` 191,557, `nodes`
350,430, mate 14 — and gives the pre-registered table:

| Quantity | Pre-registered from the run |
| --- | ---: |
| `terms` calls / `evaluate` calls | 383,114 / 191,557 |
| Pieces per `evaluate` (mean) | **22.90** (4,386,775 total) |
| Defender role masks per `evaluate` (mean) | **4.34** (831,332 total) |
| Median `N − R` per `evaluate` / per `terms` call | **19 / 7** |
| Old check-mask calls (one per piece, both sides) | 4,386,775 |
| New check-mask calls (one per defender role) | 831,332 |
| Expected call reduction `N − R` | **3,555,443** |
| Measured fall per removed call | **52.8** instructions |
| Pre-registered fall | 0.07e9 ± 0.04e9 |
| Measured fall | **0.1879e9** |

The gate passed: `N − R` is 7 on the median `terms` call and 19 on the
`evaluate` formulation, both far above 1. The measured fall is **larger** than
the estimate, not smaller; `52.8` per removed `attacks::attacks` call includes
the material/role walk that leaves the per-piece loop as well, so it is not a
clean per-call cost.

## Phase B — `M12`, the change

`eval::evaluate(pos, mover)` replaces `terms(pos, us) + terms(pos, !us)`.
Material and attack terms are walked **by role** over the five non-king
roles, so `role_at`/`piece_at` leave the loop. The side to move uses the
`Mover`'s existing `check_squares` (one mask per role, `moves.rs`); the
defender's table is built once, only when its king exists and the kings do not
touch, with one `attacks::attacks` call per role present (≤5). The early
returns are unchanged: no king, or touching kings, returns material only. The
old per-piece formulation is `#[cfg(test)] terms_reference`; two unit tests
compare `Terms` and `evaluate` over every accepted differential-corpus FEN and
two seeded play-out sets. No rule, format, checker, search, ordering or seed
changed. `.pvr` bytes are identical on all three bench cells.

## Phase C — the falsifier

Three interleaved `perf stat -e instructions:u` pairs on `core` (seed 0, 10 s,
192 MB, no export):

| Run | Unchanged HEAD | `M12` | Delta |
| --- | ---: | ---: | ---: |
| 1 | 8,004,085,929 | 7,816,219,316 | −187,866,613 |
| 2 | 8,004,085,692 | 7,816,219,017 | −187,866,675 |
| 3 | 8,004,086,129 | 7,816,219,109 | −187,867,020 |

Spreads 437 and 299; the median fall **0.1879e9** is outside both and above
the **0.03e9** adoption floor. Counters and mate are identical in every run;
median wall 588 -> 564 ms. Adopted.

## Phase D — gates and the `MD6` row

Tree-changing session, so exactly one ledger row is appended. Bench: plan 3's
driver, protocol `a`, seed 0, 10 s, 192 MB, three repeats, new binary and the
same-session plan 2 binary.

| Cell | New factor | Old factor | Mate / ref |
| --- | ---: | ---: | ---: |
| `core` | **1.200** | 1.031 | 14 / 17 |
| `easy-win` | **1.096** | 0.978 | 18 / 15 |
| `deep-win` | **0.895** | 0.955 | 20 / 22 |
| **Geomean** | **1.056** | **0.987** | |

Coverage 3/3, every proof accepted by our checker and `provari check` (9/9),
`D14` counters identical. Candidate/old **1.070**; `MD6` is **not held**. The
reference reads 0.987 against its 0.972 anchor, inside the usual band.

Gates all green: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`,
`cargo test` (50 unit + integration, 2 ignored slow), `cargo doc --no-deps`,
`make test`, `make test-full`, `make veto` (all fixtures accepted, all controls
rejected), and
`cargo test --release --test differential rules_differential_full -- --ignored`,
which reproduces plan 5's counts to the digit: 84/11 FEN, 27,088 positions,
540,383 legal moves and child keys, 72,131 explosions, 642 material
exemptions, 392 games / 26,612 plies, **0** mismatches, **0** key mismatches,
`MD3` 87/0/0.

## Distance to the goal

`core` is now **7.816e9 vs 6.569e9**, i.e. **1.190×** (from 1.218×); the
remaining gap is **1.247e9**. The rules layer is still **0.62e9 (50%)** and
unchanged. `M12` removes most of the eval-geometry line: what remains of
plan 6's geometry bucket is roughly 0.17e9 (`Mover::new` 558 per node and the
surviving reach calls), plus the shared `attackers_to` boundary. Search/table
0.25e9 (`evaluate` +0.1045e9, +25 per child on the same tree), generation
0.13e9, tree volume 0.12e9 and play/memmove 0.06e9 are untouched.

## Problems, scope changes and what was not done

- The measured fall exceeds the pre-registered range; no tuning was done to
  land inside it. The per-call attribution is a lower bound on what left the
  loop, because the by-role material walk also removed `role_at` traffic.
- `eval::evaluate` and `eval::terms` changed shape; `material` and the public
  `Terms` fields are unchanged, and no external consumer existed.
- No rules, format, checker, search or scoring change; no crate API, no
  `wishlist3`, no fourth release. One formulation change this session.
- The no-op `evaluate` growth and the inlining boundary remain unowned by this
  plan; they were only annotated in Phase A.

## Next steps and options

1. **`M6`/dated acceptance plus a rules-layer decision (recommended).** `M12`
   is the last measured prover-owned formulation lever; the remaining 1.19×
   is half rules. The cheapest decisive experiment is a fourth
   `atomic-movegen` legality/effect release measured on `core` before any
   search change, since the audit's ceiling (~1.12×) leaves a residue even
   then.
2. **Attack the search/table line next.** `Dfpn::evaluate` is +25 per child on
   an unchanged tree (0.1045e9); the annotate excerpts are the starting point,
   but the plan 6 lesson says to test whether it is crate-inlined boundary
   cost before writing prover code.
3. **Accept the residual as a dated row.** Record `M12` as the final
   prover-side gain (1.190×) and stop the initiative with the `MD6` miss
   documented.

Option 1's kickoff prompt:

> Read `docs/plans/movegen/initiative.md`, `report5.md`, `report6.md`,
> `report7.md` and `measurements/plan7/`. Plan 8: decide between a fourth
> `atomic-movegen` legality/effect release and a dated `MD6` acceptance. The
> rules layer is still 0.62e9 of the 1.247e9 `core` gap; the eval geometry is
> now spent. Measure the cheapest decisive experiment on `core` first: a
> per-call legality/effect probe against the plan 2 binary, and only then a
> plan or an acceptance row.

# Report 8 — rules endgame: the ladder is correct and slower

**Plan:** [`plan8.md`](plan8.md). **Item:** `M13` (new). **Decisions:**
`MD1`, `MD5`, `MD6`; a dated `MD7` row on the acceptance branch.
**Date:** 2026-11-19. **Session:** execution. **Tree:** `470MI@bb0d125`,
product tree unchanged; docs only. **Outcome:** **acceptance**. The exposed
`FastLegality` ladder is oracle-equivalent but costs **+0.1641e9** instructions
on `core` in its best shape, against a 0.31e9 release threshold. No fourth
release, no `wishlist3`; `MD7` records the dated miss. Measurements:
[`measurements/plan8/`](measurements/plan8/README.md).

## Phase A — the fixed inputs and the decisive screen

The unchanged tree reads **7.8162193e9** (spread 668) against plan 2's
**6.5693771e9** (spread 323), a **1.2468422e9** gap — the plan's fixed input,
reproduced. Counters are `child_evals` 3,896,430, `expansions` 191,557,
`nodes` 350,430, mate 14.

The ten-FEN screen (2,000 passes, 496,000 calls per operation; the plan's
49,600 is the per-FEN mean) measures:

| Operation | Instructions/call |
| --- | ---: |
| `V0` `attackers_to` + `by_role` | 65.7 |
| `VB` `LegalityContext::legal` oracle | 152.3 |
| `V1` `FastLegality` ladder | 72.0 |
| `V3` shipped `ChildEffect` | 213.8 |

`V1` is only 6.3 above `V0`, so the cheap-stop condition `V1 > V0 + 10` did
not fire and Phase B ran. `V2` (a legality entry point at plan-2 occupancy
cost) was not built: `V0` is its floor, and `V1` is already only 6.3 above that
floor, so no V2 shape can undercut the measured ladder by more than that
margin. `V3` is the shipped parent-bitboard shape; no further V3 API exists in
2.10.0, and the only candidate reduction is the node-snapshot shape plan 6
falsified (`NodeFacts`, +0.038e9). V3 was therefore not selected, as the plan
permits.

**The census was the decisive screen.** In a scratch copy wired to
`FastLegality::legal_traced`, counters, mate and `.pvr` are identical. Of
15,171,649 `Mover::legal` calls (plan 6's entry count to the digit), 5,159,465
take the prover's own fast path; the 10,012,184 fallback candidates split:

| Site / stage | Count |
| --- | ---: |
| in check | **8,220,516** (82.0%) |
| capture | 764,200 |
| king | 915,488 |
| quiet (pinned) | 111,980 |
| ladder trivial | 0 |
| widened `Some(true)` | 1,449,797 |
| widened `Some(false)` | 106,811 |
| widened `None` → oracle | 235,060 |
| in check → oracle | 8,220,516 |

The ladder decides only **1,556,608** calls (15.5%). The pre-registered
projection, using the screen's ladder average as the rung cost, is
`1,556,608 × (152.3 − 72.0)` = **0.125e9**; charging the failed widened
attempts gives **0.108e9**, and scaling the oracle to the whole-binary 120.9
gives **0.099e9**. These are optimistic because the ladder average is measured
on the full candidate stream, not the fallback population; every reading is
below the 0.15e9 stop threshold, and Phase B is the decision input.

## Phase B — the whole-binary falsifier

Three V1 shapes were built in `/tmp/plan8-crate` + `/tmp/plan8-proto-tree` and
measured on `core`, three interleaved `perf stat -e instructions:u` pairs each
against the unchanged binary, seed 0, 10 s, 192 MB, no export:

| V1 shape | Median instructions | Delta |
| --- | ---: | ---: |
| ladder, full `StateInfo` built-in oracle | 8,034,301,477 | **+218,082,159** |
| ladder, counts-only built-in oracle | 8,106,264,837 | **+290,045,519** |
| ladder `decide_or_none` + `LegalityContext` oracle | 7,980,337,054 | **+164,117,736** |

All three are regressions. Counters and mate are identical in every run, and
the `.pvr` is byte-identical (`714723211c7a0246…` on `core`); the ladder is
correct, as its in-crate tests claim. The best shape still pays the trivial
rung and the widened attempt on every fallback candidate, including the 82%
that are in check and can only defer; the larger `Mover` also costs per node.
The census projection was optimistic because it charged the ladder only on
decided calls and ignored the failed attempts on the oracle population.

## Phase C — decision

The pre-registered rule: release only if the measured `core` fall is
**≥ 0.31e9** with identical counters, mate and `.pvr`; anything else is
acceptance. The best measured fall is **−0.1641e9**, so the branch is
**acceptance**. `MD7` records: the split (rules 0.617e9, geometry 0.15–0.36e9,
generation 0.1255e9, tree 0.1249e9, search/table 0.2505e9), the audit's ≈1.12×
ceiling, `M12` as the last measured prover-side gain, and this plan's number.
`M1`'s performance part and `M6` close as a documented miss; `M8` is the only
open item. Exactly one of `wishlist3.md` / the `MD7` row was written: the row.

## Phase D — artefacts and gates

`measurements/plan8/` holds the README, `core-perf.txt` (baseline and all
prototype pairs), the probe source/driver/results, `census.json`,
`results.json`, `env.json`, `decision.json`, `w10.patch`, `v1-hybrid.patch`,
`census.patch` and `census-crate.patch`. No binary, `perf.data`, transcript or
oracle artefact is committed. `initiative.md` gained `M13`, one status addition and one history
line; `AGENTS.md`'s current-state paragraph is updated.

`make test` (fmt, clippy `-D warnings`, licence, unit/integration tests) was
green **before and after**. The product tree is unchanged: no edit to `src/`,
`Cargo.toml`, `Cargo.lock`, `PROVENANCE.md` or `check.rs`, no pin, no ledger
row. The release-branch gate set (`cargo clippy --all-targets`, `cargo doc`,
`make veto`, the ignored full differential) was not run because no release
patch was kept; the scratch prototype was only a decision instrument.

## Problems, scope changes and what was not done

- The plan's screen wording "49,600 calls per operation" is the per-FEN mean;
  the plan-5 precedent and this probe report the ten-FEN total, 496,000. The
  source is identical; only the denominator label changed.
- The first two prototype shapes used `FastLegality`'s built-in
  `legal_in_view` oracle, which is slower than the prover's
  `LegalityContext`. The third, hybrid shape kept the original oracle and is
  the fairest test of the ladder's marginal prize; it is still a regression.
  This is why the decision does not rest on the projection alone.
- V3 was not built. `ChildEffect` already is W8's parent-bitboard shape, and
  the only further idea in reach — a node snapshot of the parent's pieces and
  key — is the prover-side `NodeFacts` formulation falsified in plan 6. The
  plan allows V3 only if Phase A selects it; it did not.
- No bench geomean: the tree did not change, so there is no candidate row and
  no ledger entry (`MD6` remains the same-session rule for future adoptions).
- No new unit test was added: there is no product code change to guard. The
  scratch crate's existing ladder-vs-oracle tests and the byte-identical
  `.pvr` on `core` are the correctness evidence for the instrument.

## Distance to the goal

`core` is **7.816e9 vs 6.569e9**, **1.190×**, gap **1.2468e9**; the rules layer
is **0.617e9** (49.5%) of it. The audit's crate-led ceiling is ≈1.12×, and a
perfect rules close still lands near 1.035 candidate/old against a reference
reading 0.987. `M12` is the last measured prover-side gain. Performance is now
closed as a documented `MD6`/`MD7` miss; `M8` (complete shakmaty removal from
the parser/display/proof path) is the only open item.

## Next steps and options

1. **`M8`, shakmaty removal (recommended).** No rules or performance change:
   move FEN parsing, display/proof paths and line output off shakmaty while
   the checker stays shakmaty-only. The differential and veto remain the gate;
   the bench must not regress.
2. **A new measured performance lever outside the rules layer.** The remaining
   non-rules buckets are search/table 0.2505e9, tree 0.1249e9 and generation
   0.1255e9. This needs a new plan and a pre-registered whole-binary rule; the
   plan 6/7 lesson is to test the crate-inlining boundary before writing prover
   code.
3. **Re-open the crate only with a new shape.** A release is justified only if
   a prototype clears 0.31e9 on `core`; repeating the ladder exposure is not.

Option 1's kickoff prompt:

> Read `docs/plans/movegen/initiative.md`, `report8.md`, `decisions.md` (`MD7`)
> and `rules-surface.md`. Plan 9: `M8`, complete shakmaty removal from the
> prover's parser, display, proof and line paths; the checker stays shakmaty.
> Keep the differential at zero mismatches, run `make test`, `make test-full`
> and `make veto`, and measure the 10 s bench with no regression.

# Report 3 — P2 parity gate

**Plan:** [`plan3.md`](plan3.md). **Item:** 3. **Date:** 2026-11-17.
**Outcome:** gates green; `P4` reached; `P5` met on the 10 s protocol with a
margin inside the session's noise band (geomean 0.983 median over four
passes, range 0.964–1.013). No lever adopted: the measurement justifies none.
One ledger row appended at 10 s.

## Rung and absolute distance

| Rung | Result |
| --- | --- |
| `P4` (all three at 10 s, both checkers) | **reached.** 9/9 per cell, every proof accepted, `D14` counters identical |
| `P5` (`P4` + geomean `< 1.0`) | **met on 3 of 4 same-session passes**, session median **0.983** |

Last pass (protocol a): `core` 1.062, `easy-win` 0.988, `deep-win` 0.854,
geomean 0.964. The four passes gave 0.992 / 1.013 / 0.974 / 0.964. The
reference's own wall moved ~10% between passes while the ratio moved ~5%, so a
1–2% margin is **not resolvable by this instrument**. The honest statement of
the absolute distance: `deep-win` is 10–15% ahead, `easy-win` is at parity,
and `core` is 2–9% behind — on the geomean the goal's letter is met, the
per-cell intent only for two of three.

Where the `core` distance comes from — measured, not guessed:

1. **Not per-node.** At seed 0 the candidate's tree is node-for-node provari's
   (342,289 nodes) and its wall is 0.352 s against 0.389 s search-only: 8–10%
   *cheaper* per node.
2. **The seed protocol's tree.** Seeds 1 and 2 inflate `core`'s tree by 10%
   and 17% (376,527 / 401,891 nodes); the 9-run median is a seed-1 wall. The
   seed perturbs the guide's step by ±1%, which scales every child of a
   position, so it reorders subtrees rather than tie-breaks. D8 says the seed
   perturbs only tie-breaks. Not changed here: that would be a bench-driven
   default change, and it belongs to a kill review.
3. **The export.** Writing the proof costs 26 ms on `core` (8% of the wall)
   and 26 ms on `deep-win`; the ledger protocol charges it to the candidate
   and not to the reference. With both sides exporting (variant b) the
   geomean is 0.988; with neither (variant c) it is 0.918.

## What landed

- [`measurements/plan3/run_bench.py`](measurements/plan3/run_bench.py) — one
  interleaved pass measuring three protocol variants (a: candidate exports,
  b: both export, c: neither), run order rotated each repeat, per-seed
  medians, and an optional `D14` untimed-export check. It fixes a reporting
  bug of the plan2 driver: the reported counters came from a different seed
  than the reported wall. It now records `candidate_at_median`.
- Four 10 s passes and one 120 s pass of parsed results, `env-10s.json` and
  `env-120s.json` under `measurements/plan3/`.
- The ledger row at 10 s, and the plan's diagnosis in
  [`measurements/plan3/README.md`](measurements/plan3/README.md).
- No prover, table, proof or checker source change.

## Levers, and why none was adopted

| Lever | Evidence | Decision |
| --- | --- | --- |
| HOTPATH-V2 (guide closures) | per-node cost already ≤ provari's on an identical tree; `perf` profiles agree within a point | **closed.** No measured gap to close |
| NULL-PIPE (`null_win` pipelining) | same; it is a per-node lever | **closed.** Same reason |
| Proof-retention share | `deep-win` needs a 19-move proof at 144 MB of proofs (38.2 M evals) vs 22 at 64 MB (58.2 M), but a full sweep (64…160 MB) shows proofs never fill and the wall is non-monotone (4.00–4.74 s) as the **tree** moves with the table size alone | **not adopted.** Fits one cell's trajectory lottery |
| Guide / EF-PN tuning | no held-out set in this plan | **not attempted** |

The 120 s cap shows the same split effect benignly: the cap raises
`proofs_needed` to 144 MB, `deep-win` finds the 19-move proof, and the row
reads 0.831 on that cell (geomean 0.975). That is a memory split, not a
mechanism.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --check` | green |
| `cargo clippy --all-targets -- -D warnings` | green |
| `cargo test` | 44 tests green |
| `make test` (lint, licence, tests) | green |
| `make test-full` | green |
| `make veto` | 9/9 fixtures accepted, 1/1 control rejected by both checkers |
| `cargo doc --no-deps` | green |
| `D14` timed vs untimed counters | identical for 9/9 cell-seed pairs at 10 s and 120 s |
| Claim protocol | every candidate proof accepted by `470m1 check` and `provari check` (9/9 per cell per pass) |

## Measurements summary

| Cap | Protocol | Coverage | Geomean | Note |
| --- | --- | --- | ---: | --- |
| 10 s | a (ledger) | 3/3 | 0.964 | last of four passes; median 0.983 |
| 10 s | b (both export) | 3/3 | 0.988 | same pass |
| 10 s | c (search only) | 3/3 | 0.918 | same pass; claims from the twin runs |
| 120 s | a | 3/3 | 0.975 | 144 MB proofs by the cap's split |

Per-node on identical trees: 1.122 / 1.172 / 1.002 µs per node against
provari's 1.136 / 1.186 / 1.151. Child evals at 10 s (seed 0): 3,823,765 /
897,937 / 58,229,418.

## Problems and findings

- **The 10 s measurement is resolution-limited.** 70–400 ms walls carry ~1 ms
  of process start and ~4.5 ms of memory zero-fill; the reference's own wall
  drifts ~10% across passes. A verdict at the 2% level needs either a much
  larger sample or a code change with a larger effect.
- **The plan2 driver's summary mixed seeds.** It reported the median wall over
  nine runs with the counters of seed 0. The factor was right; the
  accompaniment was not. Fixed in this plan's driver.
- **A `--seconds` cap selects the memory split**, so the same search at 10 s
  and 120 s has a different tree (and `core` differs by one node). This is
  provari's `proofs_needed` design, copied in plan2; it is sound (same flags ⇒
  same counters) but it makes the cap a search parameter, not only a budget.
- **The reference's export is free under the ledger protocol.** Protocol b
  removes that asymmetry and reads 0.988; protocol c reads 0.918.
- **`refuted` output shape** (terminal draws) is still the plan2 difference;
  unchanged, not a soundness issue.

## Missing tests / open parts

- No test pins the seed's tree effect (a `core`-seed-1 counter fixture would
  catch a guide change cheaply).
- No held-out set for guide tuning; no profile of `deep-win`.
- The export is still the walker copied in plan2; its 26 ms is measured but
  not reduced.

## Next steps

1. **Kill review** (after plan 3, as the process requires): is a 0.98 geomean
   with `core` behind by up to 9% "parity"? Which is worth more — closing
   `core` or opening `movegen`?
2. Cheapest decisive experiment if margin is wanted: rewrite `format::encode`'s
   walk to use the search's own `Mover`/`Effect` instead of `play` +
   `in_check` per legal move (~12 of the 26 ms on `core`) and to avoid the
   second `legal_moves` pass. Expected ~4% on `core`, gate: proofs unchanged
   for every fixture and every bench proof accepted by both checkers.
3. Second cheapest: make the seed perturb only tie-breaks as D8 says, and
   re-measure; expected removal of the 10–17% `core` tree inflation, but this
   is a default change with a bench-driven motive and needs the review's
   sanction.

# Initiative: `parity` — a provari-parity atomic win prover and checker

## Status

**Plan 4 complete (2026-11-17).** `P4` remains reached at 10 s: all three
cells proved, 9/9 seeds/repeats, every proof accepted by both checkers, and
timed and untimed counters identical. `P5` is met on the **product default**
(seed 0; kill review 1 read 0.947, this session's seed-0 runs read 0.965 with
all three cells below 1.0). The D8 defect is fixed: a seed now perturbs only
equal-scored children, never a guide step's magnitude, and seed 0 is
unchanged. The corrected three-seed 10 s row is 1.016 because tie-break-only
seeding still reorders `core`'s many equally scored children (seed 0 0.965,
seed 1 1.147, seed 2 1.055); it is recorded as a regression row, not a speed
claim. Error handling, flag validation, exit codes, help, the ignored
bench-cell test and the docs gates are closed. Next: the owner's `movegen`
handoff decision. See [`report4.md`](report4.md).

## Reference

This initiative starts from provari's architecture and proof format. The
repository is GPL-3.0-or-later, so provari code may be copied or adapted with
provenance. `atomi` and `atomic_solver` are MIT and may also be copied or
adapted with provenance.

| Item | Value |
| --- | --- |
| Repository | `470M1` (this repo) |
| Package / binary | `470m1` (working name; rename is an owner decision) |
| Initiative directory | `docs/plans/parity/` |
| Parity reference | `aiorla/provari` `bceb1ee1faf7d9cf2422eb89c931be69ae30619d` (`v0.4.0`), GPL-3.0-or-later |
| Idea/measured-lever source | `golmman/atomi` `a4802e7cbe7ea6338e99dd21d3b84dbd6f871f76`, MIT |
| Fixture/bench source | `golmman/atomic_solver` `14abcf27ae34ec71f46d8bc82f300d44e02f5f57`, MIT |
| Rules dependency (parity) | `shakmaty` 0.30.1 with the `variant` feature |
| Checker dependency | `shakmaty` only; no prover code |
| Corpus | `corpus-v1`, sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e` (to be copied in plan1) |
| Bench ledger | [`../../bench.md`](../../bench.md) |

## Goal

**Be at least as fast as provari on the fair, reproducible shared-win bench,
with every claimed win checked by our own independent checker and by
`provari check`.**

Success condition (`P5`): all three bench cells proved at 10 s, geomean wall
`<= 1.0` against `provari --guide hand`, every claim accepted by our checker
and by `provari check`.

The goal is the top rung of a chain of necessary conditions, each a subset of
the next (`P5 ⊆ P4 ⊆ ... ⊆ P1`), so `goal => every rung` and a failed rung is
a hard "not yet":

| Rung | Requirement | Implied by the next |
| --- | --- | --- |
| `P1` | `core` proved at 120 s | yes |
| `P2` | `core` + `easy-win` at 120 s | yes (more cells) |
| `P3` | all three at 120 s | yes (tighter budget) |
| `P4` | all three at 10 s | yes (wall clause) |
| `P5` | all three at 10 s **and** geomean `< 1.0` | — (the goal) |

Nesting is sound because the search is deterministic and `--seconds` only
stops it, so proving at 10 s implies proving at 120 s; more cells is monotone
by definition. The wall clause lives only at `P5` because `geomean < 1` does
not imply each cell `< 1`. A lower rung is necessary, never sufficient; a
censored cell is never converted and a longer run is never substituted.

The goal, the rungs and their budgets are stated here and nowhere else.
[`../../bench.md`](../../bench.md) is the results ledger: it records what was
measured and the cap each row used, and makes no goal claim.

## Non-goals

- **No web frontend.** No WebAssembly, browser worker, page or web build.
- **No UCI engine.** The CLI is `prove`, `check`, `line`, `bench`; a UCI
  wrapper is out of scope.
- **No tablebases.** No Syzygy, no `.atbw` files, no tablebase verdicts in
  proofs. A tablebase integration is a separate future initiative.
- **No NNUE or learned model.** The hand guide is the product. provari's
  network/model columns are not parity targets.
- **No movegen replacement in this initiative.** The prover uses shakmaty for
  parity. Replacing it with `atomic-movegen` is the separate
  [`movegen`](../movegen/initiative.md) initiative, started only after `P5`.
- **No new search formulation.** Parity is DF-PN with proof/disproof numbers,
  as provari does it. PDS-PN/PN², restarts, portfolios and parallel search are
  out of scope unless a kill review authorises a separate item.
- **No draw or loss proofs.** The engine proves attacker wins or reports no
  win found (refuted or censored).
- **No proof database or durable corpus.** Proofs are files; a database is a
  future initiative.
- **No weakening of soundness.** Repetition and the 50-move limit are never
  wins. A faster false win is a failure.

## Working agreement

1. **Measure before planning.** Every plan starts from the committed baseline
   and read-only probes. Sizing spikes are temporary and reverted.
2. **Stack-level gating.** A plan may add a named stack of coupled mechanisms
   under one gate, stated up front. Mechanism-level adopt/reject decisions
   come from both-direction ablations after the stack gate, not from
   single-lever runs before it.
3. **Correctness gates are vetoes.** `make test` runs on every change.
   `make test-full` (including the oracle veto) runs for search, table,
   proof-output and checker changes. A bench row or speed claim is written
   only after the gate passes.
4. **Proof claims.** A win is *claimed* only after the emitted proof passed
   our independent checker **and** `provari check`. Every non-accept is
   classified as `accepted`, `checker-reject`, `unchecked` (checker says
   `valid: null`), `export-failure` or `rules-divergence`. Only `accepted`
   counts.
5. **Deterministic budget contract.** The same binary, FEN and flags give
   identical counters. Collecting a proof never changes the search
   trajectory.
6. **Oracle isolation.** provari is built from source at the pinned commit in
   a scratch directory outside the repository. If the oracle is unavailable,
   the result is `INCONCLUSIVE`; it is never substituted or skipped.
7. **Copy with provenance.** Every copied file or fragment carries a
   provenance line and a row in [`../../../PROVENANCE.md`](../../../PROVENANCE.md).
   After the copy this repository owns it; re-copies are deliberate changes.
8. **Same-session measurement.** Factors are same-host, same-session ratios.
   The reading rules in [`../../bench.md`](../../bench.md) apply.
9. **Negative results are recorded.** A mechanism that fails its gate closes
   with the evidence. Stop rules are honoured.
10. **Kill review after every 2 plans.** The review asks for the absolute
    distance to the goal, whether we would build this today, which plan or
    initiative to cancel, a delta prior-art search, and the smallest next
    experiment.

## Decisions

One-line resolutions; full text and rationale in [`decisions.md`](decisions.md).

| # | Decision | Resolution |
| --- | --- | --- |
| D1 | Licence and copying | GPL-3.0-or-later; provari (GPL) and atomi/atomic_solver (MIT) may be copied/adapted with provenance |
| D2 | Parity reference | provari `v0.4.0` at `bceb1ee`; the reference is re-pinned only by an owner decision |
| D3 | Rules dependency | shakmaty 0.30.1 for parity; `atomic-movegen` is the movegen initiative |
| D4 | Checker independence | `check` uses shakmaty only and shares no code with the prover |
| D5 | Proof format | provari compact `.pvr` v1 plus JSONL; no native format |
| D6 | Bench set and metric | three cells from `corpus-v1`; factor = candidate wall / provari hand wall; 10 s/192 MB; 3 repeats |
| D7 | Scope exclusions | no web, UCI, tablebases, NNUE/model, portfolio, draw/loss proofs |
| D8 | Determinism | same binary + FEN + flags = same counters; proof collection is trajectory-neutral |
| D9 | Claim protocol | own checker + `provari check`; a disagreement is a veto |
| D10 | Provenance | every copied file/fragment has a `PROVENANCE.md` row and a source pin |
| D11 | Movegen handoff | `movegen` starts only after `P5`; the checker stays on shakmaty |
| D12 | Documentation process | initiative is normative; plans/reports are historical; bench is an initiative-independent ledger |

## Backlog

Items are stable semantic identities. Plans are numbered sequentially and
independently of items. A plan header names its item.

### Active and proposed items

| # | Item | Status / next action |
| --- | --- | --- |
| 1 | **P0 — foundation** (plan1) | **done.** Crate, CLI skeleton, rules, proof format, independent checker, bench harness, oracle, corpus. Gate green; see [`report1.md`](report1.md) |
| 2 | **P1 — search core** (plan2) | **done.** Board/moves/effect, table, proofs, DF-PN, hand guide, proof writer, `prove`. Gate: `P3` at 120 s (all three cells, 9/9, both checkers); 10 s candidate matrix geomean 0.968. See [`report2.md`](report2.md) |
| 3 | **P2 — parity gate** (plan3) | **done.** `P4` reached; `P5` met on the protocol with a noise-band margin (median geomean 0.983). No lever adopted; levers closed with evidence. See [`report3.md`](report3.md) |
| 4 | **P3 — hardening** (plan4) | **done.** D8 tie-break-only seed, no user-triggerable CLI panic, flag validation and exit codes, ignored 10 s bench-cell test, licence/docs gates. Corrected seed row 1.016 is a regression, not a speed claim. See [`report4.md`](report4.md) |
| 5 | **Movegen handoff** | **next.** Close `parity` and open `movegen`; owner decision after plan 4 |

### Closed items

| # | Item | Outcome |
| --- | --- | --- |
| 6 | Parity margin (optional) | **cancelled** by kill review 1: the product default already met `P5`, so there was no product margin to buy. The D8 seed correction landed as item 4 instead |

### Stop rules

- If plan1's checker cannot accept a known provari proof, stop and fix the
  checker before any search work.
- If plan2 cannot prove `core` at 120 s, do not start plan3; diagnose the
  search core first.
- If a plan's gate fails, record the negative result and take the
  continue/re-scope/close decision at the next kill review. Do not silently
  raise a cap or drop a cell.

## Source take-list

| Source | Item | Mark | Milestone |
| --- | --- | --- | --- |
| provari | board, move/effect precomputation, hashes equal to shakmaty | copy/adapt | P0/P1 |
| provari | 4-slot TT clusters, filter, least-work replacement | copy/adapt | P1 |
| provari | full-key proof table, `(move, length, limit)` | copy/adapt | P1 |
| provari | DF-PN, 1+ε, hand guide, move classes, EF-PN, simulation, null replay, history | copy/adapt | P1 |
| provari | compact `.pvr` format and JSONL | copy/adapt | P0 |
| provari | independent checker (`check.rs`) | copy/adapt | P0 |
| provari | proof walk, line display, bench command | copy/adapt | P0/P1 |
| atomi | bench ledger structure, decisions/gotchas/history, planning process | idea/spec | P0 |
| atomi | measured levers: HOTPATH-V2, NULL-PIPE, proof share, guide tuning | idea | P2 |
| atomic_solver | corpus, fixtures, bench driver, test discipline | copy/adapt | P0 |

## Corpus, fixtures and measurement conventions

- **Primary column:** `provari prove --guide hand`.
- **Bench cells:** `core`, `easy-win`, `deep-win`; FENs, ids and provisional
  reference values are in [`../../bench.md`](../../bench.md).
- **Corpus:** `bench/corpus.json`, a copy of `corpus-v1`, sha256
  `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`. The
  bench driver refuses a mismatch.
- **Budgets:** 10 s and a 192 MB label on both sides. provari:
  `prove "<FEN>" --seconds 10 --memory 192 --guide hand`. Candidate:
  `prove "<FEN>" --seconds 10 --memory 192`.
- **Repeats:** three per cell per side for any reported wall ratio.
- **Fixture oracle:** known side-to-move wins copied from the legacy fixtures.
  A refutation there is a bug to investigate. Censoring is a result. Legacy
  side-to-move losses are kept as never-prove negatives: proving one is a
  soundness failure.
- **Ledger:** [`../../bench.md`](../../bench.md). Every plan that touches the
  search appends one candidate row. A row records the cap it was measured at;
  `censored` means "not claimed at that cap", a longer run is its own row, and
  geomeans are compared only within one cap.
- **Artifacts:** drivers, parsed results, `env.json` and provenance READMEs go
  under `measurements/planN/`. Raw transcripts, oracle sources/binaries/proofs
  and `__pycache__` are never committed.

## Risks

| Risk | Mitigation |
| --- | --- |
| Licence contamination | D1/D10; licence gate; provenance rows; keep upstream notices |
| Checker shares a bug with the prover | D4: checker uses shakmaty only and shares no code; `provari check` veto |
| Rules divergence between shakmaty and provari | Checker veto; classify as `rules-divergence`; report upstream when real |
| Repetition identity mismatch (en passant) | Board-only key with `EnPassantMode::Legal`; checker compares exact positions |
| Proof length/limit reuse produces a cycle or 50-move failure | Length/limit reuse rules; writer fails closed; both checkers |
| Bench cap shopping | D6 and bench reading rules: a row records its cap; geomeans only within one cap |
| Measurement drift | Same-session ratios; re-measured reference; reading rules |
| Scope creep | Non-goals; named stacks with one gate; kill reviews |
| The port is slower than provari | Plan3 measures and ports atomi's measured levers; if parity is not reachable, the kill review decides |
| Movegen work starts too early | D11: `movegen` starts only after `P5` |

## History

The chronological record is in [`history.md`](history.md). Details live in the
plan and report files.

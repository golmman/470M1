# Plan 8 — rules endgame: expose the legality ladder, or accept the dated miss

**Item:** new `M13` (rules-layer endgame decision). **Decisions:** `MD1`,
`MD5`, `MD6`; a dated `MD7` row only on the acceptance branch. **Predecessors:**
[`report7.md`](report7.md), [`report6.md`](report6.md),
[`hot-path-audit.md`](hot-path-audit.md). **Report:** `report8.md`.
**Budget:** one sitting (~5 h): A 1.5 h, B 2 h, C 1 h, D 0.5 h.

## Goal

Decide on `core` whether a fourth `atomic-movegen` release is worth an
owner-triggered cycle, or whether the rules residual is accepted as a dated
`MD6` miss. Deliverables: per-call screen, **coverage census**, whole-binary
prototype, decision, and either `wishlist3.md` or the acceptance row. **No
prover source change, no pin change, no release in this session.**

## Fixed inputs (do not re-litigate)

- `core` **7.8162e9** vs plan 2's **6.5694e9** (1.190×); gap **1.2468e9**,
  rules **0.617e9** (49.5%).
- Per run: legality **10,012,184** @ **120.9**; effect **4,779,358** @
  **132.3**; `Mover::new` 578,396 @ 558.4. Plan-2 witnesses: legality **83**,
  effect+key **50–75**. The ten-FEN probe reads higher (effect 215.2), so it is
  a screen only; decisions use whole-binary numbers.
- **No outcome holds `MD6`.** Wall elasticity over the measured pair is
  0.070/0.190 per unit instruction ratio. Closing the *entire* 0.617e9 rules
  delta lands ≈ **1.035** candidate/old (`core` 7.199e9) against a reference
  reading 0.987; rules + generation + aligned geometry floors near 1.05. The
  release can only narrow the miss; a report may not claim otherwise.
- Off-limits: search, table, eval values, tree, `.pvr`, `check.rs`, prover-side
  legality (`MD5`), fork or path dependency. The prototype is a `/tmp`
  diagnostic bound, never committed or referenced from `src/`.

## Plan-session finding (read-only)

2.10.0 already contains a cheap legality ladder: `FastLegality`
(`src/board.rs:4089`) = node-constant `TrivialLegality` + lazily built
`WidenedLegality` + `legal_in_view` as the oracle. It is `pub(crate)`; the
crate's legal generators use it (`movegen.rs:363/440/538`) and it is tested
against the oracle in-crate (`board.rs` ~5900–5940, 6180). The prover calls
only `LegalityContext::legal`, i.e. the oracle, on the 10.0M candidates its own
one-rung fast path cannot settle (`moves.rs:legal`: quiet, unpinned, non-king).
The marginal prize is `WidenedLegality`'s coverage of **captures, commoner
moves and pinned pieces**. The smallest fourth release is *exposing what the
crate already trusts*, not a new rules body: `V1`.

## Options and pre-registered estimates

| # | Option | Est. fall on `core` |
| --- | --- | ---: |
| `V1` | expose `FastLegality` + pseudo-legal precondition (**formulation change**) | 0.20–0.60e9 |
| `V2` | `W7`: legality at plan-2 occupancy cost (shape) | 0.30–0.40e9 |
| `V3` | `W8`: effect + key from parent bitboards (shape) | 0.40e9 |

`V1` and `V2` are alternatives; the cheaper measured call wins, `V3` is
additive. Rejected: switch the prover to `generate_legal_with_state` — it
changes move list, ordering and proof contract, a search change in disguise.

## Phase A — baseline, screen and census (~1.5 h)

1. **Baseline.** Rebuild the unchanged tree; three interleaved
   `perf stat -e instructions:u` runs of the candidate and plan 2 (`2f629fd`)
   on `core` (seed 0, 10 s, 192 MB, no export); record pairs and spreads.
2. **Per-call screen** (`/tmp/plan8-probe`, plan-5/6 precedent): ten FENs,
   2,000 passes, 49,600 calls per operation: `V0` (`attackers_to`+`by_role`),
   `VB` (`LegalityContext::legal`), `V1` (the ladder), `V3` vs `ChildEffect`.
3. **Coverage census — the decisive screen.** In a scratch copy of this repo
   built against the scratch crate, wire `Mover::legal`'s fallback to the
   ladder and count over a `core` run: `trivial`, `decide = Some(true)`,
   `Some(false)`, `None` (oracle), by site (capture / quiet / king / in-check).
   Counters must be identical (`child_evals` 3,896,430, `expansions` 191,557,
   `nodes` 350,430, mate 14) — the ladder is oracle-equivalent by contract, so
   any movement is a stop. Projection: `Σ count × (cost_oracle − cost_rung)`.

**Cheap stop rule.** If the census projects **< 0.15e9** for `V1` and the
screen shows `V1 > V0 + 10` per legality call and `V3 > V0 + 15` per effect
call, write the acceptance and stop.

## Phase B — whole-binary prototype (~2 h)

In `/tmp/plan8-crate` (copy of `atomic-movegen` 2.10.0) make `FastLegality`
public, add the documented pseudo-legal precondition, and add the `V3` pass
only if Phase A selected it. Build a scratch copy of this repo against it and
wire `Mover::legal` / `effect` **there only**. Run `core` three interleaved
`perf stat -e instructions:u` pairs against the unchanged binary.

Pre-registered: identical counters, mate and byte-identical `.pvr`, else stop.
The measured whole-binary fall, not the projection, is the decision input.

## Phase C — decision (~1 h)

Pre-registered before Phase B runs:

| Branch | Rule |
| --- | --- |
| **Release** | measured `core` fall **≥ 0.31e9** with identical counters/mate/`.pvr` |
| **Acceptance** | anything else |

Why 0.31e9: below half the rules delta the wall row moves <2%, inside noise,
and the release cycle costs more than it buys. Why not higher: a perfect rules
close is ≈1.035, so requiring `MD6` would pre-ordain the acceptance.

- **Release:** write `wishlist3.md` — `W10` expose `FastLegality` (node-constant
  `legal(m)`, pseudo-legal precondition documented) and, if selected, `W11`
  parent-bitboard effect+key. Each ask carries the measured per-call target
  and the exact crate-private symbol. Plan 9 integrates it by owner trigger.
- **Acceptance:** write the dated `MD7` row: the split (rules 0.617e9,
  geometry 0.15–0.36e9, generation 0.1255e9, tree 0.1249e9, search/table
  0.2505e9), the audit's 1.12× ceiling, `M12` as the last prover-side gain,
  and this plan's number. `M1`'s performance part and `M6` close as a
  documented miss; `M8` is then the only open item.

## Phase D — artefacts and docs (~0.5 h)

`measurements/plan8/`: `README.md`, probe source, the `W10` patch,
`census.json`, `results.json`, `core-perf.txt`, `env.json`, `decision.json`
(projection, whole-binary deltas, branch). No binaries, `perf.data`, raw
transcripts or oracle artifacts. `initiative.md`: add `M13`, one status and one
history line. Exactly one of `wishlist3.md` or the `MD7` row; the tree is
unchanged, so no pin and no ledger row.

## Gates

`make test` before and after proves the product tree is untouched. If the
release branch keeps a patch for plan 9, the scratch copy runs
`cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test`,
`cargo doc --no-deps`, `make veto`, and the ignored
`rules_differential_full` (plan 5 counts: 84/11, 27,088 positions, 540,383
moves/keys, 0/0, `MD3` 87/0/0).

## Stop rules

- No edit to `src/`, `Cargo.toml`, `Cargo.lock`, `PROVENANCE.md` or
  `check.rs`. The prototype lives in `/tmp`, unreferenced.
- No `Terms`, tree, order, seed or `.pvr` change; seed 0 counters are the
  contract.
- One decision. If Phase A or B stops, the acceptance runs with the stop as
  its evidence; do not re-tune the variant.
- A checker disagreement or a non-EP rules mismatch is a veto.

## Next

- **Release:** plan 9 = owner-triggered integration of `W10`±`W11`: pin and
  `PROVENANCE.md`, differential, gates, veto, 10 s `MD6` row against the
  same-session plan 2 binary, then the dated `MD7` at the measured number.
- **Acceptance:** performance is closed; the next plan is `M8`, shakmaty
  removal from the parser/display path, with no rules change.

Kickoff prompt (release branch):

> Read `plan8.md`, `report8.md`, `measurements/plan8/` and `wishlist3.md`.
> Plan 9: integrate the `W10`±`W11` release by owner trigger — pin and
> `PROVENANCE.md`, differential, gates, veto, 10 s `MD6` row against the
> same-session plan 2 binary; record the dated acceptance at the measured
> number.

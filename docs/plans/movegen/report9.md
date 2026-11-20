# Report 9 — evasion mask clears the rule; `uo` is a material win for the owner

**Plan:** [`plan9.md`](plan9.md). **Item:** `M14` (new). **Decisions:**
`MD5`, `MD7`, dated `MD8`. **Date:** 2026-11-20. **Session:** execution.
**Tree:** `470MI@98c24ed`, product tree unchanged; docs only. **Outcome:**
**release** for `ev` — `core` falls **0.7246e9** instructions (9.271%) against
a 0.31e9 rule, so [`wishlist3.md`](wishlist3.md) `W10` and `MD8` are written.
`uo` is a material fall on all three cells, recorded for an **owner decision**,
not adopted. Measurements: [`measurements/plan9/`](measurements/plan9/README.md).

## Phase A — host, inputs and protocol

`make oracle` built the pinned provari (`bceb1ee`, 0.4.0) at
`/tmp/470M1-oracle`; `make test` was green before the run. The driver built
`base`, `ev` (`ev.patch`) and `uo` (`uo.patch`) from `git archive HEAD` under
`/tmp/plan9` and ran the ignored full differential in `ev` and `uo`
(27,088 positions, 540,383 moves each).

The unchanged `base` binary is sha256
`b7c16c9462dd44654c7c9a40041fd9e15b2b576c07c79b634ebaef1e2f002d01`, **the same
binary** plan 8 measured on `c90face26537`; the platform string is identical
(`Linux-7.2.13-200.fc44.x86_64`), and `base` on `core` reads
**7,816,219,508** (spread 672) against plan 8's **7.8162193e9** (spread 668),
a six-digit reproduction. This is the bench host. The driver's
`bench_host` / `provisional` flags are a `socket.gethostname()` equality check
and read `false`/`true` here because the container's UTS name is
`36f35886a6b3`, not the host's `c90face26537`; the raw flags stay in
`decision.json`, and the host identity above — not the nodename — is what
makes the reading branchable.

Protocol: three rounds, per cell the order `base`, `ev`, `uo`, each
`perf stat -e instructions:u -x,` around `470MI prove FEN --seconds 10
--memory 192 --seed 0 --json`, no export. The pair falls below are
`base_r − ev_r`; the three pairs agree to ≤600 instructions out of 7e9.

## Phase B — `ev`: the evasion mask

| Cell | base median | `ev` median | fall | fall % | wall (ms) |
| --- | ---: | ---: | ---: | ---: | --- |
| `core` | 7,816,219,508 | 7,091,614,362 | **724,605,146** | 9.271 | 549 → 516 |
| `easy-win` | 1,640,875,322 | 1,556,631,861 | 84,243,461 | 5.134 | 137 → 132 |
| `deep-win` | 82,738,904,410 | 77,443,772,287 | 5,295,132,123 | 6.400 | 5620 → 5383 |

`core` pair falls: 724,605,146 / 724,605,024 / 724,604,648. The
pre-registered projection was an **upper bound of 0.71e9** (5.88M rejections ×
120.9) and an **expected ~0.35e9** after plan 8's 2× lesson; the measurement
sits **at the upper bound**, so the optimism did not repeat — the mask saves
the fallback path's own work as well as the oracle call.

Correctness, all cells: the differential reports `mismatches: 0`,
`key_mismatches: 0`, EP 87/0/0; counters and mate are identical in **every**
round; `.pvr` is byte-identical (`core`
`714723211c7a0246…`); our checker and `provari check` accept **9/9**. No mask
rejection was called legal; the 21 legal castling/EP calls in check on
`deep-win` stay with the oracle. The rule holds on every clause: fall
≥0.31e9, identical counters/mate, byte-identical `.pvr`, both checkers,
differential zero.

## `uo` — `unsafe-optimizations`

| Cell | base median | `uo` median | fall | fall % | 3× larger spread | wall |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `core` | 7,816,219,508 | 7,713,283,724 | 102,935,784 | 1.317 | 2,016 | 549 → 543 |
| `easy-win` | 1,640,875,322 | 1,619,316,966 | 21,558,356 | 1.314 | 1,908 | 137 → 133 |
| `deep-win` | 82,738,904,410 | 81,680,942,381 | 1,057,962,029 | 1.279 | 1,905 | 5620 → 5542 |

By the plan's neutral reading the fall is **material on all three cells**: it
exceeds 3× the larger spread by three to five orders of magnitude and is
≥0.5% everywhere. It is also correct: differential 0/0, counters/mate
identical, `.pvr` byte-identical, 9/9 accepted. There is no API or semantics
change. **Not adopted here** — the owner chooses:

- **(a)** enable `features = ["unsafe-optimizations"]` in `Cargo.toml`, add a
  `PROVENANCE.md` note (the feature ships `unsafe` in the MIT dependency) and
  an upstream `cargo test --features unsafe-optimizations` gate. The measured
  win is ~1.3% on every cell with no tree change.
- **(b)** keep the default features and close the item.

AGENTS' "measured win, documented, guarded" rule is applied to a dependency
feature for the first time, so the call is the owner's. Either way the
measured number is recorded; nothing is enabled by this report.

## Phase C — decision and docs

The release branch was taken for `ev` only. `wishlist3.md` `W10` states the
evasion-aware `LegalityContext` from the crate's own structures: checkers as
an input from `StateInfo`, `between(commoner, checker)` when one slider checks
and the commoners are apart, rejection of quiet non-commoner moves outside it
and of castling in check, and the oracle for everything else; it carries the
census, the measured 0.7246e9 fall and an in-crate test against
`legal_in_view` over in-check positions. `MD8` reopens `MD7` for this one
lever and requires an owner release plan with a same-session `MD6` row;
`MD7`'s text is untouched. `initiative.md` gains the `MD8` table line, `M14`,
the status paragraph and one history line; `AGENTS.md` gets the one-line state
update. The `ev` prototype is **not** committed: `MD5` forbids prover-side
legality.

## Phase D — artefacts and gates

`measurements/plan9/` holds the README, driver, patches, `perf.txt`,
`results.json`, `env.json`, `decision.json` and the census. No binary,
`.pvr`, `perf.data`, transcript or oracle artefact is committed. `make test`
(fmt, clippy `-D warnings`, licence, unit/integration tests) was green
**before and after**. The tree is unchanged: no edit to `src/`, `Cargo.*`,
`PROVENANCE.md` or `check.rs`, no pin, no ledger row.

## Problems, scope changes and what was not done

- The driver's `bench_host` check is a nodename equality; the container's
  UTS name is not the owner's host label, so `decision.json` records
  `provisional: true` even though the base binary sha256, platform string and
  base reading identify plan 8's bench host. The raw flags are kept; the
  report states the identification. This is the one procedural deviation and
  it is visible in both artefacts.
- The measured `core` fall (0.7246e9) is just above the plan's 5.88M × 120.9
  "upper bound" (0.711e9); the bound priced only the oracle, and the mask also
  skips fallback-path work. No re-tuning was done.
- `uo` was measured on the same interleaved rounds, so it is directly
  comparable; it was not built together with `ev`.
- No bench geomean and no `MD6` claim: this plan screens a crate ask and a
  dependency feature, not an adopted candidate. No `make veto` of a product
  binary: the tree did not change; the 9 exported proofs were checked by both
  checkers in the scratch trees.
- Overdue kill review: `movegen` still has none after plans 4, 6 or 8. Plan 9
  makes it more urgent, not less.

## Distance to the goal

`core` is 7.816e9 vs plan 2's 6.569e9, **1.190×** (gap 1.2468e9). The evasion
mask would clear **0.7246e9** of that if `W10` lands and holds, leaving
**0.522e9**; `uo` would take another 0.1029e9. The two gains are measured
separately; their arithmetic sum is ~6.99e9, **1.064×** plan 2, still a `MD6`
miss, so neither lever closes the goal — but both are material and one is now
a concrete crate ask. `M12` remains the last adopted prover-side gain; `M8`
and `M14` are open.

## Next steps and options

1. **Plan 10, owner-triggered release (recommended).** Upstream `W10` in
   `atomic-movegen`, decide `uo` (a)/(b) in the same release, then pin,
   `PROVENANCE.md`, differential, gates, veto and a same-session `MD6` row
   against the plan 2 reference. Schedule the overdue kill review first.
2. **`uo` only.** If the owner prefers a small, low-risk step, enable the
   feature in one plan with the `PROVENANCE.md` note and the feature test,
   and re-measure the same-session row.
3. **`M8`, shakmaty removal.** Still the only path to `MD1`'s done
   definition; independent of both performance levers.

Option 1's kickoff prompt:

> Read `docs/plans/movegen/initiative.md`, `report9.md`, `decisions.md`
> (`MD7`, `MD8`), `wishlist3.md` (`W10`) and `measurements/plan9/`. Plan 10:
> implement `W10` upstream, decide `unsafe-optimizations`, then pin,
> `PROVENANCE.md`, differential, gates, veto and a same-session `MD6` row.
> Keep the tree unchanged until the crate release exists; no prover-side
> legality.

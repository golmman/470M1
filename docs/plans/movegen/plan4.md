# Plan 4 — adopt the next `atomic-movegen` release

**Items:** `M2` (move/effect layer), `M6` (no regression), `M7` (upstream asks);
decisions `MD1`, `MD2`, `MD5`, `MD6`. **Session:** execution, owner-triggered.
**Predecessor:** [`wishlist2.md`](wishlist2.md) and
[`hot-path-audit.md`](hot-path-audit.md). **Report:** `report4.md`.
**Successor:** [`plan5.md`](plan5.md).

## Goal

Integrate one released `atomic-movegen` version carrying the `W6`–`W9`
hot-path asks, then make the prover use those lean APIs directly: no
translation layer on the rules path, no per-candidate view, no board rebuild
for a null move. The 10 s row is measured by `plan5` against the plan 2
reference; this plan is judged on the *per-call* acceptance numbers and on a
green differential, not on promising the bar.

**Trigger.** Nothing here starts until the owner has implemented the wishlist
upstream, cut the release and named the version. The crate is pinned and
released; no fork, no path dependency, no implicit sync.

**Not here:** no rollback to the provari hot path (rejected by the owner
2026-11-19), no search/eval/threats change, no `.pvr` format change, no checker
change, no second release in the same plan.

## Expected outcome (recorded before measuring)

The audit's optimistic ceiling for the whole wishlist is ~7.35e9 `core`
instructions, about **1.12× the plan 2 binary** (6.569e9). A pass on `MD6`
therefore requires the release to be better than the wishlist's own estimate.
The likely report is a near miss with a named residual; that is an acceptable
outcome and the reason `plan5` exists.

## Phases

### 0 — owner trigger (~15 min)

Owner confirms the released version and which items landed (`W6` counts-only
state, `W7` occupancy legality, `W8` parent-bitboard effect+key, `W9` advanced
null). Items that did not land are recorded, not worked around.

### A — pin and provenance (~30 min)

`Cargo.toml` exact pin, `Cargo.lock`, `PROVENANCE.md` row with the release
commit and date, and the `wishlist2.md` status table. No code changes before
the pin builds and the crate's own tests pass.

### B — adoption (~3–4 h)

`src/board.rs`:

- `legal` uses `W7`'s entry point on the state `Mover::new` already holds.
  `Mover::new` populates once; `legal`, `in_check` and the fast path read that
  same state. `W6`'s counts-only path is used only where no `checkers` query
  follows.
- `effect` becomes `W8`'s child query: `occupied`, `key`, `rule50`, `capture`
  from one pass on the parent's bitboards. No `PostMove`, no view copy, and no
  second `is_double_push` probe for the key (`ep_threat` keeps its own).
- `null` uses `W9`; the `Setup` rebuild and `from_setup` disappear.
- The move list stores the crate's native encoding (`M2`); `format.rs` maps it
  to the `.pvr` bits at the boundary. `from_native_move`/`native_move`
  conversion loops leave the hot path.
- Delete the translators left only for conversions. The boundary that stays:
  FEN parsing/`to_shakmaty` for the checker and proof I/O (`M8` removes the
  parser later).

`src/moves.rs`: `Mover`'s fast legality path and effect call the new entry
points; behaviour, not search, is preserved. If a signature change leaks into
`dfpn.rs`, keep it mechanical: same calls, same order, same counters.

### C — differential and tests (~2 h)

- `tests/differential.rs` full sweep: 0 mismatches, 0 key mismatches, EP
  counters 0/0/0, two identical runs.
- `board::matches_shakmaty`, `moves::matches_the_played_positions` and the EP
  regressions stay green; the null clock test pins `+1`.
- `.pvr` fixtures must not change; the pinned `dfpn` core proof and 100k-node
  counters must not change except where the native move order legitimately
  moved them (record which).
- `check.rs` untouched; `uses_nothing_from_the_prover` still passes.

### D — gates (~40 min)

`cargo fmt --check`, clippy `--all-targets -D warnings`, `cargo test`,
`cargo doc --no-deps`, `make test`, `make test-full`, `make veto` (9/9 plus
control rejected).

### E — per-call acceptance (~1 h)

On the ten-FEN probe (audit method): legality **≤100** instructions per
candidate, effect+key **≤120**, no move conversion, counts-only state, null
without a rebuild. Any miss is reported as an item, not tuned away. Then one
`core` run against the same-session plan 2 binary, as input to `plan5`.

### F — report

`measurements/plan4/` (probe binary, probe results, `core` numbers,
`env.json`, README; no raw transcripts), `report4.md`, and the `plan5` handoff
with the residual list.

## Risks and stop rules

- **Licence.** The hot-path code in the crate must be written from the crate's
  own structures; plan 2's provari-derived code is GPL and must not be copied
  into the MIT crate. If the owner instead moves repository code upstream, that
  is a crate licence decision and needs its own row.
- **Behaviour drift.** A move that becomes legal/illegal differently, an EP
  key change, or a proof changed by anything but key/order is a **stop**.
- **Hidden re-derivation.** `W8`'s `Legal` key must not clone or re-derive the
  child; if it does, the item did not land and is recorded as such.
- **No local patch.** A `/tmp` patch is allowed only to calibrate the probe.
  Adoption requires the released pin.
- **No search compensation.** If the bench misses, stop and report; do not
  retune evaluation or move order.

## Cost and deliverables

Trigger 0.25 h, pin 0.5 h, adoption 3–4 h, tests 2 h, gates 0.7 h, probe 1 h,
docs 1 h. Deliver: the new pin and provenance row, the adopted hot path with
the translators gone, green differential and gates, the probe acceptance
numbers, `measurements/plan4/`, `report4.md`. The decisive step is the probe:
if the release's per-call costs miss the acceptance numbers, `plan5`'s bench is
not run and the item returns to the wishlist.

# Wishlist 1 — upstream changes for the `movegen` handoff

**Status:** a request list, not a plan. Baseline `atomic-movegen`
`af3faf0dd80f48cc7fa0cb737f4944bca8f6bf85` (v2.9.0, MIT); W1-W3 landed, W4
open, W5 proposed; `MD1` puts parser removal in scope. The crate's own
`docs/plans/feedback/handoff3.md` in the `golmman/atomic_movegen` repository
remains the live cost wishlist (MG19/MG20); this file records the asks that
follow from [`rules-surface.md`](rules-surface.md). It does not supersede that
handoff.

**Protocol.** We own `golmman/atomic_movegen`, so an item here is a request we
may implement upstream, not a commitment made by this repository. A released
change enters `470M1` only when the owner triggers it: write `planN.md`, then
`reportN.md`, update the pin in `PROVENANCE.md`, and pass the differential,
checker, veto and bench gates. No fork, path dependency or implicit sync is
committed here.

## Priority

| # | Ask | Type | Decisive experiment | Priority |
| --- | --- | --- | --- | --- |
| W1 | **Legal-EP child key on the effect path** | rules/API + cost | measure the double-push share and the clone in the prover's `full_effect` | P1 |
| W2 | **Lichess-style FEN acceptance** | parser parity (landed in 2.9.0) | differential against shakmaty on corpus plus malformed FENs | P1 |
| W3 | **Immediate-explosion query on a child** | one-ply fact | compare with the prover's `Mover::exposes` on random play-outs | P2/P3 |
| W4 | **Node-constant analysis view** | guide geometry | profile `Mover::new` before asking for it | P3 |
| W5 | **Shakmaty-compatible Zobrist table** | key parity, only if native keys lose | native-key bench first; then zero numeric key differences | P1 |

## W1 — legal-EP child key on the effect path

**Why.** The prover keys children by shakmaty's
`EnPassantMode::Legal` hash. `atomic-movegen` can compute that key, but
`Board::hash_after_with(Legal)` clones and plays a double pawn push, and
`EffectContext::post_move_and_key` returns only the default `Always` key.
The adapter therefore either clones for exactly those moves or keeps the
provari `Mover::full_effect` path; the latter defeats replacing the effect
layer.

**Proposal.** Extend the one-derivation child summary with the selected key:

```rust
impl EffectContext {
    pub fn post_move_and_key_with(
        &self, board: &Board, m: Move, mode: EnPassantMode,
    ) -> Option<(PostMove, u64)>;
}

impl PostMove {
    pub fn key_with(&self, mode: EnPassantMode) -> u64; // needs the parent or stored base
}
```

An equivalent shape is a `MoveEffect`/`PostMove` flag that says the child can
carry a legal en-passant key, plus a key helper that uses it. The `Always` key
and `ZOBRIST_VERSION` must not change.

**Acceptance.** For every pseudo-legal move of the conformance corpus, the
child cases and seeded random play-outs, the result equals
`clone + do_move + hash_with(mode)`. Non-double-push moves take no clone;
double pushes derive the child once. The prover's bench counters are
unchanged, and a measured wall win decides adoption.

## W2 — Lichess-style FEN acceptance

**Why.** `parity` accepts FENs as shakmaty's `Setup` does: bad castling
rights, an impossible en-passant square and excess material are dropped, and
a fixed class of illegal positions is rejected. `atomic-movegen`'s
`from_fen` is permissive and `FenPolicy::Strict` is not the same predicate.
Until this matches, the prover keeps shakmaty for parsing, so the rules
replacement is incomplete even when search uses `atomic-movegen`.

**Proposal.** A named policy or constructor that mirrors the parity contract,
for example `FenPolicy::Lichess` (or `Board::from_fen_lichess`), returning the
sanitized board or the same rejection class as shakmaty. It must document
exactly what it drops and rejects; a silent widening of either set is a
finding.

**Acceptance.** The corpus, the fixture FENs and generated mutations of
castling rights, en passant, material count, back-rank pawns, king count and
check give the same accept/reject answer as shakmaty, and accepted positions
have the same board, legal moves and legal-EP key. The parser is removed from
the prover only after this gate.

## W3 — immediate-explosion query on a child

**Why.** The prover's `Mover::exposes`, `exploding_capture` and `threats` ask
a one-ply rules question: after this move, can the other side capture a piece
beside the mover's commoner and explode it at once? The adapter can keep that
code, but then two rules implementations answer the same question on the
prover side. A movegen query gives the adapter one oracle and a differential
target.

**Proposal.** A read-only query over the already-derived child, for example:

```rust
impl EffectContext {
    /// The immediate capture that removes the child side-to-move's commoner.
    pub fn exploding_capture_after(&self, board: &Board, m: Move) -> Option<Move>;
}
```

`MoveClassification` could also expose whether the move leaves such a capture,
but the child query is the useful one. It must respect touching commoners,
pawns, the blast zone, and the rule that a capturing piece beside its own
commoner dies too.

**Acceptance.** Equal to an exhaustive search over the built child's legal
moves on the conformance corpus, fixture positions and seeded random
play-outs; no board write; cost measured before any guide integration. Keep
the prover-side helper until the measurement justifies the swap.

## W4 — node-constant analysis view

**Why.** `Mover::new` derives pins, check squares, uncovering pieces, pieces
beside the enemy commoner, attack maps and exposure targets once per node.
`atomic-movegen` already has `checkers`, `pinned`, `attackers_to` and the
`attacks` geometry, but not one stable view of these facts. If this work is
material, the adapter can drop `Mover` rather than reimplement it.

**Proposal.** Prefer accessors on an existing context, or a small
`AnalysisContext`, exposing masks for: check squares by role, blockers
between a square and a colour's sliders, pieces beside a commoner, a colour's
step/reach attack map, and uncovering pieces for a target. Do not expose
mutable `StateInfo` internals or tie the API to one search design.

**Acceptance.** Equal to the prover's `Mover` fields on random play-outs;
`Mover::new`'s share of the profile is measured first. If it is small, this
item stays a watchlist entry and is not implemented.

## W5 — shakmaty-compatible Zobrist table

**Why.** Plan 1 found the tables differ: start `0x463b96181691fc9c` (ours)
against `0x5ab84d1cfc77f79e` (2.9.0). The schema and legal-EP
canonicalization agree. `MD2` makes native keys the default; W5 is only for a
measured key-driven regression.

**Proposal (2026-11-19).** If native keys lose, add a feature, constructor or
public hook that lets the caller supply the key table. The preferred shape is
an injected `ZobristKeys`/compat hook: the GPL shakmaty table then stays in
this GPL repository and `atomic-movegen` remains MIT. A built-in table in the
MIT crate needs an owner licensing decision first. Keep the default table,
`EnPassantMode` semantics and `ZOBRIST_VERSION`; a default-table change is a
version bump.

**Acceptance.** Measure native keys first. W5 is triggered only by a
key-driven bench regression; then the differential reports zero numeric key
differences on the fixtures, corpus and play-outs, each engine's own child key
and legal-EP gating stay unchanged, and the bench recovers the baseline.

## Formulation option — keep the parser outside (closed)

**Closed by `MD1` (2026-11-19).** Parser removal is in scope; shakmaty may not
remain in the prover as a convenience. The cheaper search-only formulation was
considered and rejected: it leaves the replacement partial. If W2's gate fails,
record the blocker and fix it upstream; do not silently keep the parser.

## Non-wishes

- **No insufficient-material table.** The checker's broader shakmaty rule is
  authoritative; the prover keeps that rule explicitly. `atomic-movegen`'s
  commoner-only material draw stays its own.
- **No proof-order change.** The compact `.pvr` writer and checker own the
  checks-first order; the adapter reorders moves itself.
- **No search, evaluation, tablebase or threading API** in the movegen crate.
- **No unreleased dependency.** A wishlist item becomes usable only in a
  released version recorded by a plan and a provenance pin.

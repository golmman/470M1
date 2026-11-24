# Decisions — `parity`

Full text and rationale for the decisions summarised in
[`initiative.md`](initiative.md). A decision is changed by a new dated row,
not by editing history silently.

**2026-11-24:** product decisions supersede `D2`, `D6`, `D7` and `D12` and
narrow `D8` (`PD1`, `PD2`, `PD5`, `PD11` in [`../../decisions.md`](../../decisions.md)).
`D1`, `D4`, `D5`, `D9` and `D10` carry forward.

## D1 — Licence and copying

**Resolution (2026-11-17).** This repository is GPL-3.0-or-later. provari is
GPL-3.0-or-later; `atomi` and `atomic_solver` are MIT. All three are
compatible with this repository's licence. Code and data may be copied or
adapted from any of them, provided every copied file or fragment carries a
provenance line and a row in `PROVENANCE.md`, and upstream copyright/licence
notices are kept.

**Rationale.** The owner explicitly wants to copy freely from the three
sources. A GPL repository can absorb GPL and MIT material; the provenance
record keeps the origin auditable and makes re-copies deliberate.

**Consequences.** No dependency with a GPL-incompatible licence. The licence
gate is never weakened. provari is also used as an external process; its
source and binaries are never committed.

## D2 — Parity reference

**Resolution (2026-11-17).** The parity reference is provari `v0.4.0` at
`bceb1ee1faf7d9cf2422eb89c931be69ae30619d`. The reference is re-pinned only
by an owner decision, recorded here and in `PROVENANCE.md`.

**Rationale.** `v0.4.0` is the current published provari and the source the
owner named. Pinning one commit makes the bench and the oracle reproducible.

**Consequences.** Every ledger row records the oracle pin. A re-pin is a new
row, never a silent update.

## D3 — Rules dependency for parity

**Resolution (2026-11-17).** The parity prover uses `shakmaty` 0.30.1 with the
`variant` feature for position parsing, legal moves, outcomes and hashing.
The prover may add its own board/move/effect layer for speed, but the rules
authority is shakmaty. Replacing shakmaty with `atomic-movegen` is the
separate `movegen` initiative.

**Rationale.** provari's parity is defined against shakmaty's rules. Starting
with the same rules implementation removes a large correctness variable and
lets the first initiative focus on search and proof parity.

**Consequences.** The checker also uses shakmaty, but independently (D4). The
movegen initiative must preserve the checker's independence.

## D4 — Checker independence

**Resolution (2026-11-17).** `check` uses shakmaty only and shares no code
with the prover, in either direction. A helper both sides need is written
twice. The checker's entry points are the only public surface; the prover may
run the checker but may not borrow its parts.

**Rationale.** A mistake in the prover's board, search or proof writer must
not be able to hide in the checker. This is provari's design and the reason
its proofs can be trusted without trusting the search.

**Consequences.** A test reads the checker source and fails if it imports
prover modules. The checker is the first thing plan1 builds and tests.

## D5 — Proof format

**Resolution (2026-11-17).** The proof format is provari's compact `.pvr`
version 1, plus JSONL for readable proofs. There is no native format. The
compact format writes only the attacker's moves; the checker generates every
defence. The header carries root, attacker (only when not the side to move),
tablebase limit (0 for none), length and position count.

**Rationale.** Using provari's format makes `provari check` a real external
veto and makes proofs interoperable from day one. A native format would add
work without helping parity.

**Consequences.** The writer and checker must implement the format exactly,
including move ordering, LEB128 indices, zstd framing and the no-repetition
rule. `FORMAT.md` is the interoperability spec; its text is not copied into
this repository unless a provenance row says so.

## D6 — Bench set and metric

**Resolution (2026-11-17).** The bench set is the three cells `core`,
`easy-win` and `deep-win` from `corpus-v1`. The metric is
`factor(cell) = median candidate wall / median provari --guide hand wall`.
The reference is re-measured in the same session. Budgets are 10 s and a
192 MB label on both sides; three repeats per cell per side. The ledger is
`docs/bench.md`.

**Rationale.** The set is small, fixed and selected so the reference proves
inside 10 s. Same-session ratios remove host drift. The three cells cover a
guide-helping, a guide-hurting and a deep case.

**Consequences.** A censored cell is never converted. A longer run is its own
row. Geomeans are compared only within one cap. The ledger holds candidate
rows only; reference values are re-measured per row.

## D7 — Scope exclusions

**Resolution (2026-11-17).** The parity initiative excludes the web frontend,
UCI, tablebases, NNUE/model guidance, portfolio/parallel search, draw/loss
proofs, a proof database and a new search formulation. The hand guide is the
product.

**Rationale.** Each excluded item is a large surface that does not help the
first goal. provari's own hand guide is the fair reference column.

**Consequences.** A future initiative may add any of these, but it needs its
own goal, plan and gate. The parity initiative does not grow them silently.

## D8 — Determinism

**Resolution (2026-11-17).** The search is deterministic: the same binary,
FEN and flags produce identical counters. Collecting a proof never changes
the search trajectory. A seed, if added, perturbs only tie-breaks and is part
of the deterministic flag set.

**Rationale.** Determinism makes rungs sound (proving at 10 s implies proving
at 120 s), makes measurements comparable and makes proof collection safe.

**Consequences.** No wall-clock-dependent search decisions. A proof run must
match the timed run's counters.

## D9 — Claim protocol

**Resolution (2026-11-17).** A win is claimed only after its proof passed our
independent checker and `provari check`. A checker disagreement is a veto.
Every non-accept is classified as `accepted`, `checker-reject`, `unchecked`
(checker says `valid: null`), `export-failure` or `rules-divergence`. Only
`accepted` counts.

**Rationale.** A fast false win is worse than a slow true one. The external
checker is the strongest available guard against a shared bug.

**Consequences.** A missing oracle makes the result `INCONCLUSIVE`, not
skipped. A proof that cannot be exported is not a claim.

## D10 — Provenance

**Resolution (2026-11-17).** Every copied or adapted file or fragment gets a
row in `PROVENANCE.md` with destination, source, source sha256, date and mode
(`verbatim` or `adapted: <what>`). A copied file also carries a provenance
line naming the source commit and path. Source pins live in `PROVENANCE.md`.

**Rationale.** The owner wants to copy freely; provenance keeps the copying
auditable and makes the repository's ownership of adapted files explicit.

**Consequences.** A re-copy is a deliberate change with a new row. Implicit
syncs are forbidden. Generated data is marked independent, not copied.

## D11 — Movegen handoff

**Resolution (2026-11-17).** The `movegen` initiative starts only after
`parity` reaches `P5` and the result is recorded in the ledger. The prover
then replaces shakmaty with `atomic-movegen`; the checker stays on shakmaty
so the two rules implementations remain independent.

**Rationale.** Changing the rules layer while chasing parity would confound
search and rules effects. Keeping the checker on shakmaty preserves the
external-veto property.

**Consequences.** `movegen` has its own initiative, plans and gates. It may
not start early, and it may not change the checker's rules dependency.

## D12 — Documentation process

**Resolution (2026-11-17).** Each initiative has an `initiative.md` that is
the only normative statement of its goal, rungs, current priority and
backlog. Plans and reports are historical. `docs/bench.md` is an
initiative-independent results ledger. `docs/plans/README.md` is the status
index; it is updated only when an initiative opens, pivots or closes.

**Rationale.** The atomi/atomic_solver planning process proved fruitful: a
single normative goal, durable decisions/gotchas/history, and self-contained
plans make sessions resumable and prevent silent goal drift.

**Consequences.** Plans <= 8 KB, reports <= 12 KB. A plan names its item. A
report records tools, problems, open parts, missing tests and next steps.

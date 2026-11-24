# Correctness gotchas (binding)

Short index of bugs already found in the source projects and the invariant
that prevents them. These are load-bearing: a change that breaks one needs a
test and a report finding. Read before touching search, table, proof, checker
or measurement code. Moved from `plans/parity/` (product-wide, `PD11`).

## Search and values

- **`Open` numbers saturate below `INFINITY`.** `INFINITY` marks decided
  values only. An `Open` value that saturates at `INFINITY` invents false
  refutations. atomi found this in plan1; keep the same distinction.
- **Proofs never live in the TT.** TT partial keys are sound only because a
  collision can cost a win, never invent one. Every proof goes to the
  full-key proof table.
- **The filter changes no result.** A test pins identical counters with the
  filter on and off. A filter may only cause a miss, never a false hit.
- **A length-bounded pass must not store bound-dependent refutations.** If a
  remaining-length bound is ever introduced, "no win within `L`" is not "no
  win". A proof found under a bound is a genuine proof and may be stored; a
  refutation found under a bound may not go into the full-key table unless
  the key carries the moves left. A too-long proof is not a bounded
  refutation: keep it `Open` so the search looks for a shorter proof.
- **The per-file en passant key is derived.** If the movegen's Zobrist tables
  are private, derive the en passant key and pin it against
  `hash_after_with(EnPassantMode::Legal)`.

## Rules and terminality

- **Repetition and the 50-move limit are never wins.** A line that repeats a
  position, or reaches the 50-move limit, is a draw. The checker enforces the
  same rule as the prover.
- **Identity excludes the clocks.** Positions are compared by board, side to
  move, castling rights and the en passant square only when an en passant
  capture is legal. The 50-move counter is carried separately as a proof
  limit.
- **The checker's insufficient-material rule is the checker's.** shakmaty's
  atomic insufficient-material draws may be broader than another rules
  implementation's. For parity, adopt the checker's rule in the prover's
  rules layer and validator. Anything else is a `rules-divergence` veto.
- **`Board::legal` is not a legality oracle on terminal positions.** If a
  fast path replaces a move list, pair it with the extinction rule. A
  commoner capture is never legal; a side without a king is not in check.
- **Touching kings are never in check.** Do not apply a pin or check rule
  that ignores this.

## Proofs and the checker

- **The checker shares no code with the prover.** A helper both sides need is
  written twice. A test reads the checker source and fails on a prover import.
- **The checker generates every defence.** The compact proof writes only the
  attacker's moves. A proof that relies on a written defence is malformed.
- **Move order is part of the format.** Checks first, then by from-square,
  to-square and promotion. The walk takes defences in square order. A
  different order changes the indices and invalidates the proof.
- **The header has exactly the required fields.** `attacker` appears only
  when the attacker is not the side to move. `tablebases` is 0 when no
  tablebase verdict ends a line.
- **A proof is accepted only if every line ends in a win.** A missing move, a
  draw, a repetition, the 50-move limit or a malformed index is a reject.
- **Proof length is not checked for minimality.** The checker checks
  soundness, not shortness. A longer proof is still a proof; compare length
  as a diagnostic, never as a correctness gate.

## Measurement and evidence

- **A censored cell is a result.** `>= cap / reference` is a lower bound on
  the factor, never a factor. A longer run is its own row.
- **Geomeans are compared only within one cap.** The geomean is non-monotone
  in the cap because a longer cap can claim more cells. A cap chosen per plan
  to improve the headline is cap shopping.
- **Same session, same host.** A factor compares two walls measured back to
  back on one host. Cross-host precision is not enough to call a lever.
- **Wall is the product metric; counters are the mechanism metric.** A
  counter cut that does not move the wall is a mechanism finding, not a speed
  win.
- **A measurement states its blind spots.** No plan may treat the previous
  instrument's field of view as the list of levers.
- **Proof length is a companion column.** Record our length beside the
  same-session reference mate. A row that cannot state both is incomplete.

## Product growth (PRD, 2026-11-24)

- **Counters are a 1-thread instrument.** At `--threads N > 1` counters may
  vary run to run; never use them as a regression fixture there (`PD2`).
- **The 1-thread path stays identical.** Shared tables, stop flags and
  workers must leave `--threads 1` counters and `.pvr` bytes unchanged.
- **Observers are trajectory-neutral.** Progress logging may read state at
  clock ticks only; it must not change node order, budgets or table contents.
  A test pins counters with logging off and at `debug`.
- **Defaults are part of the bench.** Moving a default into a config struct
  must keep its value; a changed default is a bench event, not a refactor.
- **A store entry is not a claim.** Proof-database entries are claimed only
  through an exported `.pvr` that both checkers accept (`PD6`).
- **Warm is not cold.** A run that reads a populated proof database is its
  own row and is never compared with a cold reference run.
- **A refactor can move inlining.** Splitting hot modules can change
  instruction counts with identical counters (movegen plan 6); measure
  `instructions:u` on every hot-path refactor.

## Licence and provenance

- **No GPL-incompatible dependency.** The licence gate is never weakened.
- **Every copy has a provenance row.** A copied file keeps its upstream
  copyright and licence notice. A re-copy is a deliberate change, never an
  implicit sync.
- **provari is an external process for the veto and the bench reference.**
  Its source, binaries and proofs are never committed.

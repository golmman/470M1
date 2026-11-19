"""Plan 6 Phase A bucket map: symbol -> group.

The groups are the plan's table rows. A symbol is assigned by the first
pattern that matches its demangled name, so the map is mechanical and can be
re-read by anyone. Two notes matter for the arithmetic:

* `Board::attackers_to` is a separate symbol only in 2.10.0; in the plan 2
  binary `Position::attackers_to` was inlined into `checked_after` (and
  `king_attacked`), which are rules symbols. The primary map keeps the plan's
  label (`attackers_to` in node geometry); `aligned_map` moves it to rules so
  the cross-binary delta does not count an inlining boundary as growth.
* `Dfpn::mid` in 2.10.0 contains inlined `children`, `Mover::legal`, and the
  effect wrapper; in plan 2 `children` was its own symbol. Both are in
  search/table, and `mid` + `children` are compared as one figure.
"""

# Order matters: the first match wins.
PATTERNS = [
    # rules layer (crate legality, effect, blast, rights, state)
    ("rules", "legal_in::"),
    ("rules", "child_effect_with_occupancy"),
    ("rules", "blast_casualties"),
    ("rules", "castling_rights_after"),
    ("rules", "populate_state"),
    ("rules", "PositionView>::checkers"),
    ("rules", "legal_ep_square"),
    ("rules", "is_pseudo_legal"),
    ("rules", "Position>::checked_after"),
    ("rules", "Position>::legal"),
    ("rules", "Position>::effect"),
    ("rules", "Position>::king_attacked"),
    # the generation adapter
    ("generation", "Position>::pseudo_legal"),
    ("generation", "pawn_moves_to"),
    # node geometry
    ("geometry", "Mover>::exposes"),
    ("geometry", "Mover>::plain"),
    ("geometry", "Mover>::new"),
    ("geometry", "Mover>::gives_check"),
    ("geometry", "Mover>::threats"),
    ("geometry", "Mover>::connectable"),
    ("geometry", "Mover>::blasted_cheaply"),
    ("geometry", "eval::terms"),
    ("geometry", "shakmaty::attacks::attacks"),
    ("geometry", "shakmaty::attacks::queen_attacks"),
    ("geometry", "Board>::attackers_to"),
    # null
    ("null", "Dfpn>::replay_threat"),
    # play + memory movement
    ("play", "Position>::play"),
    ("play", "__memmove"),
    ("play", "__memset"),
    # search and table
    ("search", "Dfpn>::mid"),
    ("search", "Dfpn>::children"),
    ("search", "Dfpn>::evaluate"),
    ("search", "Dfpn>::store"),
    ("search", "Dfpn>::prefetch_child"),
    ("search", "Dfpn>::read_filter"),
    ("search", "Dfpn>::simulate"),
    ("search", "Dfpn>::simulate_defences"),
    ("search", "Dfpn>::on_line"),
    ("search", "Dfpn>::exposed"),
    ("search", "Dfpn>::keep_attacker"),
    ("search", "Dfpn>::add_proof"),
    ("search", "Dfpn>::record_proof"),
    ("search", "Dfpn>::mated"),
    ("search", "Dfpn>::recomputed"),
    ("search", "Proofs>::"),
    ("search", "proofs::Entry"),
    ("search", "Table>::"),
    ("search", "TakeWhile"),
    ("search", "roundf"),
    # per-position adapter screens, neither geometry nor generation
    ("adapter", "moves::replies"),
    ("adapter", "moves::exploding_capture"),
    ("adapter", "moves::has_legal_move"),
    ("adapter", "moves::cannot_win"),
]


def group(symbol):
    for name, pattern in PATTERNS:
        if pattern in symbol:
            return name
    return "other"


# The plan-2 counterpart of `Board::attackers_to` lives inside these rules
# symbols, so the aligned reading moves the 2.10.0 symbol to rules.
ALIGNED = (("rules", "Board>::attackers_to"),)


def aligned_group(symbol):
    for bucket, pattern in ALIGNED:
        if pattern in symbol:
            return bucket
    return group(symbol)

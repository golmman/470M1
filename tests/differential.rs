//! Differential harness between the prover's rules layer and `atomic-movegen`
//! 2.9.0 (`af3faf0`, plan 1).
//!
//! Two entry points: the fast smoke set for `cargo test`, and the full sweep
//! over fixtures, corpus, generated mutations and seeded play-outs as
//! `#[ignore = "slow: ..."]`.
//!
//! The only exempt class is material: shakmaty's broad atomic rule draws
//! K+B/K+N/K+R vs K, while `atomic-movegen` deliberately has no such table.
//! The harness counts those hits and asserts the prover's rule separately in
//! `material_divergence_is_explicit`. Everything else fails closed with the
//! FEN, move, expected and actual value.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use atomic_movegen::{
    Board, Color as TheirColor, EnPassantMode as TheirEp, FenPolicy, Move as TheirMove, Outcome,
    PieceType as TheirPiece, StateInfo, TerminalReason, movegen,
};
use atomic470m1::board::{Move as OurMove, Position as OurPosition};
use atomic470m1::moves::{self, Mover};
use atomic470m1::{ending, position};
use shakmaty::fen::Fen;
use shakmaty::{Color, EnPassantMode, Position as _, Role, Square, uci::UciMove, variant::Atomic};

const STANDARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Default)]
struct Stats {
    fen_accepted: u64,
    fen_rejected: u64,
    positions: u64,
    legal_moves: u64,
    child_keys: u64,
    explosions: u64,
    explosion_skipped_decided: u64,
    explosion_mismatches: u64,
    explosion_ep_mismatches: u64,
    explosion_other_mismatches: u64,
    exposes_mismatches: u64,
    exposes_ep_mismatches: u64,
    exposes_other_mismatches: u64,
    node_explosion_mismatches: u64,
    node_explosion_ep_mismatches: u64,
    node_explosion_other_mismatches: u64,
    material_exempt: u64,
    key_table_differences: u64,
    games: u64,
    plies: u64,
    playout_nodes: u64,
    mismatches: u64,
}

impl Stats {
    fn json(&self, label: &str) -> String {
        let mut out = String::new();
        write!(
            out,
            "DIFFERENTIAL {{\"label\":\"{label}\",\"fen_accepted\":{},\"fen_rejected\":{},\
             \"positions\":{},\"legal_moves\":{},\"child_keys\":{},\"explosions\":{},\
             \"explosion_skipped_decided\":{},\"explosion_mismatches\":{},\
             \"explosion_ep_mismatches\":{},\"explosion_other_mismatches\":{},\
             \"exposes_mismatches\":{},\"exposes_ep_mismatches\":{},\
             \"exposes_other_mismatches\":{},\"node_explosion_mismatches\":{},\
             \"node_explosion_ep_mismatches\":{},\"node_explosion_other_mismatches\":{},\
             \"material_exempt\":{},\
             \"key_table_differences\":{},\"games\":{},\"plies\":{},\
             \"playout_nodes\":{},\"mismatches\":{}}}",
            self.fen_accepted,
            self.fen_rejected,
            self.positions,
            self.legal_moves,
            self.child_keys,
            self.explosions,
            self.explosion_skipped_decided,
            self.explosion_mismatches,
            self.explosion_ep_mismatches,
            self.explosion_other_mismatches,
            self.exposes_mismatches,
            self.exposes_ep_mismatches,
            self.exposes_other_mismatches,
            self.node_explosion_mismatches,
            self.node_explosion_ep_mismatches,
            self.node_explosion_other_mismatches,
            self.material_exempt,
            self.key_table_differences,
            self.games,
            self.plies,
            self.playout_nodes,
            self.mismatches,
        )
        .expect("writing to a String cannot fail");
        out
    }
}

/// A deterministic linear congruential generator; no `rand` dependency.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Lcg {
        Lcg(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
}

fn base_fens() -> Vec<String> {
    let mut fens = vec![STANDARD.to_owned()];
    for row in include_str!("fixtures/veto.tsv").lines() {
        let row = row.trim();
        if row.is_empty() || row.starts_with('#') || row.starts_with("name\t") {
            continue;
        }
        let mut fields = row.split('\t');
        let _name = fields.next();
        if let Some(fen) = fields.next() {
            fens.push(fen.to_owned());
        }
    }
    for row in include_str!("fixtures/never_prove.tsv").lines() {
        let row = row.trim();
        if row.is_empty() || row.starts_with('#') || row.starts_with("name\t") {
            continue;
        }
        let mut fields = row.split('\t');
        let _name = fields.next();
        if let Some(fen) = fields.next() {
            fens.push(fen.to_owned());
        }
    }
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../bench/corpus.json")).expect("corpus.json");
    fens.push(corpus["core"]["fen"].as_str().expect("core FEN").to_owned());
    for cell in corpus["sample20"].as_array().expect("sample20") {
        fens.push(cell["fen"].as_str().expect("sample FEN").to_owned());
    }
    fens
}

/// Generated acceptance mutations: every castling right with and without its
/// pieces, consistent and inconsistent en passant, material, back-rank pawns,
/// commoner counts, check classes and syntax errors.
fn mutation_fens() -> Vec<String> {
    let mut fens = Vec::new();
    for (right, rook, king) in [
        ("K", "h1", "e1"),
        ("Q", "a1", "e1"),
        ("k", "h8", "e8"),
        ("q", "a8", "e8"),
    ] {
        let mut fields: Vec<String> = STANDARD.split(' ').map(str::to_owned).collect();
        fields[2] = right.to_owned();
        let with_right = fields.join(" ");
        fens.push(with_right.clone());
        fens.push(without_piece(&with_right, rook));
        fens.push(without_piece(&with_right, king));
    }
    fens.extend(
        [
            // A consistent en passant target: White's e5 pawn may take d6.
            "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
            // No double push supports these targets.
            "4k3/8/8/8/8/8/8/4K3 w - e6 0 1",
            "4k3/8/8/8/8/8/8/4K3 w - e4 0 1",
            "4k3/8/8/8/4P3/8/8/4K3 b - e3 0 1",
            // Excess material is accepted; pawns on the back rank are not.
            "4k3/8/8/8/8/P7/PPPPPPPP/4K3 w - - 0 1",
            "4k3/8/8/8/8/8/8/P3K3 w - - 0 1",
            // Zero, one (side to move) and two commoners for one side.
            "8/8/8/8/8/8/8/8 w - - 0 1",
            "4k3/8/8/8/8/8/8/8 w - - 0 1",
            "4k3/8/8/8/8/8/8/3KK3 w - - 0 1",
            // The side that just moved is in check.
            "4k3/8/8/8/8/8/4Q3/4K3 w - - 0 1",
            "4k3/8/8/8/8/8/4q3/4K3 b - - 0 1",
            // More checkers than a game can deliver.
            "1rrrrrk1/1PPPPPPP/8/8/8/8/8/6K1 b - - 0 1",
            // Malformed syntax.
            "",
            "not a fen",
            "8/8/8/8/8/8/8/8",
            "4k3/8/8/8/8/8/8/4K3 w - -",
            "4k3/8/8/8/8/8/8/4K3 w - - 0 1 extra",
        ]
        .map(str::to_owned),
    );
    fens
}

/// Remove the piece on `square` from a FEN's placement field.
fn without_piece(fen: &str, square: &str) -> String {
    let mut fields: Vec<String> = fen.split(' ').map(str::to_owned).collect();
    let file = usize::from(square.as_bytes()[0] - b'a');
    let rank = usize::from(square.as_bytes()[1] - b'1');
    let ranks: Vec<&str> = fields[0].split('/').collect();
    let index = 7 - rank;
    let mut cells = Vec::new();
    for ch in ranks[index].chars() {
        if let Some(count) = ch.to_digit(10) {
            cells.resize(cells.len() + count as usize, '1');
        } else {
            cells.push(ch);
        }
    }
    cells[file] = '1';
    let mut row = String::new();
    let mut empty = 0;
    for cell in cells {
        if cell == '1' {
            empty += 1;
        } else {
            if empty > 0 {
                write!(row, "{empty}").expect("writing to a String cannot fail");
                empty = 0;
            }
            row.push(cell);
        }
    }
    if empty > 0 {
        write!(row, "{empty}").expect("writing to a String cannot fail");
    }
    let mut new_ranks: Vec<String> = ranks.iter().map(|rank| (*rank).to_owned()).collect();
    new_ranks[index] = row;
    fields[0] = new_ranks.join("/");
    fields.join(" ")
}

fn their_color(color: Color) -> TheirColor {
    if color.is_white() {
        TheirColor::White
    } else {
        TheirColor::Black
    }
}

fn their_role(role: Role) -> TheirPiece {
    match role {
        Role::Pawn => TheirPiece::Pawn,
        Role::Knight => TheirPiece::Knight,
        Role::Bishop => TheirPiece::Bishop,
        Role::Rook => TheirPiece::Rook,
        Role::Queen => TheirPiece::Queen,
        Role::King => TheirPiece::Commoner,
    }
}

fn their_square(square: Square) -> atomic_movegen::Square {
    atomic_movegen::Square::from_u8(square as u8)
}

fn to_their_move(m: OurMove) -> TheirMove {
    let from = their_square(m.from());
    let to = their_square(m.to());
    if m.is_castle() {
        TheirMove::make_castling(from, to)
    } else if m.is_en_passant() {
        TheirMove::make_enpassant(from, to)
    } else if let Some(role) = m.promotion() {
        TheirMove::make_promotion(from, to, their_role(role))
    } else {
        TheirMove::make_move(from, to)
    }
}

fn uci(m: shakmaty::Move) -> String {
    UciMove::from_standard(m).to_string()
}

fn our_legal(pos: &OurPosition) -> Vec<OurMove> {
    let mover = Mover::new(pos);
    pos.pseudo_legal()
        .as_slice()
        .iter()
        .copied()
        .filter(|&m| mover.legal(m))
        .collect()
}

fn our_uci(pos: &OurPosition, m: OurMove) -> String {
    uci(pos.to_shakmaty(m))
}

fn their_legal(board: &Board) -> Vec<TheirMove> {
    let mut list = atomic_movegen::MoveList::new();
    movegen::generate_legal(board, &mut list);
    list.as_slice().to_vec()
}

/// The same piece, if any: our shakmaty piece and their piece on one square.
fn same_piece(our: Option<shakmaty::Piece>, their: atomic_movegen::Piece) -> bool {
    match (our, their.color(), their.type_of()) {
        (None, None, None) => true,
        (Some(our), Some(their_color), Some(their_type)) => {
            (our.color.is_white() == (their_color == TheirColor::White))
                && their_role(our.role) == their_type
        }
        _ => false,
    }
}

fn square_set(raw: u64) -> BTreeSet<u8> {
    (0..64).filter(|index| raw & (1 << index) != 0).collect()
}

fn compare_placement(
    context: &str,
    our: &OurPosition,
    post: &atomic_movegen::PostMove,
    occupied: Option<u64>,
) {
    for index in 0..64u8 {
        let square = Square::new(u32::from(index));
        let theirs = post.piece_on(atomic_movegen::Square::from_u8(index));
        assert!(
            same_piece(our.piece_at(square), theirs),
            "{context}: placement differs on {square:?}: ours {:?}, theirs {:?}",
            our.piece_at(square),
            theirs,
        );
    }
    if let Some(occupied) = occupied {
        let mut ours = 0u64;
        for square in our.occupied() {
            ours |= 1 << (square as u8);
        }
        assert_eq!(
            square_set(ours),
            square_set(occupied),
            "{context}: occupancy differs"
        );
    }
}

fn compare_terminal(label: &str, our: &Atomic, board: &Board, stats: &mut Stats) {
    let ours = ending(our);
    let theirs = board.terminal();
    if let Some(terminal) = theirs
        && terminal.reason == TerminalReason::MaterialDraw
    {
        stats.material_exempt += 1;
        return;
    }
    if matches!(ours, Some((_, "insufficient_material"))) {
        stats.material_exempt += 1;
        return;
    }
    let turn = our.turn();
    let expected = theirs.map(|terminal| {
        let winner = match terminal.outcome {
            Outcome::Win => Some(turn),
            Outcome::Loss => Some(!turn),
            Outcome::Draw => None,
            _ => unreachable!("two outcomes"),
        };
        let reason = match terminal.reason {
            TerminalReason::Extinction => "variant_loss",
            TerminalReason::NoMovesInCheck => "checkmate",
            TerminalReason::Stalemate => "stalemate",
            TerminalReason::FiftyMoveRule => "fifty_moves",
            TerminalReason::MaterialDraw => unreachable!("handled above"),
            _ => unreachable!("the 2.9.0 reason set"),
        };
        (winner, reason)
    });
    assert_eq!(
        ours,
        expected,
        "{label}: terminality differs; FEN {}",
        Fen::from_position(our, EnPassantMode::Legal)
    );
}

fn compare_explosion_witness(
    context: &str,
    our_child: &OurPosition,
    board: &Board,
    their_move: TheirMove,
    their_witness: TheirMove,
) {
    let mut child = board.clone();
    let mut state = StateInfo::new();
    child.do_move(their_move, &mut state);
    assert!(
        child.is_legal(their_witness),
        "{context}: witness {their_witness} is not legal"
    );
    let target = their_color(!our_child.turn());
    let mut after = child.clone();
    let mut state = StateInfo::new();
    after.do_move(their_witness, &mut state);
    assert!(
        after.commoners(target).is_empty(),
        "{context}: witness {their_witness} does not explode the mover's commoner"
    );
}

fn compare_node(label: &str, our: &OurPosition, shak: &Atomic, board: &Board, stats: &mut Stats) {
    let context = || {
        format!(
            "{label}: FEN {}",
            Fen::from_position(shak, EnPassantMode::Legal)
        )
    };
    stats.positions += 1;

    // Keys: the two crates ship independently generated Zobrist tables, so
    // numeric cross-equality is a recorded finding, not a rules assertion.
    // Within each implementation the key must still be the child's key and the
    // legal-en-passant component must gate exactly the same way.
    if our.hash() != board.hash_with(TheirEp::Legal) {
        stats.key_table_differences += 1;
    }
    let ep_legal = board.legal_ep_square().is_some();
    assert_eq!(
        our.ep_hash() != 0,
        ep_legal,
        "{}: en passant legality differs (ours {}, theirs {})",
        context(),
        our.ep_hash() != 0,
        ep_legal
    );
    assert_eq!(
        board.hash_with(TheirEp::Always) == board.hash_with(TheirEp::Legal),
        ep_legal || board.ep_square().is_none(),
        "{}: movegen legal-EP canonicalization differs",
        context()
    );
    if shak.maybe_ep_square().is_some() {
        let fen = Fen::from_position(shak, EnPassantMode::Always).to_string();
        let mut fields: Vec<String> = fen.split(' ').map(str::to_owned).collect();
        fields[3] = "-".to_owned();
        let cleared = fields.join(" ");
        let our_cleared = OurPosition::from_atomic(&position(&cleared).expect("cleared FEN"));
        let their_cleared =
            Board::from_fen_with(&cleared, FenPolicy::Lichess).expect("cleared FEN");
        assert_eq!(
            our.hash() ^ our_cleared.hash(),
            our.ep_hash(),
            "{}: our en passant key component differs",
            context()
        );
        let their_ep_component =
            board.hash_with(TheirEp::Legal) ^ their_cleared.hash_with(TheirEp::Legal);
        assert_eq!(
            their_ep_component != 0,
            ep_legal,
            "{}: movegen en passant key component differs",
            context()
        );
    }

    // Legal moves: ours, shakmaty's and theirs, as UCI sets, exactly once each.
    let shak_moves: Vec<shakmaty::Move> = shak.legal_moves().into_iter().collect();
    let shak_uci: BTreeSet<String> = shak_moves.iter().copied().map(uci).collect();
    assert_eq!(
        shak_uci.len(),
        shak_moves.len(),
        "{}: shakmaty listed a move twice",
        context()
    );
    let ours = our_legal(our);
    let our_uci_set: BTreeSet<String> = ours.iter().map(|&m| our_uci(our, m)).collect();
    assert_eq!(
        our_uci_set.len(),
        ours.len(),
        "{}: our generator listed a move twice",
        context()
    );
    assert_eq!(
        our_uci_set,
        shak_uci,
        "{}: our legal set differs",
        context()
    );
    let theirs = their_legal(board);
    let their_uci_set: BTreeSet<String> = theirs.iter().copied().map(TheirMove::to_uci).collect();
    assert_eq!(
        their_uci_set.len(),
        theirs.len(),
        "{}: movegen listed a move twice",
        context()
    );
    assert_eq!(
        their_uci_set,
        shak_uci,
        "{}: movegen legal set differs",
        context()
    );
    stats.legal_moves += ours.len() as u64;

    // Pseudo-legal membership agrees with the generated legal set both ways.
    let mover = Mover::new(our);
    for &m in our.pseudo_legal().as_slice() {
        assert_eq!(
            mover.legal(m),
            shak_uci.contains(&our_uci(our, m)),
            "{}: our legality differs for {m:?}",
            context()
        );
    }
    let ctx = board.effect_context();
    for m in &ours {
        let am = to_their_move(*m);
        assert!(
            board.is_legal(am),
            "{}: our legal move {am} is not legal",
            context()
        );
    }

    // Per legal move: key, effects, placement and the immediate explosion.
    for &m in &ours {
        let am = to_their_move(m);
        let child = our.play(m);
        let effect = mover.full_effect(m);
        let (post, key) = ctx
            .post_move_and_key_with(board, am, TheirEp::Legal)
            .expect("a move with a piece");
        stats.child_keys += 1;
        assert_eq!(
            child.hash(),
            effect.hash,
            "{}: our played child key differs for {am}",
            context()
        );
        assert_eq!(
            board.hash_after_with(am, TheirEp::Legal),
            key,
            "{}: movegen child key differs for {am}",
            context()
        );
        assert_eq!(
            post.is_capture(),
            effect.capture,
            "{}: capture flag differs for {am}",
            context()
        );
        assert_eq!(
            u32::from(post.rule50()),
            effect.counter,
            "{}: 50-move counter differs for {am}",
            context()
        );

        let mut removed = 0u64;
        if !m.is_castle() {
            for square in our.occupied().without(child.occupied()) {
                removed |= 1 << (square as u8);
            }
            removed &= !(1u64 << (m.from() as u8));
            if effect.capture {
                removed |= 1u64 << (m.to() as u8);
            }
        }
        assert_eq!(
            square_set(removed),
            square_set(post.casualties().all().0),
            "{}: blast casualties differ for {am}",
            context()
        );
        compare_placement(
            &format!("{}: child after {am}", context()),
            &child,
            &post,
            Some(post.occupied().0),
        );

        if let Some(role) = m.promotion() {
            // A capture-promotion's new piece is at ground zero and dies; a
            // quiet promotion stays. Full placement is compared above.
            let expected = (!effect.capture).then(|| role.of(our.turn()));
            assert_eq!(
                child.piece_at(m.to()),
                expected,
                "{}: promotion differs for {am}",
                context()
            );
        }
        if m.is_castle() {
            let side = shakmaty::CastlingSide::from_queen_side(m.to() < m.from());
            let king_to = side.king_to(our.turn());
            let rook_to = side.rook_to(our.turn());
            assert_eq!(
                child.piece_at(king_to),
                Some(our.turn().king()),
                "{}: castling commoner differs for {am}",
                context()
            );
            assert_eq!(
                child.piece_at(rook_to),
                Some(our.turn().rook()),
                "{}: castling rook differs for {am}",
                context()
            );
        }

        // A decided child has no explosion question: movegen documents `None`
        // for it, and the search never asks. Terminality itself is compared
        // below and at the child node.
        if post.terminal().is_some() {
            stats.explosion_skipped_decided += 1;
        } else {
            let our_witness = moves::exploding_capture(&child, child.turn());
            let their_witness = ctx.exploding_capture_after(board, am);
            if our_witness.is_some() != their_witness.is_some() {
                let en_passant = their_witness.is_some_and(TheirMove::is_en_passant)
                    || our_witness.is_some_and(OurMove::is_en_passant);
                stats.explosion_mismatches += 1;
                stats.mismatches += 1;
                if en_passant {
                    stats.explosion_ep_mismatches += 1;
                } else {
                    stats.explosion_other_mismatches += 1;
                }
                eprintln!(
                    "EXPLOSION {} after {am}: ours {:?}, theirs {:?}",
                    context(),
                    our_witness.map(|m| our_uci(&child, m)),
                    their_witness.map(|m| m.to_uci()),
                );
            }
            if mover.exposes(m, &effect) != their_witness.is_some() {
                let en_passant = their_witness.is_some_and(TheirMove::is_en_passant);
                stats.exposes_mismatches += 1;
                stats.mismatches += 1;
                if en_passant {
                    stats.exposes_ep_mismatches += 1;
                } else {
                    stats.exposes_other_mismatches += 1;
                }
                eprintln!(
                    "EXPOSES {} after {am}: exposes {}, immediate capture {}",
                    context(),
                    mover.exposes(m, &effect),
                    their_witness.map(|m| m.to_uci()).unwrap_or_default(),
                );
            }
            if let Some(witness) = our_witness {
                stats.explosions += 1;
                assert!(
                    Mover::new(&child).legal(witness),
                    "{}: our witness {witness:?} is not legal after {am}",
                    context()
                );
                let after = child.play(witness);
                assert!(
                    after.king(!child.turn()).is_none(),
                    "{}: our witness {witness:?} does not explode after {am}",
                    context()
                );
            }
            if let Some(witness) = their_witness {
                compare_explosion_witness(
                    &format!("{}: after {am}", context()),
                    &child,
                    board,
                    am,
                    witness,
                );
            }
        }
    }

    // Node-level immediate explosion, the same one-ply fact the search uses.
    if board.terminal().is_some() {
        stats.explosion_skipped_decided += 1;
    } else {
        let our_node = moves::exploding_capture(our, our.turn());
        let ctx = board.effect_context();
        let mut their_ep = false;
        let mut their_other = false;
        for m in their_legal(board) {
            if ctx.classification(board, m).explodes_enemy_commoner {
                if m.is_en_passant() {
                    their_ep = true;
                } else {
                    their_other = true;
                }
            }
        }
        let their_node = their_ep || their_other;
        if our_node.is_some() != their_node {
            stats.explosion_mismatches += 1;
            stats.node_explosion_mismatches += 1;
            stats.mismatches += 1;
            if their_ep {
                stats.node_explosion_ep_mismatches += 1;
            } else {
                stats.node_explosion_other_mismatches += 1;
            }
            eprintln!(
                "NODE EXPLOSION {}: ours {:?}, theirs {their_node} (en passant {their_ep})",
                context(),
                our_node.map(|m| our_uci(our, m)),
            );
        }
    }

    compare_terminal(&context(), shak, board, stats);
}

fn run_acceptance(fen: &str, stats: &mut Stats) {
    let ours = position(fen);
    let theirs = Board::from_fen_with(fen, FenPolicy::Lichess);
    assert_eq!(
        ours.is_ok(),
        theirs.is_ok(),
        "acceptance differs for {fen:?}: ours {:?}, theirs {:?}",
        ours.as_ref().err(),
        theirs.as_ref().err()
    );
    match (ours, theirs) {
        (Ok(shak), Ok(board)) => {
            stats.fen_accepted += 1;
            let our = OurPosition::from_atomic(&shak);
            let our_fen = Fen::from_position(&shak, EnPassantMode::Legal).to_string();
            let their_fen = board.fen_with(TheirEp::Legal);
            assert_eq!(our_fen, their_fen, "accepted FEN differs for {fen:?}");
            assert_eq!(
                shak.maybe_ep_square().map(|sq| sq as u8),
                board.ep_square().map(|sq| sq as u8),
                "stored en passant differs for {fen:?}"
            );
            compare_node(fen, &our, &shak, &board, stats);
        }
        (Err(_), Err(_)) => stats.fen_rejected += 1,
        _ => unreachable!("agreement was asserted"),
    }
}

fn run_acceptance_set(fens: &[String], stats: &mut Stats) {
    for fen in fens {
        run_acceptance(fen, stats);
    }
}

fn accepted_starts() -> Vec<String> {
    base_fens()
        .into_iter()
        .filter(|fen| {
            position(fen).is_ok() && Board::from_fen_with(fen, FenPolicy::Lichess).is_ok()
        })
        .collect()
}

fn run_playouts(seeds: &[u64], games: usize, max_plies: usize, stats: &mut Stats) {
    let starts = accepted_starts();
    assert!(!starts.is_empty(), "no play-out start position");
    for (seed_index, &seed) in seeds.iter().enumerate() {
        for game in 0..games {
            let fen = &starts[(seed_index * games + game) % starts.len()];
            let shak = position(fen).expect("an accepted start");
            let mut our = OurPosition::from_atomic(&shak);
            let mut board = Board::from_fen_with(fen, FenPolicy::Lichess).expect("accepted");
            let mut rng = Lcg::new(seed ^ (game as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
            let mut seen = BTreeSet::new();
            seen.insert(our.hash());
            stats.games += 1;
            compare_node(fen, &our, &shak, &board, stats);
            let mut shak = shak;
            for _ in 0..max_plies {
                let mut legal: Vec<(String, OurMove)> = our_legal(&our)
                    .into_iter()
                    .map(|m| (our_uci(&our, m), m))
                    .collect();
                legal.sort_by(|a, b| a.0.cmp(&b.0));
                if legal.is_empty() {
                    break;
                }
                let (_, chosen) = &legal[(rng.next() % legal.len() as u64) as usize];
                let chosen = *chosen;
                let am = to_their_move(chosen);
                let shak_move = our.to_shakmaty(chosen);
                our = our.play(chosen);
                shak.play_unchecked(shak_move);
                let mut state = StateInfo::new();
                board.do_move(am, &mut state);
                stats.plies += 1;
                stats.playout_nodes += 1;
                compare_node(fen, &our, &shak, &board, stats);
                let repeated = !seen.insert(our.hash());
                if repeated || board.terminal().is_some() {
                    break;
                }
            }
        }
    }
}

#[test]
fn fen_acceptance_and_rules_smoke() {
    let mut stats = Stats::default();
    run_acceptance_set(&base_fens(), &mut stats);
    run_acceptance_set(&mutation_fens(), &mut stats);
    run_playouts(&[0x2545_f491_4f6c_dd1d], 4, 24, &mut stats);
    println!("{}", stats.json("smoke"));
    assert_eq!(stats.mismatches, 0);
}

#[test]
#[ignore = "slow: rules differential over fixtures, corpus and play-outs"]
fn rules_differential_full() {
    let mut stats = Stats::default();
    run_acceptance_set(&base_fens(), &mut stats);
    run_acceptance_set(&mutation_fens(), &mut stats);
    run_playouts(
        &[
            0x9e37_79b9_7f4a_7c15,
            0x2545_f491_4f6c_dd1d,
            0xdead_beef_cafe_babe,
        ],
        128,
        200,
        &mut stats,
    );
    println!("{}", stats.json("full"));
    assert_eq!(stats.mismatches, 0);
}

/// Numeric key parity is blocked by two independently generated Zobrist
/// tables. The harness therefore compares key semantics (child key and
/// legal-EP gating) and records this divergence; it is not a rules difference.
#[test]
fn numeric_key_parity_is_a_recorded_divergence() {
    let fen = STANDARD;
    let shak = position(fen).expect("standard position");
    let our = OurPosition::from_atomic(&shak);
    let board = Board::from_fen_with(fen, FenPolicy::Lichess).expect("standard position");
    println!(
        "our key {:016x}, movegen key {:016x}",
        our.hash(),
        board.hash_with(TheirEp::Legal)
    );
    assert_ne!(
        our.hash(),
        board.hash_with(TheirEp::Legal),
        "{fen}: the plan's numeric key equality holds; retire this finding"
    );
    // The same board maps to a stable key on each side of the boundary.
    let shak2 = position(fen).expect("standard position");
    let board2 = Board::from_fen_with(fen, FenPolicy::Lichess).expect("standard position");
    assert_eq!(our.hash(), OurPosition::from_atomic(&shak2).hash());
    assert_eq!(
        board.hash_with(TheirEp::Legal),
        board2.hash_with(TheirEp::Legal)
    );
}

/// The material exemption is explicit: shakmaty's broad atomic rule draws
/// K+B/K+N/K+R vs K, while `atomic-movegen` deliberately does not.
#[test]
fn material_divergence_is_explicit() {
    for (fen, role) in [
        ("k7/8/8/8/8/8/8/KB6 w - - 0 1", Role::Bishop),
        ("k7/8/8/8/8/8/8/KN6 w - - 0 1", Role::Knight),
        ("k7/8/8/8/8/8/8/KR6 w - - 0 1", Role::Rook),
    ] {
        let shak = position(fen).expect("a legal position");
        let our = OurPosition::from_atomic(&shak);
        let board = Board::from_fen_with(fen, FenPolicy::Lichess).expect("a legal position");
        assert!(
            shak.is_insufficient_material(),
            "{fen}: shakmaty should call this insufficient"
        );
        assert_eq!(
            ending(&shak),
            Some((None, "insufficient_material")),
            "{fen}: the prover's ending should use the broad rule"
        );
        assert!(
            moves::cannot_win(&our, Color::White) && moves::cannot_win(&our, Color::Black),
            "{fen}: the prover's `cannot_win` should use the broad rule"
        );
        assert_ne!(
            board.terminal().map(|terminal| terminal.reason),
            Some(TerminalReason::MaterialDraw),
            "{fen}: movegen has no {role:?} material table"
        );
        assert_eq!(
            board.outcome(),
            None,
            "{fen}: the game is not decided by movegen's material rule"
        );
    }
}

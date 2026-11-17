//! A proof's lines, as `470m1 line` shows them: the proof's move and every
//! defence with the attacker moves it lasts, a longest line, and the proof's
//! tree as PGN.
//!
//! Adapted from provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`
//! Size note: this is a copied/adapted upstream module, kept whole by plan1;
//! see `PROVENANCE.md`.
//! (`src/lines.rs`): tablebase branches removed.

use serde_json::{Value, json};
use shakmaty::san::{San, SanPlus};
use shakmaty::{
    Color, EnPassantMode, Move, Position as _, fen::Fen, uci::UciMove, variant::Atomic,
};

use crate::{Status, hash, status, walk};

/// A proof's lines from the position after `played` (SAN or UCI, move
/// numbers allowed), as JSON: against the proof's moves for `attacker`, the
/// most attacker moves the defender can last from there, the proof's move,
/// each defence with the attacker moves it lasts, longest first, and a
/// longest line. With `pgn`, the proof's tree from there too, as PGN, ending
/// its lines where the proof wins within `until` attacker moves.
pub fn show(
    root: Atomic,
    attacker: Color,
    moves: walk::Moves,
    played: &str,
    pgn: bool,
    until: Option<u32>,
) -> Result<(Value, Option<String>), String> {
    let mut proof = Proof::new(attacker, moves);
    let mut quiet = |_| {};
    let mut lines = Lines {
        proof: &mut proof,
        until,
        report: &mut quiet,
        written: 0,
    };
    let mut pos = root;
    for token in played.split_whitespace() {
        // A move number, alone or before its move: "6.", "6...", "7.e4".
        let text = match token.find(|c: char| !c.is_ascii_digit()) {
            Some(i) if token[i..].starts_with('.') => token[i..].trim_start_matches('.'),
            _ => token,
        };
        if text.is_empty() {
            continue;
        }
        if !lines.open(&pos) {
            return Err("the game is over here".into());
        }
        let m = text
            .parse::<San>()
            .ok()
            .and_then(|san| san.to_move(&pos).ok())
            .or_else(|| text.parse::<UciMove>().ok()?.to_move(&pos).ok())
            .ok_or(format!("{token} is not a legal move here"))?;
        if pos.turn() == attacker {
            match lines.proof_move(&pos) {
                Ok(proved) if proved != m => {
                    return Err(format!(
                        "the proof plays {} here, not {token}",
                        SanPlus::from_move(pos.clone(), proved)
                    ));
                }
                Ok(_) => {}
                Err(error) => return Err(error),
            }
        }
        pos.play_unchecked(m);
    }
    if !lines.open(&pos) {
        return Err("the game is over here".into());
    }
    let fen = Fen::from_position(&pos, EnPassantMode::Legal).to_string();
    let mut result = json!({ "fen": fen });
    let mut after = pos.clone();
    if pos.turn() == attacker {
        let m = lines.proof_move(&pos)?;
        result["move"] = json!(SanPlus::from_move(pos.clone(), m).to_string());
        after.play_unchecked(m);
    }
    let defences = lines.replies(&after)?;
    let longest = defences.first().map_or(0, |d| d.0);
    result["moves"] = json!(longest + u32::from(pos.turn() == attacker));
    result["defences"] = json!(
        defences
            .iter()
            .map(|&(length, reply)| json!([
                SanPlus::from_move(after.clone(), reply).to_string(),
                length
            ]))
            .collect::<Vec<_>>()
    );
    let longest = lines.proof.longest_line(&pos)?;
    let mut line = Vec::new();
    let mut here = pos.clone();
    for (i, &m) in longest.iter().enumerate() {
        line.push(play(&mut here, m, i == 0));
    }
    result["line"] = json!(line.join(" "));
    let mut text = None;
    if pgn {
        let (tree, moves) = lines.tree(pos, &fen)?;
        text = Some(tree);
        result["pgn_moves"] = json!(moves);
    }
    Ok((result, text))
}

/// A proof read for its lines: its attacker's moves, and the measures of the
/// positions walked so far.
pub struct Proof {
    attacker: Color,
    moves: walk::Moves,
    walk: walk::Walk,
}

impl Proof {
    pub fn new(attacker: Color, moves: walk::Moves) -> Proof {
        Proof {
            attacker,
            moves,
            walk: walk::Walk::measures_only(),
        }
    }

    pub fn attacker(&self) -> Color {
        self.attacker
    }

    pub fn moves(&self) -> &walk::Moves {
        &self.moves
    }

    fn open(&self, pos: &Atomic) -> bool {
        matches!(status(pos, self.attacker), Status::Open)
    }

    /// The attacker's move as the proof's length counts it: a capture that
    /// wins at once where there is one, and the proof's move otherwise.
    pub fn proof_move(&self, pos: &Atomic) -> Result<Move, String> {
        if let Some(m) = walk::winning_capture(pos, self.attacker) {
            return Ok(m);
        }
        self.moves
            .get(&hash(pos))
            .copied()
            .flatten()
            .ok_or_else(|| {
                format!(
                    "the proof has no move for {}",
                    Fen::from_position(pos, EnPassantMode::Legal)
                )
            })
    }

    /// The defender's replies with the attacker moves each lasts, longest
    /// first and the larger subproof first among equals; none once the game
    /// has decided.
    fn replies(&mut self, after: &Atomic) -> Result<Vec<(u32, Move)>, String> {
        if !self.open(after) {
            return Ok(Vec::new());
        }
        let mut replies = Vec::new();
        for reply in after.legal_moves() {
            let mut next = after.clone();
            next.play_unchecked(reply);
            let lasts = if self.open(&next) {
                let measure = walk::measure(&next, self.attacker, &self.moves, &mut self.walk)?;
                (measure.length, measure.tree)
            } else {
                (0, 0)
            };
            replies.push((lasts, reply));
        }
        replies.sort_by_key(|&(lasts, _)| std::cmp::Reverse(lasts));
        Ok(replies
            .into_iter()
            .map(|((length, _), reply)| (length, reply))
            .collect())
    }

    /// A longest line from `pos`: the proof's moves against the replies that
    /// last longest.
    pub fn longest_line(&mut self, pos: &Atomic) -> Result<Vec<Move>, String> {
        let mut line = Vec::new();
        let mut here = pos.clone();
        while self.open(&here) {
            let m = if here.turn() == self.attacker {
                self.proof_move(&here)?
            } else {
                self.replies(&here)?
                    .first()
                    .map(|&(_, reply)| reply)
                    .ok_or("the proof ends without a win")?
            };
            here.play_unchecked(m);
            line.push(m);
        }
        Ok(line)
    }

    /// Whether the attacker's move here ends the game.
    fn wins_at_once(&self, pos: &Atomic) -> Result<bool, String> {
        let m = self.proof_move(pos)?;
        let mut next = pos.clone();
        next.play_unchecked(m);
        Ok(matches!(status(&next, self.attacker), Status::Won))
    }

    /// How many attacker moves the proof takes to win from `pos`, whoever
    /// is to move: none where it does not cover `pos`, and 0 where the
    /// attacker has won.
    pub fn value(&mut self, pos: &Atomic) -> Option<u32> {
        match status(pos, self.attacker) {
            Status::Won => return Some(0),
            Status::Over => return None,
            Status::Open => {}
        }
        let (attacker, moves, walk) = (self.attacker, &self.moves, &mut self.walk);
        let measure = if pos.turn() == attacker {
            walk::measure(pos, attacker, moves, walk)
        } else {
            walk::measure_after(pos, attacker, moves, walk)
        };
        measure.ok().map(|measure| measure.length)
    }
}

/// A proof read for its lines from a position, and where its PGN tree stops.
struct Lines<'a> {
    proof: &'a mut Proof,
    /// Where the tree stops: at positions the proof wins within this many
    /// attacker moves.
    until: Option<u32>,
    /// Hears the moves and comments written so far, every few thousand.
    report: &'a mut dyn FnMut(u64),
    written: u64,
}

impl Lines<'_> {
    fn open(&self, pos: &Atomic) -> bool {
        self.proof.open(pos)
    }

    fn proof_move(&self, pos: &Atomic) -> Result<Move, String> {
        self.proof.proof_move(pos)
    }

    fn replies(&mut self, after: &Atomic) -> Result<Vec<(u32, Move)>, String> {
        self.proof.replies(after)
    }

    fn wins_at_once(&self, pos: &Atomic) -> Result<bool, String> {
        self.proof.wins_at_once(pos)
    }

    /// The proof's tree from `pos`, whose FEN is `fen`, as a PGN game, and
    /// the moves it writes.
    fn tree(&mut self, pos: Atomic, fen: &str) -> Result<(String, usize), String> {
        let attacker = self.proof.attacker;
        let (us, them) = if attacker.is_white() {
            ("White", "Black")
        } else {
            ("Black", "White")
        };
        let cut = self.until.map_or(String::new(), |until| {
            format!(" Lines end where the proof wins within {until} moves.")
        });
        let mut tokens = vec![format!(
            "{{The proof's tree. The main line is {them}'s longest defence and the \
             variations are its other defences, except those after which {us} wins \
             at once. At each branch, mate in N counts {us}'s moves in the proof \
             from there.{cut}}}"
        )];
        self.pgn(pos, &mut tokens, true)?;
        let outcome = if attacker.is_white() { "1-0" } else { "0-1" };
        let moves = tokens
            .iter()
            .filter(|token| !token.starts_with(['{', '(', ')']))
            .count();
        tokens.push(outcome.to_owned());
        let text = format!(
            "[Event \"Proof\"]\n[Site \"?\"]\n[Date \"????.??.??\"]\n[Round \"?\"]\n\
             [White \"?\"]\n[Black \"?\"]\n[Result \"{outcome}\"]\n[Variant \"Atomic\"]\n\
             [SetUp \"1\"]\n[FEN \"{fen}\"]\n\n{}\n\n",
            movetext(&tokens)
        );
        Ok((text, moves))
    }

    /// Writes a move or a comment of the tree, and reports every few
    /// thousand.
    fn write(&mut self, tokens: &mut Vec<String>, token: String) {
        tokens.push(token);
        self.written += 1;
        if self.written.is_multiple_of(4096) {
            (self.report)(self.written);
        }
    }

    /// Writes the proof's tree from `pos` as PGN movetext: the attacker's
    /// moves, the defence that lasts longest as the main line and the others
    /// as variations, but those after which the attacker wins at once, and at
    /// each branch the attacker moves each defence lasts; with `until`, a
    /// defence that lasts no longer ends its line there.
    fn pgn(
        &mut self,
        mut pos: Atomic,
        tokens: &mut Vec<String>,
        mut numbered: bool,
    ) -> Result<(), String> {
        while self.open(&pos) {
            if pos.turn() == self.proof.attacker {
                let m = self.proof_move(&pos)?;
                self.write(tokens, play(&mut pos, m, numbered));
                numbered = false;
                continue;
            }
            let replies = self.replies(&pos)?;
            let Some(&(lasts, main)) = replies.first() else {
                break;
            };
            let mut others = Vec::new();
            for &(length, reply) in &replies[1..] {
                let mut next = pos.clone();
                next.play_unchecked(reply);
                if !self.wins_at_once(&next)? {
                    others.push((length, reply));
                }
            }
            let until = self.until;
            let short = |length: u32| until.is_some_and(|until| length <= until);
            let ends = short(lasts);
            let before = pos.clone();
            self.write(tokens, play(&mut pos, main, numbered));
            if !others.is_empty() || ends {
                self.write(tokens, format!("{{mate in {lasts}}}"));
            }
            for &(length, reply) in &others {
                let mut next = before.clone();
                tokens.push("(".to_owned());
                self.write(tokens, play(&mut next, reply, true));
                self.write(tokens, format!("{{mate in {length}}}"));
                if !short(length) {
                    self.pgn(next, tokens, false)?;
                }
                tokens.push(")".to_owned());
            }
            if ends {
                break;
            }
            numbered = !others.is_empty();
        }
        Ok(())
    }
}

/// Plays a move and writes it in SAN, numbered as PGN numbers it: always for
/// White, for Black where `numbered`.
pub fn play(pos: &mut Atomic, m: Move, numbered: bool) -> String {
    let number = pos.fullmoves().get();
    let white = pos.turn().is_white();
    let san = SanPlus::from_move_and_play_unchecked(pos, m);
    match (white, numbered) {
        (true, _) => format!("{number}. {san}"),
        (false, true) => format!("{number}... {san}"),
        (false, false) => san.to_string(),
    }
}

/// PGN movetext from its tokens, in lines of at most 79 characters where
/// tokens allow, with variations in parentheses that hug their moves.
fn movetext(tokens: &[String]) -> String {
    let mut text = String::new();
    let mut width = 0;
    let mut glued = true;
    for token in tokens {
        if !glued && token != ")" {
            if width + 1 + token.len() > 79 {
                text.push('\n');
                width = 0;
            } else {
                text.push(' ');
                width += 1;
            }
        }
        text.push_str(token);
        width += token.len();
        glued = token == "(";
    }
    text
}

#[cfg(test)]
mod tests {
    use shakmaty::{Position as _, uci::UciMove, variant::Atomic};

    use super::Proof;
    use crate::format::{SMALL, decode, encode_jsonl, small_defended, strategy};

    fn play(pos: &Atomic, uci: &str) -> Atomic {
        let m = uci.parse::<UciMove>().unwrap().to_move(pos).unwrap();
        let mut next = pos.clone();
        next.play_unchecked(m);
        next
    }

    #[test]
    fn writes_a_tree() {
        let (_, bytes) = encode_jsonl(SMALL).unwrap();
        let (header, indices) = decode(&bytes).unwrap().unwrap();
        let (root, attacker, moves) = strategy(&header, &indices).unwrap();
        let mut proof = Proof::new(attacker, moves.clone());
        let (shown, pgn) = super::show(root.clone(), attacker, moves, "", true, None).unwrap();
        assert!(pgn.unwrap().contains("[Result \"1-0\"]"));
        assert_eq!(shown["moves"].as_u64(), Some(4));
        let (line, _) = proof.longest_line(&root).map(|line| (line, 0)).unwrap();
        assert_eq!(line.len(), 7);
    }

    #[test]
    fn values_follow_the_proof() {
        let (_, bytes) = encode_jsonl(SMALL).unwrap();
        let (header, indices) = decode(&bytes).unwrap().unwrap();
        let (root, attacker, moves) = strategy(&header, &indices).unwrap();
        let mut proof = Proof::new(attacker, moves);
        assert_eq!(proof.value(&root), Some(4));
        assert_eq!(proof.value(&play(&root, "g5f7")), Some(3));
        assert_eq!(proof.value(&play(&root, "h4h5")), None);
        assert_eq!(proof.value(&root), Some(4));
    }

    #[test]
    fn shows_a_proof_from_the_defender_to_move() {
        let (_, bytes) = small_defended();
        let (header, indices) = decode(&bytes).unwrap().unwrap();
        let (root, attacker, moves) = strategy(&header, &indices).unwrap();
        let (shown, pgn) =
            super::show(root.clone(), attacker, moves.clone(), "", true, None).unwrap();
        assert_eq!(
            (shown["moves"].as_u64(), shown.get("move")),
            (Some(3), None)
        );
        assert_eq!(shown["defences"][0][1], 3);
        assert!(pgn.unwrap().contains("[Result \"1-0\"]"));
        assert_eq!(Proof::new(attacker, moves).value(&root), Some(3));
    }
}

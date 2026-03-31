use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

static OPENING_DATA: OnceLock<OpeningDatabase> = OnceLock::new();

const OPENINGS_TSV: &str = include_str!("../openings.tsv");

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningInfo {
    pub eco: String,
    pub name: String,
    pub phase: GamePhase,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamePhase {
    Opening,
    EarlyMiddlegame,
    Middlegame,
    EarlyEndgame,
    Endgame,
}

impl GamePhase {
    /// Determine the game phase from move count (half-moves played) and total pieces remaining.
    pub fn detect(half_moves: usize, total_pieces: usize) -> Self {
        // total_pieces is the count of all pieces on the board (both sides, including kings)
        // Standard game starts with 32 pieces
        match (half_moves, total_pieces) {
            (m, _) if m <= 12 => GamePhase::Opening,
            (m, _) if m <= 24 => GamePhase::EarlyMiddlegame,
            (_, p) if p > 22 => GamePhase::Middlegame,
            (_, p) if p > 14 => GamePhase::Middlegame,
            (_, p) if p > 10 => GamePhase::EarlyEndgame,
            _ => GamePhase::Endgame,
        }
    }
}

struct TrieNode {
    children: HashMap<String, TrieNode>,
    opening: Option<OpeningMeta>,
}

#[derive(Clone)]
struct OpeningMeta {
    eco: String,
    name: String,
}

struct OpeningDatabase {
    trie: TrieNode,
    epd_map: HashMap<String, OpeningMeta>,
}

fn new_trie_node() -> TrieNode {
    TrieNode {
        children: HashMap::new(),
        opening: None,
    }
}

fn parse_and_build() -> OpeningDatabase {
    let mut root = new_trie_node();
    let mut epd_map = HashMap::new();

    for line in OPENINGS_TSV.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 5 {
            continue;
        }

        let eco = fields[0];
        let name = fields[1];
        let uci = fields[3];
        let epd = fields[4];

        let meta = OpeningMeta {
            eco: eco.to_string(),
            name: name.to_string(),
        };

        // Insert into trie
        let moves: Vec<&str> = uci.split_whitespace().collect();
        let mut node = &mut root;
        for mv in &moves {
            node = node
                .children
                .entry((*mv).to_string())
                .or_insert_with(new_trie_node);
        }
        node.opening = Some(meta.clone());

        // Insert into EPD map (last entry wins for duplicate EPDs)
        epd_map.insert(epd.to_string(), meta);
    }

    OpeningDatabase {
        trie: root,
        epd_map,
    }
}

fn get_database() -> &'static OpeningDatabase {
    OPENING_DATA.get_or_init(parse_and_build)
}

/// Match an opening by walking the move trie. Returns the deepest (most specific) match.
fn match_by_moves(move_list: &[String]) -> Option<&'static OpeningMeta> {
    let db = get_database();
    let mut node = &db.trie;
    let mut last_match: Option<&OpeningMeta> = None;

    for mv in move_list {
        match node.children.get(mv.as_str()) {
            Some(next) => {
                node = next;
                if node.opening.is_some() {
                    last_match = node.opening.as_ref();
                }
            }
            None => break,
        }
    }

    last_match
}

/// Fallback: match by EPD string for transposition detection.
fn match_by_epd(epd: &str) -> Option<&'static OpeningMeta> {
    get_database().epd_map.get(epd).map(|m| m)
}

/// Detect the opening from the game's move list, with EPD fallback for transpositions.
/// `epd` should be the EPD of the current position (FEN without halfmove/fullmove clocks).
/// `total_pieces` is the number of pieces currently on the board (both sides).
pub fn detect_opening(move_list: &[String], epd: Option<&str>, total_pieces: usize) -> OpeningInfo {
    let half_moves = move_list.len();
    let phase = GamePhase::detect(half_moves, total_pieces);

    // Try trie match first (most specific)
    if let Some(meta) = match_by_moves(move_list) {
        return OpeningInfo {
            eco: meta.eco.clone(),
            name: meta.name.clone(),
            phase,
        };
    }

    // Fallback to EPD for transpositions
    if let Some(epd_str) = epd {
        if let Some(meta) = match_by_epd(epd_str) {
            return OpeningInfo {
                eco: meta.eco.clone(),
                name: meta.name.clone(),
                phase,
            };
        }
    }

    OpeningInfo {
        eco: String::new(),
        name: String::new(),
        phase,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruy_lopez() {
        let moves: Vec<String> = vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1b5"]
            .into_iter()
            .map(String::from)
            .collect();

        let result = detect_opening(&moves, None, 32);
        assert_eq!(result.eco, "C60");
        assert!(result.name.contains("Ruy Lopez"));
    }

    #[test]
    fn test_partial_match() {
        let moves: Vec<String> = vec!["e2e4", "e7e5", "g1f3"]
            .into_iter()
            .map(String::from)
            .collect();

        let result = detect_opening(&moves, None, 32);
        // Should match "King's Knight Opening" or similar
        assert!(!result.eco.is_empty());
    }

    #[test]
    fn test_no_match() {
        let moves: Vec<String> = vec!["z9z9"].into_iter().map(String::from).collect();
        let result = detect_opening(&moves, None, 32);
        assert!(result.eco.is_empty());
    }

    #[test]
    fn test_epd_fallback() {
        // EPD for the starting position after 1. Nh3
        let epd = "rnbqkbnr/pppppppp/8/8/8/7N/PPPPPPPP/RNBQKB1R b KQkq -";
        let moves: Vec<String> = vec!["invalid_move"].into_iter().map(String::from).collect();
        let result = detect_opening(&moves, Some(epd), 32);
        assert_eq!(result.eco, "A00");
        assert!(result.name.contains("Amar"));
    }

    #[test]
    fn test_game_phase_opening() {
        assert!(matches!(GamePhase::detect(4, 32), GamePhase::Opening));
    }

    #[test]
    fn test_game_phase_early_middlegame() {
        assert!(matches!(
            GamePhase::detect(16, 30),
            GamePhase::EarlyMiddlegame
        ));
    }

    #[test]
    fn test_game_phase_middlegame() {
        assert!(matches!(GamePhase::detect(30, 24), GamePhase::Middlegame));
    }

    #[test]
    fn test_game_phase_early_endgame() {
        assert!(matches!(GamePhase::detect(40, 12), GamePhase::EarlyEndgame));
    }

    #[test]
    fn test_game_phase_endgame() {
        assert!(matches!(GamePhase::detect(60, 6), GamePhase::Endgame));
    }
}

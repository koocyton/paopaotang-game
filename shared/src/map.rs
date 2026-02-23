use crate::constants::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    HardBlock,
    SoftBlock,
}

pub type GameMap = [[Tile; MAP_COLS]; MAP_ROWS];

pub fn generate_map() -> GameMap {
    let mut map = [[Tile::Empty; MAP_COLS]; MAP_ROWS];

    // Hard blocks in a grid pattern (every other row/col, starting from 1,1)
    for r in 0..MAP_ROWS {
        for c in 0..MAP_COLS {
            if r % 2 == 0 && c % 2 == 0 && r > 0 && c > 0 && r < MAP_ROWS - 1 && c < MAP_COLS - 1
            {
                map[r][c] = Tile::HardBlock;
            }
        }
    }

    // Soft blocks fill most remaining spaces
    for r in 0..MAP_ROWS {
        for c in 0..MAP_COLS {
            if map[r][c] != Tile::Empty {
                continue;
            }
            if is_spawn_safe_zone(r, c) {
                continue;
            }
            map[r][c] = Tile::SoftBlock;
        }
    }

    map
}

fn is_spawn_safe_zone(r: usize, c: usize) -> bool {
    let spawns = spawn_positions();
    for (sr, sc) in &spawns {
        let dr = if r >= *sr { r - sr } else { sr - r };
        let dc = if c >= *sc { c - sc } else { sc - c };
        if dr + dc <= 2 {
            return true;
        }
    }
    false
}

pub fn spawn_positions() -> [(usize, usize); MAX_PLAYERS] {
    [
        (0, 0),
        (0, MAP_COLS - 1),
        (MAP_ROWS - 1, 0),
        (MAP_ROWS - 1, MAP_COLS - 1),
    ]
}

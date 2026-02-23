use crate::map::Tile;
use serde::{Deserialize, Serialize};

// Client -> Server messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMsg {
    Join { name: String },
    Move { dx: f64, dy: f64 },
    PlaceBomb,
}

// Server -> Client messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMsg {
    Welcome {
        player_id: u8,
        room_id: String,
    },
    GameState {
        players: Vec<PlayerState>,
        bombs: Vec<BombState>,
        explosions: Vec<ExplosionState>,
        items: Vec<ItemState>,
        map: Vec<Vec<Tile>>,
        tick: u64,
    },
    GameStart {
        map: Vec<Vec<Tile>>,
        players: Vec<PlayerState>,
    },
    GameOver {
        winner: Option<u8>,
    },
    Waiting {
        player_count: usize,
        need: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: u8,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub alive: bool,
    pub speed: f64,
    pub bomb_range: u32,
    pub max_bombs: u32,
    pub color_index: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BombState {
    pub x: usize,
    pub y: usize,
    pub owner: u8,
    pub timer: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplosionState {
    pub x: usize,
    pub y: usize,
    pub timer: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemKind {
    BombRange,
    BombCount,
    Speed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemState {
    pub x: usize,
    pub y: usize,
    pub kind: ItemKind,
}

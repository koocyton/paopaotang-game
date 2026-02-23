pub const TILE_SIZE: u32 = 32;
pub const MAP_COLS: usize = 15;
pub const MAP_ROWS: usize = 13;
pub const CANVAS_WIDTH: u32 = TILE_SIZE * MAP_COLS as u32;
pub const CANVAS_HEIGHT: u32 = TILE_SIZE * MAP_ROWS as u32;

pub const TICK_RATE_MS: u64 = 50; // 20 ticks per second
pub const BOMB_TIMER_TICKS: u32 = 40; // 2 seconds
pub const EXPLOSION_DURATION_TICKS: u32 = 10; // 0.5 seconds

pub const DEFAULT_SPEED: f64 = 2.0;
pub const SPEED_BOOST: f64 = 0.5;
pub const DEFAULT_BOMB_RANGE: u32 = 1;
pub const DEFAULT_MAX_BOMBS: u32 = 1;

pub const MAX_PLAYERS: usize = 4;

pub const PLAYER_COLORS: [&str; 4] = ["#FF4444", "#4444FF", "#44BB44", "#FFAA00"];

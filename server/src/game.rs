use rand::Rng;
use shared::constants::*;
use shared::map::{self, GameMap, Tile};
use shared::protocol::*;

pub struct Player {
    pub id: u8,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub alive: bool,
    pub speed: f64,
    pub bomb_range: u32,
    pub max_bombs: u32,
    pub active_bombs: u32,
    pub color_index: u8,
    pub dx: f64,
    pub dy: f64,
}

struct Bomb {
    x: usize,
    y: usize,
    owner: u8,
    timer: u32,
    range: u32,
}

struct Explosion {
    x: usize,
    y: usize,
    timer: u32,
}

struct Item {
    x: usize,
    y: usize,
    kind: ItemKind,
}

pub struct GameInstance {
    pub map: GameMap,
    pub players: Vec<Player>,
    bombs: Vec<Bomb>,
    explosions: Vec<Explosion>,
    items: Vec<Item>,
    pub tick: u64,
    pub running: bool,
    pub finished: bool,
    pub winner: Option<u8>,
}

impl GameInstance {
    pub fn new() -> Self {
        GameInstance {
            map: map::generate_map(),
            players: Vec::new(),
            bombs: Vec::new(),
            explosions: Vec::new(),
            items: Vec::new(),
            tick: 0,
            running: false,
            finished: false,
            winner: None,
        }
    }

    pub fn add_player(&mut self, id: u8, name: String) {
        let spawns = map::spawn_positions();
        let idx = self.players.len().min(spawns.len() - 1);
        let (sr, sc) = spawns[idx];
        self.players.push(Player {
            id,
            name,
            x: sc as f64,
            y: sr as f64,
            alive: true,
            speed: DEFAULT_SPEED,
            bomb_range: DEFAULT_BOMB_RANGE,
            max_bombs: DEFAULT_MAX_BOMBS,
            active_bombs: 0,
            color_index: idx as u8,
            dx: 0.0,
            dy: 0.0,
        });
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn set_player_movement(&mut self, player_id: u8, dx: f64, dy: f64) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.dx = dx.clamp(-1.0, 1.0);
            p.dy = dy.clamp(-1.0, 1.0);
        }
    }

    pub fn place_bomb(&mut self, player_id: u8) {
        let (px, py, range, can_place) = {
            if let Some(p) = self.players.iter().find(|p| p.id == player_id && p.alive) {
                let bx = (p.x + 0.5) as usize;
                let by = (p.y + 0.5) as usize;
                (bx, by, p.bomb_range, p.active_bombs < p.max_bombs)
            } else {
                return;
            }
        };

        if !can_place {
            return;
        }

        let already_has_bomb = self.bombs.iter().any(|b| b.x == px && b.y == py);
        if already_has_bomb {
            return;
        }

        self.bombs.push(Bomb {
            x: px,
            y: py,
            owner: player_id,
            timer: BOMB_TIMER_TICKS,
            range,
        });

        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.active_bombs += 1;
        }
    }

    pub fn tick(&mut self) {
        if !self.running || self.finished {
            return;
        }
        self.tick += 1;
        self.move_players();
        self.update_bombs();
        self.update_explosions();
        self.check_item_pickup();
        self.check_game_over();
    }

    fn move_players(&mut self) {
        for p in self.players.iter_mut() {
            if !p.alive {
                continue;
            }
            let step = p.speed * (TICK_RATE_MS as f64 / 1000.0);
            let new_x = p.x + p.dx * step;
            let new_y = p.y + p.dy * step;

            if can_move_to(&self.map, &self.bombs, new_x, p.y, p.id) {
                p.x = new_x;
            }
            if can_move_to(&self.map, &self.bombs, p.x, new_y, p.id) {
                p.y = new_y;
            }

            p.x = p.x.clamp(0.0, (MAP_COLS - 1) as f64);
            p.y = p.y.clamp(0.0, (MAP_ROWS - 1) as f64);
        }
    }

    fn update_bombs(&mut self) {
        let mut exploded = Vec::new();

        for bomb in self.bombs.iter_mut() {
            if bomb.timer > 0 {
                bomb.timer -= 1;
            }
            if bomb.timer == 0 {
                exploded.push((bomb.x, bomb.y, bomb.owner, bomb.range));
            }
        }

        for (bx, by, owner, range) in &exploded {
            self.create_explosion(*bx, *by, *range);
            if let Some(p) = self.players.iter_mut().find(|p| p.id == *owner) {
                if p.active_bombs > 0 {
                    p.active_bombs -= 1;
                }
            }
        }

        self.bombs.retain(|b| b.timer > 0);
    }

    fn create_explosion(&mut self, cx: usize, cy: usize, range: u32) {
        let mut rng = rand::thread_rng();

        self.explosions.push(Explosion {
            x: cx,
            y: cy,
            timer: EXPLOSION_DURATION_TICKS,
        });

        let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (ddx, ddy) in &directions {
            for i in 1..=(range as i32) {
                let nx = cx as i32 + ddx * i;
                let ny = cy as i32 + ddy * i;
                if nx < 0 || ny < 0 || nx >= MAP_COLS as i32 || ny >= MAP_ROWS as i32 {
                    break;
                }
                let ux = nx as usize;
                let uy = ny as usize;

                match self.map[uy][ux] {
                    Tile::HardBlock => break,
                    Tile::SoftBlock => {
                        self.map[uy][ux] = Tile::Empty;
                        self.explosions.push(Explosion {
                            x: ux,
                            y: uy,
                            timer: EXPLOSION_DURATION_TICKS,
                        });
                        if rng.gen_ratio(1, 3) {
                            let kind = match rng.gen_range(0..3) {
                                0 => ItemKind::BombRange,
                                1 => ItemKind::BombCount,
                                _ => ItemKind::Speed,
                            };
                            self.items.push(Item {
                                x: ux,
                                y: uy,
                                kind,
                            });
                        }
                        break;
                    }
                    Tile::Empty => {
                        self.explosions.push(Explosion {
                            x: ux,
                            y: uy,
                            timer: EXPLOSION_DURATION_TICKS,
                        });
                    }
                }
            }
        }

        // Kill players in explosions
        for p in self.players.iter_mut() {
            if !p.alive {
                continue;
            }
            let px = (p.x + 0.5) as usize;
            let py = (p.y + 0.5) as usize;
            for exp in &self.explosions {
                if exp.x == px && exp.y == py {
                    p.alive = false;
                    break;
                }
            }
        }

        // Chain explosions: detonate any bomb caught in the blast
        let mut chain: Vec<usize> = Vec::new();
        for (i, bomb) in self.bombs.iter().enumerate() {
            for exp in &self.explosions {
                if bomb.x == exp.x && bomb.y == exp.y && bomb.timer > 0 {
                    chain.push(i);
                    break;
                }
            }
        }
        for i in chain.into_iter().rev() {
            self.bombs[i].timer = 0;
        }
    }

    fn update_explosions(&mut self) {
        for exp in self.explosions.iter_mut() {
            if exp.timer > 0 {
                exp.timer -= 1;
            }
        }
        // Remove items caught in explosions
        self.items.retain(|item| {
            !self
                .explosions
                .iter()
                .any(|e| e.x == item.x && e.y == item.y && e.timer > 0)
        });
        self.explosions.retain(|e| e.timer > 0);
    }

    fn check_item_pickup(&mut self) {
        let mut picked = Vec::new();
        for (i, item) in self.items.iter().enumerate() {
            for p in self.players.iter_mut() {
                if !p.alive {
                    continue;
                }
                let px = (p.x + 0.5) as usize;
                let py = (p.y + 0.5) as usize;
                if px == item.x && py == item.y {
                    match item.kind {
                        ItemKind::BombRange => p.bomb_range += 1,
                        ItemKind::BombCount => p.max_bombs += 1,
                        ItemKind::Speed => p.speed += SPEED_BOOST,
                    }
                    picked.push(i);
                    break;
                }
            }
        }
        for i in picked.into_iter().rev() {
            self.items.remove(i);
        }
    }

    fn check_game_over(&mut self) {
        let alive: Vec<u8> = self
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.id)
            .collect();

        if alive.len() <= 1 && self.players.len() >= 2 {
            self.finished = true;
            self.running = false;
            self.winner = alive.first().copied();
        }
    }

    pub fn get_state_msg(&self) -> ServerMsg {
        ServerMsg::GameState {
            players: self.player_states(),
            bombs: self.bombs.iter().map(|b| BombState {
                x: b.x,
                y: b.y,
                owner: b.owner,
                timer: b.timer,
            }).collect(),
            explosions: self.explosions.iter().map(|e| ExplosionState {
                x: e.x,
                y: e.y,
                timer: e.timer,
            }).collect(),
            items: self.items.iter().map(|i| ItemState {
                x: i.x,
                y: i.y,
                kind: i.kind,
            }).collect(),
            map: self.map.iter().map(|row| row.to_vec()).collect(),
            tick: self.tick,
        }
    }

    pub fn player_states(&self) -> Vec<PlayerState> {
        self.players
            .iter()
            .map(|p| PlayerState {
                id: p.id,
                name: p.name.clone(),
                x: p.x,
                y: p.y,
                alive: p.alive,
                speed: p.speed,
                bomb_range: p.bomb_range,
                max_bombs: p.max_bombs,
                color_index: p.color_index,
            })
            .collect()
    }
}

fn can_move_to(map: &GameMap, _bombs: &[Bomb], x: f64, y: f64, _player_id: u8) -> bool {
    let margin = 0.15;
    let corners = [
        (x + margin, y + margin),
        (x + 1.0 - margin, y + margin),
        (x + margin, y + 1.0 - margin),
        (x + 1.0 - margin, y + 1.0 - margin),
    ];

    for (cx, cy) in &corners {
        let col = *cx as usize;
        let row = *cy as usize;
        if col >= MAP_COLS || row >= MAP_ROWS {
            return false;
        }
        match map[row][col] {
            Tile::HardBlock | Tile::SoftBlock => return false,
            _ => {}
        }
    }

    true
}

use shared::map::Tile;
use shared::protocol::*;

use crate::input::InputState;
use crate::network::Network;
use crate::renderer::Renderer;

#[derive(PartialEq)]
enum Phase {
    Connecting,
    Waiting { count: usize, need: usize },
    Playing,
    GameOver { winner: Option<u8> },
}

pub struct Game {
    renderer: Renderer,
    input: InputState,
    network: Network,
    phase: Phase,
    my_id: u8,
    map: Vec<Vec<Tile>>,
    players: Vec<PlayerState>,
    bombs: Vec<BombState>,
    explosions: Vec<ExplosionState>,
    items: Vec<ItemState>,
    tick: u64,
    last_dx: f64,
    last_dy: f64,
    sent_join: bool,
}

impl Game {
    pub fn new(canvas: &web_sys::HtmlCanvasElement, ws_url: &str) -> Self {
        Game {
            renderer: Renderer::new(canvas),
            input: InputState::new(),
            network: Network::new(ws_url),
            phase: Phase::Connecting,
            my_id: 0,
            map: Vec::new(),
            players: Vec::new(),
            bombs: Vec::new(),
            explosions: Vec::new(),
            items: Vec::new(),
            tick: 0,
            last_dx: 0.0,
            last_dy: 0.0,
            sent_join: false,
        }
    }

    pub fn update(&mut self) {
        self.process_network();

        match &self.phase {
            Phase::Connecting => {
                if self.network.is_connected() && !self.sent_join {
                    let name = format!("Player{}", (js_sys::Math::random() * 999.0) as u32);
                    self.network.send(&ClientMsg::Join { name });
                    self.sent_join = true;
                }
            }
            Phase::Waiting { .. } => {}
            Phase::Playing => {
                let (dx, dy) = self.input.get_movement();
                if dx != self.last_dx || dy != self.last_dy {
                    self.network.send(&ClientMsg::Move { dx, dy });
                    self.last_dx = dx;
                    self.last_dy = dy;
                }
                if self.input.wants_bomb() {
                    self.network.send(&ClientMsg::PlaceBomb);
                }
            }
            Phase::GameOver { .. } => {}
        }

        self.input.clear_frame();
    }

    pub fn render(&self) {
        match &self.phase {
            Phase::Connecting => {
                self.renderer.draw_waiting(0, 2);
            }
            Phase::Waiting { count, need } => {
                self.renderer.draw_waiting(*count, *need);
            }
            Phase::Playing => {
                self.renderer.clear();
                self.renderer.draw_map(&self.map);
                self.renderer.draw_items(&self.items);
                self.renderer.draw_bombs(&self.bombs, self.tick);
                self.renderer.draw_explosions(&self.explosions);
                self.renderer.draw_players(&self.players);
                self.renderer.draw_hud(&self.players, self.my_id);
            }
            Phase::GameOver { winner } => {
                self.renderer.clear();
                self.renderer.draw_map(&self.map);
                self.renderer.draw_players(&self.players);
                self.renderer.draw_game_over(*winner, &self.players);
            }
        }
    }

    fn process_network(&mut self) {
        let messages = self.network.poll();
        for msg in messages {
            match msg {
                ServerMsg::Welcome { player_id, .. } => {
                    self.my_id = player_id;
                }
                ServerMsg::Waiting {
                    player_count,
                    need,
                } => {
                    self.phase = Phase::Waiting {
                        count: player_count,
                        need,
                    };
                }
                ServerMsg::GameStart { map, players } => {
                    self.map = map;
                    self.players = players;
                    self.phase = Phase::Playing;
                }
                ServerMsg::GameState {
                    players,
                    bombs,
                    explosions,
                    items,
                    map,
                    tick,
                } => {
                    self.players = players;
                    self.bombs = bombs;
                    self.explosions = explosions;
                    self.items = items;
                    self.map = map;
                    self.tick = tick;
                }
                ServerMsg::GameOver { winner } => {
                    self.phase = Phase::GameOver { winner };
                }
            }
        }
    }
}

use shared::constants::*;
use shared::map::Tile;
use shared::protocol::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::sprites::SpriteSheet;

pub struct Renderer {
    ctx: CanvasRenderingContext2d,
    sprites: SpriteSheet,
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        ctx.set_image_smoothing_enabled(false);
        Renderer {
            ctx,
            sprites: SpriteSheet::generate(),
        }
    }

    pub fn clear(&self) {
        self.ctx.clear_rect(
            0.0,
            0.0,
            CANVAS_WIDTH as f64,
            CANVAS_HEIGHT as f64,
        );
    }

    pub fn draw_map(&self, map: &[Vec<Tile>]) {
        for (r, row) in map.iter().enumerate() {
            for (c, tile) in row.iter().enumerate() {
                let x = (c as u32 * TILE_SIZE) as f64;
                let y = (r as u32 * TILE_SIZE) as f64;

                self.ctx
                    .draw_image_with_html_canvas_element(&self.sprites.ground, x, y)
                    .unwrap();

                match tile {
                    Tile::HardBlock => {
                        self.ctx
                            .draw_image_with_html_canvas_element(&self.sprites.hard_block, x, y)
                            .unwrap();
                    }
                    Tile::SoftBlock => {
                        self.ctx
                            .draw_image_with_html_canvas_element(&self.sprites.soft_block, x, y)
                            .unwrap();
                    }
                    Tile::Empty => {}
                }
            }
        }
    }

    pub fn draw_items(&self, items: &[ItemState]) {
        for item in items {
            let x = (item.x as u32 * TILE_SIZE) as f64;
            let y = (item.y as u32 * TILE_SIZE) as f64;
            let sprite = match item.kind {
                ItemKind::BombRange => &self.sprites.item_range,
                ItemKind::BombCount => &self.sprites.item_bomb,
                ItemKind::Speed => &self.sprites.item_speed,
            };
            self.ctx
                .draw_image_with_html_canvas_element(sprite, x, y)
                .unwrap();
        }
    }

    pub fn draw_bombs(&self, bombs: &[BombState], tick: u64) {
        for bomb in bombs {
            let x = (bomb.x as u32 * TILE_SIZE) as f64;
            let y = (bomb.y as u32 * TILE_SIZE) as f64;
            let frame = ((tick / 5) % 3) as usize;
            self.ctx
                .draw_image_with_html_canvas_element(&self.sprites.bomb[frame], x, y)
                .unwrap();
        }
    }

    pub fn draw_explosions(&self, explosions: &[ExplosionState]) {
        for exp in explosions {
            let x = (exp.x as u32 * TILE_SIZE) as f64;
            let y = (exp.y as u32 * TILE_SIZE) as f64;
            self.ctx
                .draw_image_with_html_canvas_element(&self.sprites.explosion_center, x, y)
                .unwrap();
        }
    }

    pub fn draw_players(&self, players: &[PlayerState]) {
        for p in players {
            if !p.alive {
                continue;
            }
            let idx = p.color_index as usize;
            if idx < self.sprites.players.len() {
                self.ctx
                    .draw_image_with_html_canvas_element(
                        &self.sprites.players[idx],
                        p.x * TILE_SIZE as f64,
                        p.y * TILE_SIZE as f64,
                    )
                    .unwrap();
            }

            // Name tag
            self.ctx.set_fill_style_str("#FFFFFF");
            self.ctx.set_font("bold 10px monospace");
            self.ctx.set_text_align("center");
            let _ = self.ctx.fill_text(
                &p.name,
                p.x * TILE_SIZE as f64 + TILE_SIZE as f64 / 2.0,
                p.y * TILE_SIZE as f64 - 2.0,
            );
        }
    }

    pub fn draw_hud(&self, players: &[PlayerState], my_id: u8) {
        let y_base = CANVAS_HEIGHT as f64 + 5.0;
        self.ctx.set_font("12px monospace");
        self.ctx.set_text_align("left");

        for (i, p) in players.iter().enumerate() {
            let x = 10.0 + i as f64 * 130.0;
            let color = PLAYER_COLORS
                .get(p.color_index as usize)
                .unwrap_or(&"#FFF");
            self.ctx.set_fill_style_str(color);

            let marker = if p.id == my_id { " (YOU)" } else { "" };
            let status = if p.alive { "" } else { " [X]" };
            let text = format!("{}{}{}", p.name, marker, status);
            let _ = self.ctx.fill_text(&text, x, y_base + 14.0);
        }
    }

    pub fn draw_waiting(&self, count: usize, need: usize) {
        self.clear();
        self.ctx.set_fill_style_str("#1a1a2e");
        self.ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH as f64, CANVAS_HEIGHT as f64);

        self.ctx.set_fill_style_str("#FFFFFF");
        self.ctx.set_font("bold 24px monospace");
        self.ctx.set_text_align("center");
        let _ = self.ctx.fill_text(
            "泡泡堂 Online",
            CANVAS_WIDTH as f64 / 2.0,
            CANVAS_HEIGHT as f64 / 2.0 - 40.0,
        );

        self.ctx.set_font("16px monospace");
        let _ = self.ctx.fill_text(
            &format!("等待玩家... {}/{}", count, need),
            CANVAS_WIDTH as f64 / 2.0,
            CANVAS_HEIGHT as f64 / 2.0 + 10.0,
        );

        self.ctx.set_fill_style_str("#AAAAAA");
        self.ctx.set_font("12px monospace");
        let _ = self.ctx.fill_text(
            "方向键/WASD 移动 | 空格 放泡泡",
            CANVAS_WIDTH as f64 / 2.0,
            CANVAS_HEIGHT as f64 / 2.0 + 50.0,
        );
    }

    pub fn draw_game_over(&self, winner: Option<u8>, players: &[PlayerState]) {
        self.ctx.set_fill_style_str("rgba(0,0,0,0.7)");
        self.ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH as f64, CANVAS_HEIGHT as f64);

        self.ctx.set_fill_style_str("#FFD700");
        self.ctx.set_font("bold 32px monospace");
        self.ctx.set_text_align("center");
        let _ = self.ctx.fill_text(
            "GAME OVER",
            CANVAS_WIDTH as f64 / 2.0,
            CANVAS_HEIGHT as f64 / 2.0 - 30.0,
        );

        self.ctx.set_font("20px monospace");
        let winner_text = match winner {
            Some(id) => {
                let name = players
                    .iter()
                    .find(|p| p.id == id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("???");
                format!("Winner: {}!", name)
            }
            None => "Draw!".to_string(),
        };
        self.ctx.set_fill_style_str("#FFFFFF");
        let _ = self.ctx.fill_text(
            &winner_text,
            CANVAS_WIDTH as f64 / 2.0,
            CANVAS_HEIGHT as f64 / 2.0 + 10.0,
        );
    }
}

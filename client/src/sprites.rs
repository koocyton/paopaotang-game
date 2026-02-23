use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn create_offscreen_canvas(w: u32, h: u32) -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    canvas.set_width(w);
    canvas.set_height(h);
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    (canvas, ctx)
}

fn fill_rect(ctx: &CanvasRenderingContext2d, color: &str, x: f64, y: f64, w: f64, h: f64) {
    ctx.set_fill_style_str(color);
    ctx.fill_rect(x, y, w, h);
}

fn draw_pixel(ctx: &CanvasRenderingContext2d, color: &str, px: u32, py: u32, scale: u32) {
    let s = scale as f64;
    fill_rect(ctx, color, px as f64 * s, py as f64 * s, s, s);
}

pub struct SpriteSheet {
    pub ground: HtmlCanvasElement,
    pub hard_block: HtmlCanvasElement,
    pub soft_block: HtmlCanvasElement,
    pub players: Vec<HtmlCanvasElement>, // 4 players
    pub bomb: Vec<HtmlCanvasElement>,    // 3 frames
    pub explosion_center: HtmlCanvasElement,
    pub explosion_h: HtmlCanvasElement,
    pub explosion_v: HtmlCanvasElement,
    pub explosion_end_l: HtmlCanvasElement,
    pub explosion_end_r: HtmlCanvasElement,
    pub explosion_end_u: HtmlCanvasElement,
    pub explosion_end_d: HtmlCanvasElement,
    pub item_range: HtmlCanvasElement,
    pub item_bomb: HtmlCanvasElement,
    pub item_speed: HtmlCanvasElement,
}

impl SpriteSheet {
    pub fn generate() -> Self {
        SpriteSheet {
            ground: draw_ground(),
            hard_block: draw_hard_block(),
            soft_block: draw_soft_block(),
            players: vec![
                draw_player("#FF4444", "#CC2222", "#FFCCCC"),
                draw_player("#4444FF", "#2222CC", "#CCCCFF"),
                draw_player("#44BB44", "#228822", "#CCFFCC"),
                draw_player("#FFAA00", "#CC8800", "#FFEECC"),
            ],
            bomb: vec![draw_bomb(0), draw_bomb(1), draw_bomb(2)],
            explosion_center: draw_explosion_center(),
            explosion_h: draw_explosion_arm(true),
            explosion_v: draw_explosion_arm(false),
            explosion_end_l: draw_explosion_end(0),
            explosion_end_r: draw_explosion_end(1),
            explosion_end_u: draw_explosion_end(2),
            explosion_end_d: draw_explosion_end(3),
            item_range: draw_item_range(),
            item_bomb: draw_item_bomb(),
            item_speed: draw_item_speed(),
        }
    }
}

fn draw_ground() -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    fill_rect(&ctx, "#8BC34A", 0.0, 0.0, 32.0, 32.0);
    // subtle grass pattern
    fill_rect(&ctx, "#7CB342", 2.0, 3.0, 2.0, 2.0);
    fill_rect(&ctx, "#7CB342", 14.0, 8.0, 2.0, 2.0);
    fill_rect(&ctx, "#7CB342", 24.0, 20.0, 2.0, 2.0);
    fill_rect(&ctx, "#7CB342", 8.0, 26.0, 2.0, 2.0);
    fill_rect(&ctx, "#9CCC65", 20.0, 4.0, 2.0, 2.0);
    fill_rect(&ctx, "#9CCC65", 6.0, 16.0, 2.0, 2.0);
    fill_rect(&ctx, "#9CCC65", 28.0, 12.0, 2.0, 2.0);
    canvas
}

fn draw_hard_block() -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    fill_rect(&ctx, "#616161", 0.0, 0.0, 32.0, 32.0);
    fill_rect(&ctx, "#757575", 1.0, 1.0, 30.0, 30.0);
    // brick lines
    fill_rect(&ctx, "#616161", 0.0, 15.0, 32.0, 2.0);
    fill_rect(&ctx, "#616161", 15.0, 0.0, 2.0, 32.0);
    // highlights
    fill_rect(&ctx, "#9E9E9E", 1.0, 1.0, 14.0, 2.0);
    fill_rect(&ctx, "#9E9E9E", 1.0, 1.0, 2.0, 14.0);
    fill_rect(&ctx, "#9E9E9E", 17.0, 1.0, 14.0, 2.0);
    fill_rect(&ctx, "#9E9E9E", 1.0, 17.0, 14.0, 2.0);
    canvas
}

fn draw_soft_block() -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    fill_rect(&ctx, "#8D6E63", 0.0, 0.0, 32.0, 32.0);
    fill_rect(&ctx, "#A1887F", 1.0, 1.0, 30.0, 30.0);
    // cross pattern
    fill_rect(&ctx, "#8D6E63", 0.0, 10.0, 32.0, 1.0);
    fill_rect(&ctx, "#8D6E63", 0.0, 21.0, 32.0, 1.0);
    fill_rect(&ctx, "#8D6E63", 10.0, 0.0, 1.0, 32.0);
    fill_rect(&ctx, "#8D6E63", 21.0, 0.0, 1.0, 32.0);
    // highlight
    fill_rect(&ctx, "#BCAAA4", 2.0, 2.0, 7.0, 2.0);
    fill_rect(&ctx, "#BCAAA4", 12.0, 2.0, 8.0, 2.0);
    fill_rect(&ctx, "#BCAAA4", 2.0, 12.0, 7.0, 2.0);
    canvas
}

fn draw_player(main_color: &str, dark_color: &str, light_color: &str) -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    let s = 2u32; // pixel scale

    // Head (8x8 logical pixels centered)
    for py in 1..5 {
        for px in 5..11 {
            draw_pixel(&ctx, light_color, px, py, s);
        }
    }
    // Eyes
    draw_pixel(&ctx, "#333333", 6, 2, s);
    draw_pixel(&ctx, "#333333", 9, 2, s);
    // Mouth
    draw_pixel(&ctx, "#333333", 7, 3, s);
    draw_pixel(&ctx, "#333333", 8, 3, s);

    // Body
    for py in 5..10 {
        for px in 5..11 {
            draw_pixel(&ctx, main_color, px, py, s);
        }
    }
    // Belt
    for px in 5..11 {
        draw_pixel(&ctx, dark_color, px, 8, s);
    }

    // Arms
    for py in 5..8 {
        draw_pixel(&ctx, light_color, 4, py, s);
        draw_pixel(&ctx, light_color, 11, py, s);
    }

    // Legs
    for py in 10..13 {
        draw_pixel(&ctx, dark_color, 6, py, s);
        draw_pixel(&ctx, dark_color, 7, py, s);
        draw_pixel(&ctx, dark_color, 8, py, s);
        draw_pixel(&ctx, dark_color, 9, py, s);
    }

    // Shoes
    draw_pixel(&ctx, "#333333", 5, 13, s);
    draw_pixel(&ctx, "#333333", 6, 13, s);
    draw_pixel(&ctx, "#333333", 7, 13, s);
    draw_pixel(&ctx, "#333333", 8, 13, s);
    draw_pixel(&ctx, "#333333", 9, 13, s);
    draw_pixel(&ctx, "#333333", 10, 13, s);

    canvas
}

fn draw_bomb(frame: u32) -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    let size = match frame {
        0 => 8.0,
        1 => 10.0,
        _ => 12.0,
    };
    let _offset = (32.0 - size * 2.0) / 2.0;

    // Body
    ctx.begin_path();
    ctx.arc(16.0, 18.0, size, 0.0, std::f64::consts::PI * 2.0)
        .unwrap();
    ctx.set_fill_style_str("#333333");
    ctx.fill();

    // Highlight
    ctx.begin_path();
    ctx.arc(13.0, 15.0, size * 0.3, 0.0, std::f64::consts::PI * 2.0)
        .unwrap();
    ctx.set_fill_style_str("#666666");
    ctx.fill();

    // Fuse
    fill_rect(&ctx, "#8D6E63", 14.0, 2.0, 4.0, 6.0);
    // Spark
    let spark_colors = ["#FF6600", "#FFAA00", "#FFFF00"];
    let sc = spark_colors[frame as usize % 3];
    fill_rect(&ctx, sc, 13.0, 0.0, 6.0, 4.0);
    fill_rect(&ctx, "#FFFFFF", 15.0, 1.0, 2.0, 2.0);

    canvas
}

fn draw_explosion_center() -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    fill_rect(&ctx, "#FF6600", 0.0, 0.0, 32.0, 32.0);
    fill_rect(&ctx, "#FFAA00", 4.0, 4.0, 24.0, 24.0);
    fill_rect(&ctx, "#FFFF00", 8.0, 8.0, 16.0, 16.0);
    fill_rect(&ctx, "#FFFFFF", 12.0, 12.0, 8.0, 8.0);
    canvas
}

fn draw_explosion_arm(horizontal: bool) -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    if horizontal {
        fill_rect(&ctx, "#FF6600", 0.0, 4.0, 32.0, 24.0);
        fill_rect(&ctx, "#FFAA00", 0.0, 8.0, 32.0, 16.0);
        fill_rect(&ctx, "#FFFF00", 0.0, 12.0, 32.0, 8.0);
    } else {
        fill_rect(&ctx, "#FF6600", 4.0, 0.0, 24.0, 32.0);
        fill_rect(&ctx, "#FFAA00", 8.0, 0.0, 16.0, 32.0);
        fill_rect(&ctx, "#FFFF00", 12.0, 0.0, 8.0, 32.0);
    }
    canvas
}

fn draw_explosion_end(dir: u32) -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    match dir {
        0 => {
            // left end
            fill_rect(&ctx, "#FF6600", 8.0, 4.0, 24.0, 24.0);
            fill_rect(&ctx, "#FFAA00", 12.0, 8.0, 20.0, 16.0);
            fill_rect(&ctx, "#FFFF00", 16.0, 12.0, 16.0, 8.0);
        }
        1 => {
            // right end
            fill_rect(&ctx, "#FF6600", 0.0, 4.0, 24.0, 24.0);
            fill_rect(&ctx, "#FFAA00", 0.0, 8.0, 20.0, 16.0);
            fill_rect(&ctx, "#FFFF00", 0.0, 12.0, 16.0, 8.0);
        }
        2 => {
            // up end
            fill_rect(&ctx, "#FF6600", 4.0, 8.0, 24.0, 24.0);
            fill_rect(&ctx, "#FFAA00", 8.0, 12.0, 16.0, 20.0);
            fill_rect(&ctx, "#FFFF00", 12.0, 16.0, 8.0, 16.0);
        }
        _ => {
            // down end
            fill_rect(&ctx, "#FF6600", 4.0, 0.0, 24.0, 24.0);
            fill_rect(&ctx, "#FFAA00", 8.0, 0.0, 16.0, 20.0);
            fill_rect(&ctx, "#FFFF00", 12.0, 0.0, 8.0, 16.0);
        }
    }
    canvas
}

fn draw_item_range() -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    // Background
    fill_rect(&ctx, "#E3F2FD", 2.0, 2.0, 28.0, 28.0);
    fill_rect(&ctx, "#BBDEFB", 4.0, 4.0, 24.0, 24.0);
    // Flame icon
    fill_rect(&ctx, "#FF6600", 12.0, 6.0, 8.0, 4.0);
    fill_rect(&ctx, "#FF6600", 10.0, 10.0, 12.0, 6.0);
    fill_rect(&ctx, "#FF6600", 12.0, 16.0, 8.0, 4.0);
    fill_rect(&ctx, "#FFAA00", 14.0, 8.0, 4.0, 10.0);
    fill_rect(&ctx, "#FFFF00", 15.0, 10.0, 2.0, 6.0);
    // Arrow up
    fill_rect(&ctx, "#FF0000", 14.0, 22.0, 4.0, 4.0);
    fill_rect(&ctx, "#FF0000", 12.0, 24.0, 8.0, 2.0);
    canvas
}

fn draw_item_bomb() -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    fill_rect(&ctx, "#F3E5F5", 2.0, 2.0, 28.0, 28.0);
    fill_rect(&ctx, "#E1BEE7", 4.0, 4.0, 24.0, 24.0);
    // Mini bomb
    ctx.begin_path();
    ctx.arc(16.0, 18.0, 7.0, 0.0, std::f64::consts::PI * 2.0)
        .unwrap();
    ctx.set_fill_style_str("#333333");
    ctx.fill();
    ctx.begin_path();
    ctx.arc(14.0, 16.0, 2.0, 0.0, std::f64::consts::PI * 2.0)
        .unwrap();
    ctx.set_fill_style_str("#666666");
    ctx.fill();
    // +1 text
    ctx.set_fill_style_str("#FFFFFF");
    ctx.set_font("bold 10px monospace");
    let _ = ctx.fill_text("+1", 6.0, 14.0);
    canvas
}

fn draw_item_speed() -> HtmlCanvasElement {
    let (canvas, ctx) = create_offscreen_canvas(32, 32);
    fill_rect(&ctx, "#FFF3E0", 2.0, 2.0, 28.0, 28.0);
    fill_rect(&ctx, "#FFE0B2", 4.0, 4.0, 24.0, 24.0);
    // Shoe shape
    fill_rect(&ctx, "#5D4037", 8.0, 10.0, 6.0, 12.0);
    fill_rect(&ctx, "#5D4037", 14.0, 16.0, 10.0, 6.0);
    fill_rect(&ctx, "#795548", 9.0, 11.0, 4.0, 4.0);
    // Speed lines
    fill_rect(&ctx, "#FF9800", 4.0, 12.0, 3.0, 1.0);
    fill_rect(&ctx, "#FF9800", 3.0, 16.0, 4.0, 1.0);
    fill_rect(&ctx, "#FF9800", 4.0, 20.0, 3.0, 1.0);
    canvas
}

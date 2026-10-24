//! Burning away (the Grimoire's page change, the first-time setup's unpicked options): the
//! old picture is painted offscreen and laid back cell by cell where the fire hasn't reached,
//! with a front of embers and a scorch ahead of it.

use crate::kit;
use macroquad::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// Canvases the burning pictures are painted on, by (slot, w, h).
    static TARGETS: RefCell<HashMap<(usize, u32, u32), RenderTarget>> =
        RefCell::new(HashMap::new());
}

/// Size of a cell of the burn.
const ASH_CELL: f32 = 2.0;
const CHARRED: Color = Color::new(0.1, 0.04, 0.02, 1.0);
const SCORCH: Color = Color::new(0.42, 0.24, 0.11, 1.0);
const EMBER: Color = Color::new(1.0, 0.42, 0.08, 1.0);
const EMBER_HOT: Color = Color::new(1.0, 0.85, 0.4, 1.0);

/// Something burning away: `draw_old` paints it (in screen coordinates inside `rect`) onto an
/// offscreen canvas, which is laid back only where the fire hasn't reached yet. From where it
/// caught, a front of embers spreads; ahead of it the surface chars brown, behind it whatever
/// was drawn underneath shows. `k` is 0..1 of the burn. Each `slot` has its own canvas, so
/// several things can burn in one frame.
pub fn burn(
    rect: Rect,
    k: f32,
    seed: u32,
    slot: usize,
    draw_old: impl FnOnce(),
    t: f32,
    alpha: f32,
) {
    let page = Rect::new(
        rect.x.round(),
        rect.y.round(),
        rect.w.round(),
        rect.h.round(),
    );
    let (w, h) = (page.w as u32, page.h as u32);
    let target = TARGETS.with(|cell| {
        let mut targets = cell.borrow_mut();
        let rt = targets.entry((slot, w, h)).or_insert_with(|| {
            let rt = render_target(w, h);
            rt.texture.set_filter(FilterMode::Nearest);
            rt
        });
        rt.clone()
    });
    push_camera_state();
    let mut cam = Camera2D::from_display_rect(page);
    cam.zoom.y = -cam.zoom.y;
    cam.render_target = Some(target.clone());
    set_camera(&cam);
    clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
    draw_old();
    pop_camera_state();

    let front = k.clamp(0.0, 1.0) * 1.15;
    const CHAR: f32 = 0.1;
    let cols = (page.w / ASH_CELL).ceil() as i32;
    let rows = (page.h / ASH_CELL).ceil() as i32;
    let ignite = vec2(
        0.2 + 0.6 * hash(seed * 3 + 1),
        0.55 + 0.4 * hash(seed * 3 + 2),
    );
    let threshold = |col: i32, row: i32| {
        let d = (vec2(col as f32 / cols as f32, row as f32 / rows as f32) - ignite).length() / 0.95;
        let noise = value_noise(col as f32 / 8.0, row as f32 / 8.0, seed) * 0.7
            + value_noise(col as f32 / 3.0, row as f32 / 3.0, seed + 77) * 0.3;
        (0.6 * d + 0.4 * noise).clamp(0.0, 1.0)
    };
    for row in 0..rows {
        let y = row as f32 * ASH_CELL;
        let ch = ASH_CELL.min(page.h - y);
        // runs of cells the fire hasn't reached are laid back in one piece
        let mut run: Option<i32> = None;
        for col in 0..=cols {
            let unburnt = col < cols && threshold(col, row) > front;
            match (unburnt, run) {
                (true, None) => run = Some(col),
                (false, Some(start)) => {
                    let x = start as f32 * ASH_CELL;
                    let rw = (col as f32 * ASH_CELL).min(page.w) - x;
                    let params = DrawTextureParams {
                        source: Some(Rect::new(x, y, rw, ch)),
                        dest_size: Some(vec2(rw, ch)),
                        ..Default::default()
                    };
                    draw_texture_ex(
                        &target.texture,
                        page.x + x,
                        page.y + y,
                        kit::fade(WHITE, alpha),
                        params,
                    );
                    run = None;
                }
                _ => {}
            }
        }
        // the scorch ahead of the front, the embers on it
        for col in 0..cols {
            let ahead = threshold(col, row) - front;
            let color = if (-0.012..0.005).contains(&ahead) {
                EMBER_HOT
            } else if (0.005..0.018).contains(&ahead) {
                EMBER
            } else if (0.018..CHAR).contains(&ahead) {
                // charred black at the flames, browning out to a faint scorch
                let k = (ahead - 0.018) / (CHAR - 0.018);
                let c = mix(CHARRED, SCORCH, k);
                Color::new(c.r, c.g, c.b, 0.95 * (1.0 - k * k))
            } else {
                continue;
            };
            let x = col as f32 * ASH_CELL;
            let cw = ASH_CELL.min(page.w - x);
            draw_rectangle(
                page.x + x,
                page.y + y,
                cw,
                ch,
                Color::new(color.r, color.g, color.b, color.a * alpha),
            );
        }
    }
    kit::sparks(page, t * 2.0, alpha);
}

/// Smooth value noise in 0..1 on a grid of hashed corners.
pub fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let corner = |dx: i32, dy: i32| {
        let ix = (x0 as i32 + dx) as u32;
        let iy = (y0 as i32 + dy) as u32;
        hash(ix.wrapping_mul(73_856_093) ^ iy.wrapping_mul(19_349_663) ^ seed)
    };
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * sx;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * sx;
    top + (bottom - top) * sy
}

fn mix(a: Color, b: Color, k: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * k,
        a.g + (b.g - a.g) * k,
        a.b + (b.b - a.b) * k,
        a.a,
    )
}

fn hash(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x5bd1_e995;
    x ^= x >> 15;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    (x & 0xFFFF) as f32 / 65535.0
}

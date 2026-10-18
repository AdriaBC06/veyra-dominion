mod config;
mod sprites;

use config::*;
use macroquad::prelude::*;
use sprites::{sprite, SpriteId, GROUND_TILE, GROUND_VARIANTS};

fn window_conf() -> Conf {
    Conf {
        window_title: GAME_TITLE.to_owned(),
        window_width: WINDOW_W,
        window_height: WINDOW_H,
        window_resizable: true,
        ..Default::default()
    }
}

/// Camera that renders the logical 640x360 frame into the render target.
/// Render targets are stored flipped, hence the inverted y zoom.
fn target_camera(target: &RenderTarget, origin: Vec2) -> Camera2D {
    let mut cam = Camera2D::from_display_rect(Rect::new(origin.x, origin.y, LOGICAL_W, LOGICAL_H));
    cam.zoom.y = -cam.zoom.y;
    cam.render_target = Some(target.clone());
    cam
}

/// Draws the logical frame centred in the window, integer-scaled when it fits.
fn present(target: &RenderTarget) {
    set_default_camera();
    clear_background(BLACK);
    let (sw, sh) = (screen_width(), screen_height());
    let mut scale = (sw / LOGICAL_W).min(sh / LOGICAL_H);
    if scale >= 1.0 {
        scale = scale.floor();
    }
    let (w, h) = (LOGICAL_W * scale, LOGICAL_H * scale);
    draw_texture_ex(
        &target.texture,
        ((sw - w) * 0.5).floor(),
        ((sh - h) * 0.5).floor(),
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        },
    );
}

/// Draws a sprite frame with its top-left corner at `pos`.
fn draw_sprite(id: SpriteId, frame: usize, pos: Vec2, flip_x: bool) {
    let (tex, src) = sprite(id, frame);
    draw_texture_ex(
        &tex,
        pos.x.round(),
        pos.y.round(),
        WHITE,
        DrawTextureParams {
            source: Some(src),
            flip_x,
            ..Default::default()
        },
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    sprites::init();
    let target = render_target(LOGICAL_W as u32, LOGICAL_H as u32);
    target.texture.set_filter(FilterMode::Nearest);

    let mut accumulator = 0.0f32;
    let mut sim_time = 0.0f32;

    loop {
        accumulator += get_frame_time().min(MAX_FRAME_DT);
        let mut ticks = 0;
        while accumulator >= TICK_DT && ticks < MAX_TICKS_PER_FRAME {
            sim_time += TICK_DT;
            accumulator -= TICK_DT;
            ticks += 1;
        }
        if ticks == MAX_TICKS_PER_FRAME {
            accumulator = 0.0;
        }

        set_camera(&target_camera(&target, Vec2::ZERO));
        clear_background(BLACK);
        let tile = GROUND_TILE as f32;
        for ty in 0..(LOGICAL_H / tile).ceil() as usize {
            for tx in 0..(LOGICAL_W / tile).ceil() as usize {
                let v = (tx * 7 + ty * 13) % GROUND_VARIANTS;
                draw_sprite(
                    SpriteId::Ground,
                    v,
                    vec2(tx as f32, ty as f32) * tile,
                    false,
                );
            }
        }
        // Atlas preview: rest, impulse right, impulse left.
        let wing_frame = (sim_time * 3.0) as usize;
        for (i, (frame, flip)) in [(0, false), (1, false), (1, true)].into_iter().enumerate() {
            let pos = vec2(
                260.0 + i as f32 * 48.0,
                160.0 + (sim_time * 1.5 * std::f32::consts::TAU).sin() * 2.0,
            );
            draw_sprite(
                SpriteId::VeyraWings,
                wing_frame,
                pos + vec2(-4.0, 6.0),
                flip,
            );
            draw_sprite(SpriteId::VeyraTail, 0, pos + vec2(10.0, 15.0), flip);
            draw_sprite(SpriteId::VeyraBody, frame, pos, flip);
        }
        draw_text(GAME_TITLE, 16.0, 32.0, 32.0, WHITE);

        present(&target);
        next_frame().await;
    }
}

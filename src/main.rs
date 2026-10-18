mod config;
mod game;
mod sprites;

use config::*;
use game::{Game, InputState};
use macroquad::prelude::*;
use sprites::{sprite, SpriteId, GROUND_TILE};
use std::f32::consts::TAU;

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

fn read_input() -> InputState {
    let key = |a: KeyCode, b: KeyCode| (is_key_down(a) || is_key_down(b)) as i32 as f32;
    InputState {
        move_dir: vec2(
            key(KeyCode::D, KeyCode::Right) - key(KeyCode::A, KeyCode::Left),
            key(KeyCode::S, KeyCode::Down) - key(KeyCode::W, KeyCode::Up),
        ),
    }
}

/// Fills the visible area with procedural ground tiles.
fn draw_ground(origin: Vec2) {
    let tile = GROUND_TILE as f32;
    let (x0, y0) = (
        (origin.x / tile).floor() as i32,
        (origin.y / tile).floor() as i32,
    );
    let (cols, rows) = ((LOGICAL_W / tile) as i32 + 2, (LOGICAL_H / tile) as i32 + 2);
    for iy in y0..y0 + rows {
        for ix in x0..x0 + cols {
            let v = sprites::ground_variant(ix, iy);
            draw_sprite(
                SpriteId::Ground,
                v,
                vec2(ix as f32, iy as f32) * tile,
                false,
            );
        }
    }
}

/// Draws Veyra hovering over `pos` (the point on the ground below her).
fn draw_veyra(game: &Game, pos: Vec2, t: f32) {
    let p = &game.player;
    let bob = BOB_AMPLITUDE * (TAU * BOB_HZ * t).sin();
    let shrink = 1.0 - SHADOW_SHRINK * bob / BOB_AMPLITUDE;
    draw_ellipse(
        pos.x,
        pos.y,
        6.0 * shrink,
        2.0 * shrink,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );

    let body = pos - vec2(8.0, 24.0 + HOVER_HEIGHT + bob.round());
    let moving = p.vel.length() > IMPULSE_SPEED;
    let facing_left = p.facing < 0.0;
    // The tail hangs on the side opposite to where she faces, so it trails behind.
    let tail_x = if facing_left { 10.0 } else { -2.0 };
    draw_sprite(
        SpriteId::VeyraWings,
        (t * WING_FPS) as usize,
        body + vec2(-4.0, 6.0),
        false,
    );
    draw_sprite(
        SpriteId::VeyraTail,
        0,
        body + vec2(tail_x, 15.0) + p.tail_lag,
        !facing_left,
    );
    draw_sprite(SpriteId::VeyraBody, moving as usize, body, facing_left);
}

#[macroquad::main(window_conf)]
async fn main() {
    sprites::init();
    let target = render_target(LOGICAL_W as u32, LOGICAL_H as u32);
    target.texture.set_filter(FilterMode::Nearest);

    let mut game = Game::new();
    let mut accumulator = 0.0f32;

    loop {
        let input = read_input();
        accumulator += get_frame_time().min(MAX_FRAME_DT);
        let mut ticks = 0;
        while accumulator >= TICK_DT && ticks < MAX_TICKS_PER_FRAME {
            game.tick(&input);
            accumulator -= TICK_DT;
            ticks += 1;
        }
        if ticks == MAX_TICKS_PER_FRAME {
            accumulator = 0.0;
        }

        let alpha = accumulator / TICK_DT;
        let player_pos = game.player.prev_pos.lerp(game.player.pos, alpha);
        let origin = (player_pos - vec2(LOGICAL_W, LOGICAL_H) * 0.5).round();
        set_camera(&target_camera(&target, origin));
        clear_background(BLACK);
        draw_ground(origin);
        draw_veyra(&game, player_pos.round(), game.time + alpha * TICK_DT);

        present(&target);
        next_frame().await;
    }
}

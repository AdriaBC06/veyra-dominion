mod assets;
mod config;
mod enemies;
mod game;
mod hud;
mod spatial;
mod upgrades;

use assets::{sprite, SpriteId, FLASH_FRAME_OFFSET, GROUND_TILE};
use config::*;
use game::{Game, InputState, SimEvent};
use macroquad::prelude::*;
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
    draw_sprite_tinted(id, frame, pos, flip_x, WHITE);
}

fn draw_sprite_tinted(id: SpriteId, frame: usize, pos: Vec2, flip_x: bool, color: Color) {
    let (tex, src) = sprite(id, frame);
    draw_texture_ex(
        &tex,
        pos.x.round(),
        pos.y.round(),
        color,
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
            let v = assets::ground_variant(ix, iy);
            draw_sprite(
                SpriteId::Ground,
                v,
                vec2(ix as f32, iy as f32) * tile,
                false,
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppState {
    Title,
    Playing,
    Paused,
    LevelUp,
    GameOver,
    Victory,
}

fn draw_shadow(pos: Vec2, half_w: f32, half_h: f32, opacity: f32) {
    let color = Color::new(0.0, 0.0, 0.0, 0.35 * opacity);
    draw_ellipse(pos.x, pos.y, half_w, half_h, 0.0, color);
}

/// Enemies are drawn with their feet at their position, facing Veyra.
fn draw_enemies(game: &Game) {
    let e = &game.enemies;
    let px = game.player.pos.x;
    for i in 0..e.len() {
        let pos = e.pos(i).round();
        let def = &ENEMY_DEFS[e.kind[i] as usize];
        let mut frame = (game.time * ENEMY_ANIM_FPS + e.phase[i]) as usize % 2;
        if e.flash[i] > 0.0 {
            frame += FLASH_FRAME_OFFSET;
        }
        draw_shadow(pos, 5.0, 1.5, 1.0);
        draw_sprite(def.sprite, frame, pos - vec2(8.0, 15.0), px > pos.x);
    }
}

fn draw_souls(game: &Game) {
    for (i, s) in game.souls.iter().enumerate() {
        let bob = (TAU * SOUL_BOB_HZ * game.time + i as f32).sin();
        draw_shadow(s.pos, 2.5, 1.0, 0.6);
        draw_sprite(SpriteId::Soul, 0, s.pos - vec2(4.0, 9.0 + bob), false);
    }
}

fn draw_projectiles(game: &Game) {
    for p in &game.projectiles {
        draw_sprite(
            SpriteId::Fireball,
            0,
            p.pos - vec2(4.0, 4.0 + PROJ_DRAW_HEIGHT),
            false,
        );
    }
}

/// Fades are centred horizontally on their position and stand on it.
fn draw_fades(game: &Game) {
    for f in &game.fades {
        let k = 1.0 - f.t / f.duration;
        let (_, src) = sprite(f.sprite, f.frame);
        let pos = f.pos - vec2(src.w * 0.5, src.h - 1.0);
        let flip = game.player.pos.x > f.pos.x;
        draw_sprite_tinted(f.sprite, f.frame, pos, flip, Color::new(1.0, 1.0, 1.0, k));
    }
}

/// Draws Veyra hovering over `pos` (the point on the ground below her).
fn draw_veyra(game: &Game, pos: Vec2, t: f32) {
    let p = &game.player;
    let opacity = game.dying.map_or(1.0, |d| (1.0 - d / DEATH_TIME).max(0.0));
    let blink_off =
        game.dying.is_none() && p.iframes > 0.0 && (p.iframes / BLINK_PERIOD) as u32 % 2 == 1;
    if blink_off {
        return;
    }
    let tint = Color::new(1.0, 1.0, 1.0, opacity);
    let bob = BOB_AMPLITUDE * (TAU * BOB_HZ * t).sin();
    let shrink = 1.0 - SHADOW_SHRINK * bob / BOB_AMPLITUDE;
    draw_shadow(pos, 6.0 * shrink, 2.0 * shrink, opacity);

    let body = pos - vec2(8.0, 24.0 + HOVER_HEIGHT + bob.round());
    let moving = p.vel.length() > IMPULSE_SPEED;
    let facing_left = p.facing < 0.0;
    // The tail hangs on the side opposite to where she faces, so it trails behind.
    let tail_x = if facing_left { 10.0 } else { -2.0 };
    draw_sprite_tinted(
        SpriteId::VeyraWings,
        (t * WING_FPS) as usize,
        body + vec2(-4.0, 6.0),
        false,
        tint,
    );
    draw_sprite_tinted(
        SpriteId::VeyraTail,
        0,
        body + vec2(tail_x, 15.0) + p.tail_lag,
        !facing_left,
        tint,
    );
    draw_sprite_tinted(
        SpriteId::VeyraBody,
        moving as usize,
        body,
        facing_left,
        tint,
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    assets::init();
    let target = render_target(LOGICAL_W as u32, LOGICAL_H as u32);
    target.texture.set_filter(FilterMode::Nearest);

    let mut game = Game::new();
    let mut state = AppState::Title;
    let mut accumulator = 0.0f32;
    let mut selected = 0usize;
    let mut debug = false;
    let mut update_ms = 0.0f32;

    loop {
        let restart = match state {
            AppState::Title => is_key_pressed(KeyCode::Enter),
            AppState::GameOver | AppState::Victory => is_key_pressed(KeyCode::R),
            _ => false,
        };
        if restart {
            game = Game::new();
            accumulator = 0.0;
            state = AppState::Playing;
        } else if matches!(
            state,
            AppState::GameOver | AppState::Paused | AppState::Victory
        ) && is_key_pressed(KeyCode::T)
        {
            game = Game::new();
            state = AppState::Title;
        }

        if is_key_pressed(KeyCode::F3) {
            debug = !debug;
        }
        match state {
            AppState::Playing if is_key_pressed(KeyCode::Escape) => state = AppState::Paused,
            AppState::Paused
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) =>
            {
                state = AppState::Playing;
                accumulator = 0.0;
            }
            AppState::Victory if is_key_pressed(KeyCode::E) => {
                game.start_endless();
                state = AppState::Playing;
                accumulator = 0.0;
            }
            _ => {}
        }

        if state == AppState::Playing {
            let started = get_time();
            let input = read_input();
            accumulator += get_frame_time().min(MAX_FRAME_DT);
            let mut ticks = 0;
            while accumulator >= TICK_DT && ticks < MAX_TICKS_PER_FRAME {
                accumulator -= TICK_DT;
                ticks += 1;
                match game.tick(&input) {
                    SimEvent::None => {}
                    SimEvent::LevelUp => {
                        state = AppState::LevelUp;
                        selected = 0;
                        break;
                    }
                    SimEvent::PlayerDied => {
                        state = AppState::GameOver;
                        accumulator = 0.0;
                        break;
                    }
                    SimEvent::Victory => {
                        state = AppState::Victory;
                        accumulator = 0.0;
                        break;
                    }
                }
            }
            if ticks == MAX_TICKS_PER_FRAME {
                accumulator = 0.0;
            }
            update_ms = ((get_time() - started) * 1000.0) as f32;
        }

        if state == AppState::LevelUp {
            let count = game.choices.iter().flatten().count().max(1);
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Up) {
                selected = (selected + count - 1) % count;
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Down) {
                selected = (selected + 1) % count;
            }
            let keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3];
            let pick = keys
                .iter()
                .position(|&k| is_key_pressed(k))
                .filter(|&slot| slot < count)
                .or(is_key_pressed(KeyCode::Enter).then_some(selected));
            if let Some(slot) = pick {
                game.choose_upgrade(slot);
                selected = 0;
                if game.pending_levels == 0 {
                    state = AppState::Playing;
                }
            }
        }

        let alpha = accumulator / TICK_DT;
        let player_pos = game.player.prev_pos.lerp(game.player.pos, alpha);
        let origin = (player_pos - vec2(LOGICAL_W, LOGICAL_H) * 0.5).round();
        set_camera(&target_camera(&target, origin));
        clear_background(BLACK);
        draw_ground(origin);
        draw_souls(&game);
        draw_enemies(&game);
        draw_fades(&game);
        draw_veyra(&game, player_pos.round(), game.time + alpha * TICK_DT);
        draw_projectiles(&game);

        set_camera(&target_camera(&target, Vec2::ZERO));
        match state {
            AppState::Title => {
                hud::draw_center_screen(GAME_TITLE, &["Survive 5:00", "", "Enter to rise"])
            }
            AppState::Playing => hud::draw_hud(&game),
            AppState::Paused => {
                hud::draw_hud(&game);
                hud::draw_center_screen("Paused", &["Esc resume   T title"]);
            }
            AppState::LevelUp => {
                hud::draw_hud(&game);
                hud::draw_level_up(&game, selected);
            }
            AppState::Victory => {
                let stats = format!("Kills {}   Level {}", game.kills, game.level);
                hud::draw_center_screen(
                    "The village burns",
                    &[&stats, "", "E endless   R restart   T title"],
                );
            }
            AppState::GameOver => {
                let stats = format!(
                    "Survived {}   Kills {}",
                    hud::format_time(game.time),
                    game.kills
                );
                hud::draw_center_screen("Banished", &[&stats, "", "R restart   T title"]);
            }
        }

        if debug {
            hud::draw_debug(&game, get_fps(), update_ms);
        }

        present(&target);
        next_frame().await;
    }
}

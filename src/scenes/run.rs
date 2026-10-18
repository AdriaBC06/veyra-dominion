//! The run scene: fixed-step simulation, world drawing and the in-run overlays
//! (pause menu, level-up choice, game over and victory).

use super::Nav;
use crate::assets::{self, sprite, SpriteId, FLASH_FRAME_OFFSET, GROUND_TILE};
use crate::audio::{self, Sfx};
use crate::config::*;
use crate::game::{Cue, Game, InputState, SimEvent};
use crate::hud;
use crate::i18n::{tr, trf, Key};
use crate::screen::Screen;
use crate::ui::{self, Action, Focus, UiInput};
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// Mouse movement ignores a cursor this close to Veyra (logical pixels).
const MOUSE_DEADZONE: f32 = 6.0;
/// The heartbeat starts at this HP fraction, and quickens below the second.
const LOW_HP: f32 = 0.3;
const CRITICAL_HP: f32 = 0.15;
/// Seconds between heartbeats while low and critical.
const HEARTBEAT_SLOW: f32 = 0.9;
const HEARTBEAT_FAST: f32 = 0.6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Playing,
    Paused,
    LevelUp,
    GameOver,
    Victory,
}

pub struct Run {
    pub game: Game,
    state: State,
    accumulator: f32,
    focus: Focus,
    /// Seconds until the next low-HP heartbeat.
    heartbeat: f32,
    /// Time spent in the simulation last frame, for the debug overlay.
    pub update_ms: f32,
}

const PAUSE_MENU: [Key; 3] = [Key::MenuResume, Key::MenuSettings, Key::MenuTitle];
const GAME_OVER_MENU: [Key; 2] = [Key::MenuRestart, Key::MenuTitle];
const VICTORY_MENU: [Key; 3] = [Key::MenuEndless, Key::MenuRestart, Key::MenuTitle];

impl Run {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            state: State::Playing,
            accumulator: 0.0,
            focus: Focus::default(),
            heartbeat: 0.0,
            update_ms: 0.0,
        }
    }

    /// Buttons and the number of text lines above them for the current menu, if any.
    fn menu(&self) -> Option<(&'static [Key], usize)> {
        match self.state {
            State::Paused => Some((&PAUSE_MENU, 0)),
            State::GameOver => Some((&GAME_OVER_MENU, 1)),
            State::Victory => Some((&VICTORY_MENU, 1)),
            State::Playing | State::LevelUp => None,
        }
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
        self.focus = Focus::default();
        self.accumulator = 0.0;
    }

    pub fn update(&mut self, input: &UiInput, view: Vec2, mouse_move: bool) -> Option<Nav> {
        match self.state {
            State::Playing if input.back => {
                audio::cue(Sfx::Pause);
                self.set_state(State::Paused);
            }
            State::Playing => self.simulate(input, view, mouse_move),
            State::LevelUp => self.level_up(input, view),
            _ => return self.menu_update(input, view),
        }
        None
    }

    fn simulate(&mut self, input: &UiInput, view: Vec2, mouse_move: bool) {
        let started = get_time();
        let mut sim_input = read_input();
        let to_mouse = input.mouse - view * 0.5;
        if mouse_move
            && sim_input.move_dir == Vec2::ZERO
            && input.mouse_down
            && to_mouse.length() > MOUSE_DEADZONE
        {
            sim_input.move_dir = to_mouse;
        }
        self.accumulator += get_frame_time().min(MAX_FRAME_DT);
        let mut ticks = 0;
        while self.accumulator >= TICK_DT && ticks < MAX_TICKS_PER_FRAME {
            self.accumulator -= TICK_DT;
            ticks += 1;
            match self.game.tick(&sim_input) {
                SimEvent::None => {}
                SimEvent::LevelUp => {
                    audio::cue(Sfx::LevelUp);
                    self.state = State::LevelUp;
                    self.focus = Focus::default();
                    break;
                }
                SimEvent::PlayerDied => {
                    self.set_state(State::GameOver);
                    break;
                }
                SimEvent::Victory => {
                    self.set_state(State::Victory);
                    break;
                }
            }
        }
        if ticks == MAX_TICKS_PER_FRAME {
            self.accumulator = 0.0;
        }
        for cue in self.game.cues.drain(..) {
            audio::cue(match cue {
                Cue::Fire => Sfx::Fire,
                Cue::Hit { armored: false } => Sfx::Hit,
                Cue::Hit { armored: true } => Sfx::HitArmored,
                Cue::Kill(Family::Beast) => Sfx::KillBeast,
                Cue::Kill(Family::Human) => Sfx::KillHuman,
                Cue::Kill(Family::Armored) => Sfx::KillArmored,
                Cue::Soul => Sfx::Soul,
                Cue::Hurt => Sfx::Hurt,
                Cue::Died => Sfx::Death,
                Cue::Arrival => Sfx::Horn,
            });
        }
        self.heartbeat();
        self.update_ms = ((get_time() - started) * 1000.0) as f32;
    }

    /// Beats while Veyra is low on HP, faster when critical; the first beat
    /// comes as soon as she drops low.
    fn heartbeat(&mut self) {
        let p = &self.game.player;
        let health = p.hp / p.max_hp;
        if self.game.dying.is_some() || health > LOW_HP {
            self.heartbeat = 0.0;
            return;
        }
        self.heartbeat -= get_frame_time().min(MAX_FRAME_DT);
        if self.heartbeat <= 0.0 {
            audio::cue(Sfx::Heartbeat);
            self.heartbeat = if health > CRITICAL_HP {
                HEARTBEAT_SLOW
            } else {
                HEARTBEAT_FAST
            };
        }
    }

    fn level_up(&mut self, input: &UiInput, view: Vec2) {
        let count = self.game.choices.iter().flatten().count().max(1);
        let cards = hud::level_up_cards(view);
        let action = self.focus.update(input, &cards[..count]);
        let keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3];
        let by_key = keys
            .iter()
            .position(|&k| is_key_pressed(k))
            .filter(|&slot| slot < count);
        let pick = by_key.or(match action {
            Action::Confirm(slot) => Some(slot),
            _ => None,
        });
        if let Some(slot) = pick {
            audio::recue(Sfx::UiConfirm, Sfx::UpgradePick);
            self.game.choose_upgrade(slot);
            self.focus.index = 0;
            if self.game.pending_levels == 0 {
                self.state = State::Playing;
            }
        }
    }

    fn menu_update(&mut self, input: &UiInput, view: Vec2) -> Option<Nav> {
        let (labels, lines) = self.menu()?;
        let rects = ui::button_column(view, hud::center_screen_buttons(lines), labels.len());
        let picked = match self.focus.update(input, &rects) {
            Action::Confirm(i) => labels[i],
            Action::Back if self.state == State::Paused => Key::MenuResume,
            _ => return None,
        };
        match picked {
            Key::MenuResume => self.set_state(State::Playing),
            Key::MenuEndless => {
                self.game.start_endless();
                self.set_state(State::Playing);
            }
            Key::MenuSettings => return Some(Nav::Settings),
            Key::MenuRestart => return Some(Nav::NewRun),
            _ => return Some(Nav::Title),
        }
        None
    }

    /// Draws the world around Veyra with the world camera.
    pub fn draw_world(&self, screen: &Screen) {
        let view = screen.view();
        let game = &self.game;
        let alpha = self.accumulator / TICK_DT;
        let player_pos = game.player.prev_pos.lerp(game.player.pos, alpha);
        let origin = (player_pos - view * 0.5).round();
        set_camera(&screen.camera(origin));
        clear_background(BLACK);
        draw_ground(origin, view);
        draw_souls(game);
        draw_enemies(game);
        draw_fades(game);
        draw_veyra(game, player_pos.round(), game.time + alpha * TICK_DT);
        draw_projectiles(game);
    }

    /// HUD and overlays; call with the UI camera set.
    pub fn draw_ui(&self, view: Vec2) {
        let game = &self.game;
        let stats = match self.state {
            State::Playing => return hud::draw_hud(game, view),
            State::LevelUp => {
                hud::draw_hud(game, view);
                return hud::draw_level_up(game, view, &self.focus);
            }
            State::Paused => {
                hud::draw_hud(game, view);
                hud::draw_center_screen(view, tr(Key::Paused), &[]);
                None
            }
            State::Victory => Some((
                Key::VictoryTitle,
                trf(Key::VictoryStats, &[&game.kills, &game.level]),
            )),
            State::GameOver => Some((
                Key::GameOverTitle,
                trf(
                    Key::GameOverStats,
                    &[&hud::format_time(game.time), &game.kills],
                ),
            )),
        };
        if let Some((title, stats)) = stats {
            hud::draw_center_screen(view, tr(title), &[&stats]);
        }
        if let Some((keys, lines)) = self.menu() {
            let rects = ui::button_column(view, hud::center_screen_buttons(lines), keys.len());
            let labels: Vec<&str> = keys.iter().map(|&k| tr(k)).collect();
            ui::draw_buttons(&rects, &labels, &self.focus);
        }
    }
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

/// Fills the visible area with procedural ground tiles.
fn draw_ground(origin: Vec2, view: Vec2) {
    let tile = GROUND_TILE as f32;
    let (x0, y0) = (
        (origin.x / tile).floor() as i32,
        (origin.y / tile).floor() as i32,
    );
    let (cols, rows) = ((view.x / tile) as i32 + 2, (view.y / tile) as i32 + 2);
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

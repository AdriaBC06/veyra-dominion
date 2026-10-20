//! The run scene: fixed-step simulation, world drawing and the in-run overlays
//! (pause menu, level-up choice, game over and victory).

use super::Nav;
use crate::assets::{
    self, sprite, SpriteId, FLASH_FRAME_OFFSET, GROUND_TILE, VEYRA_FRAME, VEYRA_PIVOT,
};
use crate::audio::{self, Jingle, Sfx};
use crate::blood::BloodLayer;
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
    /// Every pool and splash of blood shed in this run.
    blood: BloodLayer,
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
            blood: BloodLayer::default(),
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
            State::Playing => {
                if is_key_pressed(KeyCode::F4) {
                    self.game.debug_spawn_all();
                }
                if is_key_pressed(KeyCode::F5) {
                    self.game.debug_make_elites();
                }
                if is_key_pressed(KeyCode::F6) {
                    self.game.debug_kill_visible(view);
                }
                if is_key_pressed(KeyCode::F7) {
                    self.game.debug_level_arts();
                }
                self.simulate(input, view, mouse_move);
            }
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
                    audio::jingle(Jingle::GameOver);
                    self.set_state(State::GameOver);
                    break;
                }
                SimEvent::Victory => {
                    audio::jingle(Jingle::Victory);
                    self.set_state(State::Victory);
                    break;
                }
            }
        }
        if ticks == MAX_TICKS_PER_FRAME {
            self.accumulator = 0.0;
        }
        for s in self.game.splats.drain(..) {
            self.blood.add(&s);
        }
        self.blood.sync(self.game.player.pos - view * 0.5, view);
        for cue in self.game.cues.drain(..) {
            audio::cue(match cue {
                Cue::Fire => Sfx::Fire,
                Cue::Lash => Sfx::Lash,
                Cue::Kiss => Sfx::Kiss,
                Cue::Allure => Sfx::Allure,
                Cue::Hellgate => Sfx::Hellgate,
                Cue::Chain => Sfx::Chain,
                Cue::Gust => Sfx::Gust,
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
        self.blood.draw(origin, view);
        draw_gates(game);
        draw_souls(game);
        draw_bursts(game, true);
        draw_enemies(game);
        draw_fades(game);
        draw_bursts(game, false);
        let pos = player_pos.round();
        draw_allure_wave(game, pos);
        draw_charm_marks(game);
        draw_embers(game, pos, false);
        draw_veyra(game, pos, game.time + alpha * TICK_DT);
        draw_embers(game, pos, true);
        draw_lashes(game, pos);
        draw_chains(game);
        draw_bolts(game, alpha);
        draw_projectiles(game);
        draw_feathers(game);
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
        let (_, src) = sprite(def.sprite, frame);
        draw_shadow(pos, src.w * 0.32, (src.w * 0.09).max(1.5), 1.0);
        let corner = pos - vec2((src.w * 0.5).floor(), src.h - 1.0);
        let flip = px > pos.x;
        if e.elite[i] {
            draw_elite_outline(
                def.sprite,
                frame % FLASH_FRAME_OFFSET,
                corner,
                flip,
                game.time,
            );
        }
        let tint = if e.charmed(i, game.time) {
            Color::from_hex(CHARM_TINT)
        } else {
            WHITE
        };
        draw_sprite_tinted(def.sprite, frame, corner, flip, tint);
    }
}

/// A beating heart over every charmed enemy's head.
fn draw_charm_marks(game: &Game) {
    let e = &game.enemies;
    let frame = assets::anim_frame(SpriteId::CharmHeart, game.time, true);
    for i in (0..e.len()).filter(|&i| e.charmed(i, game.time)) {
        let h = ENEMY_DEFS[e.kind[i] as usize].size.1 as f32;
        let bob = (TAU * 2.0 * game.time + e.phase[i]).sin().round();
        let corner = e.pos(i).round() - vec2(3.0, h + CHARM_MARK_HEIGHT + 7.0 + bob);
        draw_sprite(SpriteId::CharmHeart, frame, corner, false);
    }
}

/// Allure's pulse: a ring of hearts that spreads to its edge, then fades there.
fn draw_allure_wave(game: &Game, pos: Vec2) {
    let a = &game.arts.allure;
    let k = a.since_pulse / ALLURE_WAVE_TIME;
    if k >= 2.0 {
        return;
    }
    let spread = 1.0 - (1.0 - k.min(1.0)).powi(3);
    let tint = Color::new(1.0, 1.0, 1.0, (2.0 - k).min(1.0));
    let frame = assets::anim_frame(SpriteId::CharmHeart, a.since_pulse, true);
    for n in 0..ALLURE_WAVE_HEARTS {
        let angle = n as f32 * TAU / ALLURE_WAVE_HEARTS as f32 + a.since_pulse * 0.8;
        let ground = pos + Vec2::from_angle(angle) * a.radius * spread;
        draw_sprite_tinted(
            SpriteId::CharmHeart,
            frame,
            ground.round() - vec2(3.0, 9.0),
            false,
            tint,
        );
    }
}

/// Hellgate pools on the ground, fading in and out.
fn draw_gates(game: &Game) {
    for g in &game.arts.gates {
        let frame = assets::anim_frame(SpriteId::Hellgate, g.t, true);
        let tint = Color::new(1.0, 1.0, 1.0, g.alpha());
        draw_sprite_tinted(
            SpriteId::Hellgate,
            frame,
            g.pos.round() - vec2(18.0, 16.0),
            false,
            tint,
        );
    }
}

/// Chains of Torment: a jagged bolt from her hand through every enemy bitten, that
/// crackles into a new shape every few frames, with a spark on each victim.
fn draw_chains(game: &Game) {
    let [glow, body, core] = CHAIN_COLORS.map(Color::from_hex);
    let spark = assets::anim_frame(SpriteId::ChainSpark, game.time, true);
    for c in &game.arts.chains {
        let fade = ((CHAIN_SHOW_TIME - c.t) / (CHAIN_SHOW_TIME * 0.5)).min(1.0);
        let with = |mut col: Color, a: f32| {
            col.a = a * fade;
            col
        };
        let flicker = (c.t / 0.05) as u32;
        let links = c.links();
        let lift = |k: usize| {
            let h = if k == 0 {
                PROJ_DRAW_HEIGHT
            } else {
                CHAIN_DRAW_HEIGHT
            };
            links[k].round() - vec2(0.0, h)
        };
        for k in 1..links.len() {
            let (a, b) = (lift(k - 1), lift(k));
            let across = (b - a).perp().normalize_or_zero();
            // A kink every `CHAIN_KINK` pixels, the ends pinned.
            let steps = ((a.distance(b) / CHAIN_KINK).ceil() as usize).clamp(2, 32);
            let mut points = [a; 33];
            for (m, p) in points.iter_mut().enumerate().take(steps + 1).skip(1) {
                let jag = if m == steps {
                    0.0
                } else {
                    let seed = c.seed ^ ((k as u32) << 12) ^ ((m as u32) << 4) ^ flicker;
                    (jitter(seed) % 7) as f32 - 3.0
                };
                let t = m as f32 / steps as f32;
                *p = (a.lerp(b, t) + across * jag).round() + vec2(0.5, 0.5);
            }
            let points = &points[..=steps];
            for (width, col) in [(5.0, glow), (3.0, body), (1.0, core)] {
                for w in points.windows(2) {
                    draw_line(w[0].x, w[0].y, w[1].x, w[1].y, width, with(col, 1.0));
                }
            }
            let corner = lift(k) - vec2(4.0, 4.0);
            draw_sprite_tinted(SpriteId::ChainSpark, spark, corner, false, with(WHITE, 1.0));
        }
    }
}

/// A cheap integer hash for the chain's jagged shape.
fn jitter(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

/// Wing Gust feathers, fluttering away and fading.
fn draw_feathers(game: &Game) {
    for f in &game.arts.feathers {
        let frame = assets::anim_frame(SpriteId::GustFeather, f.t, true);
        // Solid for most of the flight; a long fade turns them to brown dots on the grass.
        let fade = ((1.0 - f.t / GUST_FEATHER_LIFE) / 0.3).min(1.0);
        let tint = Color::new(1.0, 1.0, 1.0, fade);
        let corner = f.pos.round() - vec2(5.0, 3.0 + f.height.round());
        draw_sprite_tinted(SpriteId::GustFeather, frame, corner, f.vel.x > 0.0, tint);
    }
}

/// A 1 px ring in `ELITE_OUTLINE` around the sprite's own dark outline: its white
/// silhouette tinted and drawn at the four neighbouring offsets. It breathes gently.
fn draw_elite_outline(id: SpriteId, frame: usize, corner: Vec2, flip: bool, t: f32) {
    let mut color = Color::from_hex(ELITE_OUTLINE);
    color.a = 0.8 + 0.2 * (TAU * 1.5 * t).sin();
    for off in [
        vec2(-1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, -1.0),
        vec2(0.0, 1.0),
    ] {
        draw_sprite_tinted(id, frame + FLASH_FRAME_OFFSET, corner + off, flip, color);
    }
}

/// Death-burst bits, standing on their point: the ones `landed` lie under the horde,
/// the flying ones over it.
fn draw_bursts(game: &Game, landed: bool) {
    for b in game.bursts.iter().filter(|b| b.landed() == landed) {
        let mut color = Color::from_hex(b.color);
        color.a = b.alpha();
        let p = (b.pos - vec2(0.0, b.z)).round() - vec2(1.0, BURST_BIT);
        draw_rectangle(p.x, p.y, BURST_BIT, BURST_BIT, color);
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
    for p in &game.arts.projectiles {
        draw_sprite(
            SpriteId::Fireball,
            0,
            p.pos - vec2(4.0, 4.0 + PROJ_DRAW_HEIGHT),
            false,
        );
    }
}

/// Tail lashes at Veyra's sides, the crescent's tips just above the ground.
fn draw_lashes(game: &Game, pos: Vec2) {
    for l in game.arts.lashes.iter().filter(|l| l.delay <= 0.0) {
        let (_, src) = sprite(SpriteId::TailLash, 0);
        let flip = l.side > 0.0;
        let x = if flip {
            pos.x + 1.0
        } else {
            pos.x - 1.0 - src.w
        };
        let frame = assets::anim_frame(SpriteId::TailLash, l.t, false);
        draw_sprite(
            SpriteId::TailLash,
            frame,
            vec2(x, pos.y - 2.0 - src.h),
            flip,
        );
    }
}

/// Kiss of Ruin hearts, pulsing as they fly.
fn draw_bolts(game: &Game, alpha: f32) {
    let frame = assets::anim_frame(SpriteId::HeartBolt, game.time, true);
    for b in &game.arts.bolts {
        let pos = b.pos + b.vel * (alpha - 1.0) * TICK_DT;
        draw_sprite(
            SpriteId::HeartBolt,
            frame,
            pos - vec2(4.0, 4.0 + BOLT_DRAW_HEIGHT),
            false,
        );
    }
}

/// Brimstone Halo embers circling Veyra: the ones on the far side (`front` false)
/// go behind her, the near ones in front.
fn draw_embers(game: &Game, pos: Vec2, front: bool) {
    let a = game.arts.halo_alpha();
    if a <= 0.0 {
        return;
    }
    let frame = assets::anim_frame(SpriteId::Ember, game.time, true);
    let tint = Color::new(1.0, 1.0, 1.0, a);
    for off in game.arts.embers().filter(|o| (o.y >= 0.0) == front) {
        let ground = (pos + off).round();
        draw_shadow(ground, 2.5, 1.0, 0.6 * a);
        let corner = ground - vec2(3.0, 4.0 + EMBER_DRAW_HEIGHT);
        draw_sprite_tinted(SpriteId::Ember, frame, corner, false, tint);
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

/// Draws Veyra hovering over `pos` (the point on the ground below her): wings, tail,
/// hair and body stacked on one pivot, or the death strip while she burns away.
fn draw_veyra(game: &Game, pos: Vec2, t: f32) {
    let p = &game.player;
    let bob = BOB_AMPLITUDE * (TAU * BOB_HZ * t).sin();
    let pivot = pos
        - vec2(
            0.0,
            HOVER_HEIGHT + VEYRA_FRAME.y - VEYRA_PIVOT.y + bob.round(),
        );
    // She faces left in the art; flipping mirrors the frame around the pivot pixel.
    let flip = p.facing > 0.0;
    let pivot_x = if flip {
        VEYRA_FRAME.x - 1.0 - VEYRA_PIVOT.x
    } else {
        VEYRA_PIVOT.x
    };
    let corner = pivot - vec2(pivot_x, VEYRA_PIVOT.y);

    if let Some(dying) = game.dying {
        // The strip burns away, then its last frame fades out until the game over screen.
        let len = assets::anim_secs(SpriteId::VeyraDeath);
        let fade = ((DEATH_TIME - dying) / (DEATH_TIME - len)).clamp(0.0, 1.0);
        let (w, h) = VEYRA_SHADOW;
        draw_shadow(pos, w, h, fade);
        let frame = assets::anim_frame(SpriteId::VeyraDeath, dying, false);
        let tint = Color::new(1.0, 1.0, 1.0, fade);
        draw_sprite_tinted(SpriteId::VeyraDeath, frame, corner, flip, tint);
        return;
    }

    let shrink = 1.0 - SHADOW_SHRINK * bob / BOB_AMPLITUDE;
    draw_shadow(pos, VEYRA_SHADOW.0 * shrink, VEYRA_SHADOW.1 * shrink, 1.0);

    let since_hurt = PLAYER_IFRAMES - p.iframes;
    let hurt = p.iframes > 0.0 && since_hurt < assets::anim_secs(SpriteId::VeyraHurt);
    // Past the hurt frames she blinks to translucent rather than vanishing, so she
    // never gets lost in the horde.
    let blink = p.iframes > 0.0 && !hurt && (p.iframes / BLINK_PERIOD) as u32 % 2 == 1;
    let tint = Color::new(1.0, 1.0, 1.0, if blink { 0.4 } else { 1.0 });

    let moving = p.vel.length() > IMPULSE_SPEED;
    let (body, body_t, looped) = if hurt {
        (SpriteId::VeyraHurt, since_hurt, false)
    } else if p.since_cast < assets::anim_secs(SpriteId::VeyraCast) {
        (SpriteId::VeyraCast, p.since_cast, false)
    } else if moving {
        (SpriteId::VeyraMove, t, true)
    } else {
        (SpriteId::VeyraIdle, t, true)
    };
    let hair = if moving {
        SpriteId::VeyraHairMove
    } else {
        SpriteId::VeyraHair
    };
    // Tail and hair trail the movement a little, the hair less so it stays on her head.
    let tail_lag = p.tail_lag.clamp_length_max(TAIL_LAG_MAX);
    let hair_lag = (tail_lag * 0.5).clamp_length_max(HAIR_LAG_MAX);
    let layers = [
        (SpriteId::VeyraWings, t, true, Vec2::ZERO),
        (SpriteId::VeyraTail, t, true, tail_lag),
        (hair, t, true, hair_lag),
        (body, body_t, looped, Vec2::ZERO),
    ];
    for (id, secs, looped, offset) in layers {
        let frame = assets::anim_frame(id, secs, looped);
        draw_sprite_tinted(id, frame, corner + offset.round(), flip, tint);
    }
}

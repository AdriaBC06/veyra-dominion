//! The run scene: fixed-step simulation, world drawing and the in-run overlays
//! (pause menu, level-up choice, game over and victory).

use super::Nav;
use crate::assets::{
    self, sprite, SpriteId, FLASH_FRAME_OFFSET, GROUND_TILE, VEYRA_FRAME, VEYRA_PIVOT,
};
use crate::audio::{self, Jingle, Sfx};
use crate::blood::BloodLayer;
use crate::config::*;
use crate::enemies::MODE_WINDUP;
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
/// The Offering screen ignores input this long, so a held key doesn't skip it.
const OFFERING_HOLD: f32 = 0.6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Playing,
    Paused,
    LevelUp,
    Offering,
    GameOver,
    Victory,
    /// The F2 debug panel: pick Arts and Sins, force evolutions.
    Debug,
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
    /// Seconds the Offering screen has been up.
    offering_t: f32,
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
            offering_t: 0.0,
        }
    }

    /// Buttons and the number of text lines above them for the current menu, if any.
    fn menu(&self) -> Option<(&'static [Key], usize)> {
        match self.state {
            State::Paused => Some((&PAUSE_MENU, 0)),
            State::GameOver => Some((&GAME_OVER_MENU, 1)),
            State::Victory => Some((&VICTORY_MENU, 1)),
            State::Playing | State::LevelUp | State::Offering | State::Debug => None,
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
                if is_key_pressed(KeyCode::F8) {
                    self.game.debug_drop_offering();
                }
                if is_key_pressed(KeyCode::F2) {
                    self.set_state(State::Debug);
                    return None;
                }
                self.simulate(input, view, mouse_move);
            }
            State::Debug => self.debug_panel(input, view),
            State::LevelUp => self.level_up(input, view),
            State::Offering => self.offering(input, view),
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
                SimEvent::Offering => {
                    self.set_state(State::Offering);
                    self.offering_t = 0.0;
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
                Cue::Scythe => Sfx::Scythe,
                Cue::Cross => Sfx::Cross,
                Cue::Sigil => Sfx::Sigil,
                Cue::SealBlast => Sfx::SealBlast,
                Cue::Bats => Sfx::Bats,
                Cue::BatBlast => Sfx::BatBlast,
                Cue::BlackMass => Sfx::BlackMass,
                Cue::Gaze => Sfx::Gaze,
                Cue::Thralls => Sfx::Thralls,
                Cue::SirenBurst => Sfx::SirenBurst,
                Cue::Offering => Sfx::Offering,
                Cue::Hit { armored: false } => Sfx::Hit,
                Cue::Hit { armored: true } => Sfx::HitArmored,
                Cue::Kill(Family::Beast) => Sfx::KillBeast,
                Cue::Kill(Family::Human) => Sfx::KillHuman,
                Cue::Kill(Family::Armored) => Sfx::KillArmored,
                Cue::Soul => Sfx::Soul,
                Cue::Hurt => Sfx::Hurt,
                Cue::Died => Sfx::Death,
                Cue::Arrival => Sfx::Horn,
                Cue::Throw => Sfx::Gust,
                Cue::Blast => Sfx::BatBlast,
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

    /// The Offering screen: accept (card 1) or sacrifice (card 2), once `OFFERING_HOLD`
    /// has passed.
    fn offering(&mut self, input: &UiInput, view: Vec2) {
        self.offering_t += get_frame_time().min(MAX_FRAME_DT);
        let cards = hud::offering_cards(view);
        let action = self.focus.update(input, &cards);
        if self.offering_t < OFFERING_HOLD {
            return;
        }
        let keys = [KeyCode::Key1, KeyCode::Key2];
        let pick = keys
            .iter()
            .position(|&k| is_key_pressed(k))
            .or(match action {
                Action::Confirm(slot) => Some(slot),
                _ => None,
            });
        match pick {
            Some(0) => self.game.accept_offering(),
            Some(_) => self.game.sacrifice_offering(),
            None => return,
        }
        audio::cue(Sfx::UiConfirm);
        self.set_state(State::Playing);
    }

    /// F2 panel: up/down a level of the Art or Sin picked, or force an evolution or union.
    fn debug_panel(&mut self, input: &UiInput, view: Vec2) {
        let rects = hud::debug_rects(view);
        let action = self.focus.update(input, &rects);
        if is_key_pressed(KeyCode::F2) || action == Action::Back {
            self.set_state(State::Playing);
            return;
        }
        let right_click = is_mouse_button_pressed(MouseButton::Right);
        let (item, up) = match action {
            Action::Confirm(i) => (i, true),
            Action::Adjust(i, d) => (i, d > 0.0),
            _ => match self.focus.hovered.filter(|_| right_click) {
                Some(i) => (i, false),
                None => return,
            },
        };
        let n = ARTS.len();
        if item < n {
            self.game.debug_art_level(item, up);
        } else if item < 2 * n {
            let art = item - n;
            if up && ARTS[art].evolution.is_some() {
                self.game.debug_evolve(art);
            } else if up {
                self.game.debug_unite(hud::debug_union(art));
            }
        } else {
            self.game.debug_sin_level(item - 2 * n, up);
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
        draw_mass_circle(game, player_pos.round());
        draw_scent(game, player_pos.round());
        draw_souls(game);
        draw_offerings(game);
        draw_fires(game);
        draw_bursts(game, true);
        draw_support_auras(game);
        draw_enemies(game);
        draw_blesses(game);
        draw_thralls(game);
        draw_fades(game);
        draw_bursts(game, false);
        let pos = player_pos.round();
        draw_scent_motes(game, pos);
        draw_allure_wave(game, pos);
        draw_siren_bursts(game);
        draw_charm_marks(game);
        draw_burn_marks(game);
        draw_eye_marks(game);
        draw_bomb_blasts(game);
        draw_enemy_blasts(game);
        draw_embers(game, pos, false);
        draw_bats(game, pos, false);
        draw_veyra(game, pos, game.time + alpha * TICK_DT);
        draw_embers(game, pos, true);
        draw_bats(game, pos, true);
        draw_gazes(game);
        draw_lashes(game, pos);
        draw_chains(game);
        draw_seal_blasts(game);
        draw_sigils(game, alpha);
        draw_crosses(game, alpha);
        draw_scythes(game, alpha);
        draw_bolts(game, alpha);
        draw_projectiles(game, alpha);
        draw_shots(game);
        draw_bombs(game);
        draw_mass_flash(game, origin, view);
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
            State::Debug => {
                hud::draw_hud(game, view);
                hud::draw_debug_panel(game, view, &self.focus);
                None
            }
            State::Offering => {
                hud::draw_hud(game, view);
                let ready = self.offering_t >= OFFERING_HOLD;
                return hud::draw_offering(game, view, ready, &self.focus);
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
        // Frozen ones (Gaze of Desire) hold their step.
        let frozen = e.frozen(i, game.time);
        let walk = if frozen {
            0.0
        } else {
            game.time * ENEMY_ANIM_FPS
        };
        let mut frame = (walk + e.phase[i]) as usize % 2;
        // Telegraphs: a charger winding up blinks white, a primed bomber faster.
        let blink = match (def.behavior, e.mode[i]) {
            (Behavior::Charger { .. }, MODE_WINDUP) => {
                ((game.time * 12.0) as u32).is_multiple_of(2)
            }
            (Behavior::Bomber { .. }, MODE_WINDUP) => ((game.time * 20.0) as u32).is_multiple_of(2),
            _ => false,
        };
        if e.flash[i] > 0.0 || blink {
            frame += FLASH_FRAME_OFFSET;
        }
        let (_, src) = sprite(def.sprite, frame);
        draw_shadow(pos, src.w * 0.32, (src.w * 0.09).max(1.5), 1.0);
        let lift = if matches!(def.behavior, Behavior::Flyer { .. }) {
            FLYER_HEIGHT + (TAU * 1.5 * game.time + e.phase[i] * 3.0).sin().round()
        } else {
            0.0
        };
        let corner = pos - vec2((src.w * 0.5).floor(), src.h - 1.0 + lift);
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
        } else if frozen && e.flash[i] <= 0.0 {
            Color::from_hex(FROZEN_TINT)
        } else if e.weak(i, game.time) && e.flash[i] <= 0.0 {
            Color::from_hex(WEAK_TINT)
        } else {
            WHITE
        };
        draw_sprite_tinted(def.sprite, frame, corner, flip, tint);
    }
}

/// A beating heart over every charmed enemy's head, and over every foe the Infernal
/// Court converted.
fn draw_charm_marks(game: &Game) {
    let e = &game.enemies;
    let frame = assets::anim_frame(SpriteId::CharmHeart, game.time, true);
    let mark = |pos: Vec2, h: f32, phase: f32, alpha: f32| {
        let bob = (TAU * 2.0 * game.time + phase).sin().round();
        let corner = pos.round() - vec2(3.0, h + CHARM_MARK_HEIGHT + 7.0 + bob);
        let tint = Color::new(1.0, 1.0, 1.0, alpha);
        draw_sprite_tinted(SpriteId::CharmHeart, frame, corner, false, tint);
    };
    for i in (0..e.len()).filter(|&i| e.charmed(i, game.time)) {
        let h = ENEMY_DEFS[e.kind[i] as usize].size.1 as f32;
        mark(e.pos(i), h, e.phase[i], 1.0);
    }
    for th in game.arts.thralls.iter().filter(|t| t.kind.is_some()) {
        mark(th.pos, th.def().size.1 as f32, th.phase, th.alpha());
    }
}

/// Thralls: summoned villagers in their own colours, converted foes (Infernal Court)
/// in theirs tinted like the charmed. They face where they go and fade in and out.
fn draw_thralls(game: &Game) {
    for th in &game.arts.thralls {
        let def = th.def();
        let pos = th.pos.round();
        let walk = if th.moving {
            game.time * ENEMY_ANIM_FPS
        } else {
            0.0
        };
        let mut frame = (walk + th.phase) as usize % 2;
        if th.flash > 0.0 {
            frame += FLASH_FRAME_OFFSET;
        }
        let alpha = th.alpha();
        let (_, src) = sprite(def.sprite, frame);
        draw_shadow(pos, src.w * 0.32, (src.w * 0.09).max(1.5), alpha);
        let corner = pos - vec2((src.w * 0.5).floor(), src.h - 1.0);
        let mut tint = if th.kind.is_some() && th.flash <= 0.0 {
            Color::from_hex(CHARM_TINT)
        } else {
            WHITE
        };
        tint.a = alpha;
        draw_sprite_tinted(def.sprite, frame, corner, th.facing > 0.0, tint);
    }
}

/// Evil Eye: a golden eye over every frozen enemy's head.
fn draw_eye_marks(game: &Game) {
    if !game.arts.evolved(GAZE_OF_DESIRE) {
        return;
    }
    let e = &game.enemies;
    let frame = assets::anim_frame(SpriteId::EvilEye, game.time, true);
    for i in (0..e.len()).filter(|&i| e.frozen(i, game.time)) {
        let h = ENEMY_DEFS[e.kind[i] as usize].size.1 as f32;
        let corner = e.pos(i).round() - vec2(4.0, h + CHARM_MARK_HEIGHT + 7.0);
        draw_sprite(SpriteId::EvilEye, frame, corner, false);
    }
}

/// Heretic's Brand: a small flame licks the body of every burning enemy.
fn draw_burn_marks(game: &Game) {
    if !game.arts.evolved(PROFANED_CROSS) {
        return;
    }
    let e = &game.enemies;
    for i in (0..e.len()).filter(|&i| e.burn_until[i] > game.time) {
        let h = ENEMY_DEFS[e.kind[i] as usize].size.1 as f32;
        let frame = assets::anim_frame(SpriteId::GateFlame, game.time + e.phase[i], true);
        let corner = e.pos(i).round() - vec2(3.0, (h * 0.5).round() + 6.0);
        draw_sprite(SpriteId::GateFlame, frame, corner, false);
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

/// Siren's Dominion bursts: a ring of hearts thrown out from where the enemy was.
fn draw_siren_bursts(game: &Game) {
    for b in &game.arts.siren_bursts {
        let k = b.t / SIREN_BURST_TIME;
        let spread = 1.0 - (1.0 - k).powi(2);
        let tint = Color::new(1.0, 1.0, 1.0, (2.0 * (1.0 - k)).min(1.0));
        let frame = assets::anim_frame(SpriteId::CharmHeart, b.t, true);
        for n in 0..SIREN_BURST_HEARTS {
            let angle = n as f32 * TAU / SIREN_BURST_HEARTS as f32;
            let off = Vec2::from_angle(angle) * b.radius * spread;
            // The ground is seen at an angle: the ring is half as deep as it is wide.
            let ground = b.pos + vec2(off.x, off.y * 0.5);
            let corner = ground.round() - vec2(3.0, 9.0);
            draw_sprite_tinted(SpriteId::CharmHeart, frame, corner, false, tint);
        }
    }
}

/// Abyssal Offerings on the ground, hovering and glowing.
fn draw_offerings(game: &Game) {
    let (_, src) = sprite(SpriteId::Offering, 0);
    for (i, o) in game.offerings.iter().enumerate() {
        let bob = (TAU * 0.8 * game.time + i as f32).sin().round();
        draw_shadow(o.pos, 6.0, 2.0, 0.8);
        let frame = assets::anim_frame(SpriteId::Offering, game.time + i as f32 * 0.3, true);
        let corner = o.pos.round() - vec2((src.w * 0.5).floor(), src.h + 2.0 + bob);
        draw_sprite(SpriteId::Offering, frame, corner, false);
    }
}

/// Hellgate pools: a patch of scorched ground as big as the Art's area, in 2 px cells
/// with a ragged, fading edge and embers glowing in the char, plus small flames
/// flickering on their own beats; all fade in and out.
fn draw_gates(game: &Game) {
    let rx = HELLGATE_RADIUS * game.arts.stats[HELLGATE].area;
    let ry = rx * 0.5;
    let [char_, scorch, edge, ember] = GATE_COLORS.map(Color::from_hex);
    let flames =
        ((TAU * 0.5 * rx * ry / GATE_FLAME_AREA) as usize).clamp(GATE_MIN_FLAMES, GATE_MAX_FLAMES);
    let (cx, cy) = (
        (rx / GATE_CELL).ceil() as i32,
        (ry / GATE_CELL).ceil() as i32,
    );
    for g in &game.arts.gates {
        let a = g.alpha();
        let p = g.pos.round();
        for iy in -cy..=cy {
            for ix in -cx..=cx {
                let off = vec2(ix as f32, iy as f32) * GATE_CELL;
                let d = ((off.x / rx).powi(2) + (off.y / ry).powi(2)).sqrt();
                let h = jitter(
                    g.seed
                        ^ (ix as u32).wrapping_mul(0x9e37_79b9)
                        ^ (iy as u32).wrapping_mul(0x85eb_ca6b),
                );
                // A ragged edge: each cell reaches its own distance.
                let reach = 0.7 + 0.35 * (h & 0xff) as f32 / 255.0;
                if d > reach {
                    continue;
                }
                let k = d / reach;
                let (mut c, alpha) = if k < 0.55 {
                    (char_, 0.8)
                } else if k < 0.85 {
                    (scorch, 0.6)
                } else {
                    (edge, 0.3)
                };
                c.a = alpha * a;
                // A few cells of the char glow like embers, breathing.
                if k < 0.7 && (h >> 8).is_multiple_of(GATE_EMBER_ODDS) {
                    let beat = 0.5 + 0.5 * (game.time * 5.0 + (h >> 16) as f32).sin();
                    c = ember;
                    c.a = (0.45 + 0.45 * beat) * a;
                }
                let q = p + off;
                draw_rectangle(q.x, q.y, GATE_CELL, GATE_CELL, c);
            }
        }
        // Flames on random spots of the patch, the far ones first.
        let mut spots = [Vec2::ZERO; GATE_MAX_FLAMES];
        for (n, spot) in spots.iter_mut().enumerate().take(flames) {
            let h = jitter(g.seed ^ (n as u32).wrapping_mul(0x9e37_79b9));
            let angle = (h & 0xffff) as f32 / 65536.0 * TAU;
            let dist = (((h >> 16) & 0xff) as f32 / 256.0).sqrt() * 0.75;
            *spot = p + vec2(angle.cos() * rx, angle.sin() * ry) * dist;
        }
        let spots = &mut spots[..flames];
        spots.sort_by(|a, b| a.y.total_cmp(&b.y));
        for (n, spot) in spots.iter().enumerate() {
            let phase = (jitter(g.seed.wrapping_add(n as u32)) % 400) as f32 / 1000.0;
            let frame = assets::anim_frame(SpriteId::GateFlame, g.t + phase, true);
            let corner = spot.round() - vec2(3.0, 9.0);
            let tint = Color::new(1.0, 1.0, 1.0, a);
            draw_sprite_tinted(SpriteId::GateFlame, frame, corner, false, tint);
        }
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

/// A faint ring around each Curate: the reach of his blessing.
fn draw_support_auras(game: &Game) {
    let e = &game.enemies;
    let color = Color::from_hex(SUPPORT_AURA).with_alpha(0.35);
    for i in 0..e.len() {
        if let Behavior::Support { radius, .. } = ENEMY_DEFS[e.kind[i] as usize].behavior {
            let p = e.pos(i).round();
            draw_ellipse_lines(p.x, p.y, radius, radius * 0.5, 0.0, 1.0, color);
        }
    }
}

/// A small pale cross over every enemy a Curate just healed.
fn draw_blesses(game: &Game) {
    let e = &game.enemies;
    for i in (0..e.len()).filter(|&i| e.bless_until[i] > game.time) {
        let h = ENEMY_DEFS[e.kind[i] as usize].size.1 as f32;
        let age = BLESS_SHOW - (e.bless_until[i] - game.time);
        let frame = assets::anim_frame(SpriteId::Bless, age, true);
        let corner = (e.pos(i) - vec2(2.0, h + 9.0 + age * 8.0)).round();
        draw_sprite(SpriteId::Bless, frame, corner, false);
    }
}

/// Burning patches left by torches.
fn draw_fires(game: &Game) {
    for f in &game.fires {
        let fade = 1.0 - (f.t / FIRE_LIFE).powi(3);
        let frame = assets::anim_frame(SpriteId::FirePatch, f.t, true);
        let corner = f.pos.round() - vec2(13.0, 8.0);
        let tint = Color::new(1.0, 1.0, 1.0, fade);
        draw_sprite_tinted(SpriteId::FirePatch, frame, corner, false, tint);
    }
}

/// Stones and lobbed torches, with their shadow on the ground.
fn draw_shots(game: &Game) {
    for s in &game.shots {
        draw_shadow(s.pos.round(), 2.0, 1.0, 0.8);
        let p = (s.pos - vec2(0.0, s.height())).round();
        match s.kind {
            ShotKind::Stone => draw_sprite(SpriteId::Stone, 0, p - vec2(2.0, 2.0), false),
            ShotKind::Torch => {
                let frame = assets::anim_frame(SpriteId::TorchSpin, s.t, true);
                draw_sprite(SpriteId::TorchSpin, frame, p - vec2(4.0, 4.0), false);
            }
        }
    }
}

/// Zealots' explosions, centred a little above the ground where they stood.
fn draw_enemy_blasts(game: &Game) {
    for b in &game.blasts {
        let frame = assets::anim_frame(SpriteId::ZealotBlast, b.t, false);
        draw_sprite(
            SpriteId::ZealotBlast,
            frame,
            b.pos.round() - vec2(22.0, 20.0),
            false,
        );
    }
}

/// Hellflare flames, flying head first and interpolated along their flight.
fn draw_projectiles(game: &Game, alpha: f32) {
    for p in &game.arts.projectiles {
        let pos = p.pos + p.vel * (alpha - 1.0) * TICK_DT;
        draw_flying(SpriteId::Fireball, pos, p.vel, PROJ_DRAW_HEIGHT, game.time);
    }
}

/// A strip baked at 9 headings (straight up to straight down on the right, mirrored
/// for the left) times 4 flickers, centred `height` above `pos` and turned along `vel`.
fn draw_flying(id: SpriteId, pos: Vec2, vel: Vec2, height: f32, time: f32) {
    const HEADINGS: usize = 9;
    const FLICKERS: usize = 4;
    let angle = vel.y.atan2(vel.x.abs());
    let heading = ((angle / std::f32::consts::PI + 0.5) * (HEADINGS - 1) as f32).round() as usize;
    let flicker = assets::anim_frame(id, time, true) % FLICKERS;
    let frame = heading.min(HEADINGS - 1) * FLICKERS + flicker;
    let (_, src) = sprite(id, frame);
    let corner = pos - vec2((src.w * 0.5).floor(), (src.h * 0.5).floor() + height);
    draw_sprite(id, frame, corner, vel.x < 0.0);
}

/// Tail lashes at Veyra's sides, centred on her hitbox. A longer reach (area)
/// stretches the swing outwards.
fn draw_lashes(game: &Game, pos: Vec2) {
    let area = game.arts.stats[TAIL_LASH].area;
    for l in game.arts.lashes.iter().filter(|l| l.delay <= 0.0) {
        let frame = assets::anim_frame(SpriteId::TailLash, l.t, false);
        let (tex, src) = sprite(SpriteId::TailLash, frame);
        let w = (src.w + LASH_REACH * (area - 1.0)).round();
        let flip = l.side > 0.0;
        let x = if flip { pos.x + 1.0 } else { pos.x - 1.0 - w };
        draw_texture_ex(
            &tex,
            x,
            pos.y - (src.h * 0.5).round(),
            WHITE,
            DrawTextureParams {
                source: Some(src),
                dest_size: Some(vec2(w, src.h)),
                flip_x: flip,
                ..Default::default()
            },
        );
    }
}

/// Scent of Sin: a pink haze around her with a clear rim, and a ring that pulses out
/// from it every time it burns.
fn draw_scent(game: &Game, pos: Vec2) {
    if !game.arts.owns(SCENT_OF_SIN) {
        return;
    }
    let radius = SCENT_RADIUS * game.arts.stats[SCENT_OF_SIN].area;
    let color = |a: f32| {
        let mut c = Color::from_hex(SCENT_COLOR);
        c.a = a;
        c
    };
    // The haze breathes a little; on each burn it flares and a ring runs outwards.
    let since = game.arts.since_scent;
    let flare = (1.0 - since / 0.3).max(0.0);
    let breath = 0.5 + 0.5 * (TAU * 0.6 * game.time).sin();
    draw_circle(
        pos.x,
        pos.y,
        radius,
        color(0.2 + 0.04 * breath + 0.1 * flare),
    );
    draw_circle(pos.x, pos.y, radius * 0.6, color(0.06));
    draw_circle_lines(pos.x, pos.y, radius, 2.0, color(0.85 + 0.15 * flare));
    if since < 0.4 {
        let k = since / 0.4;
        draw_circle_lines(
            pos.x,
            pos.y,
            radius * (1.0 + 0.25 * k),
            2.0,
            color(0.8 * (1.0 - k)),
        );
    }
}

/// Motes of the Scent drifting up through the haze and fading as they rise.
fn draw_scent_motes(game: &Game, pos: Vec2) {
    if !game.arts.owns(SCENT_OF_SIN) {
        return;
    }
    let radius = SCENT_RADIUS * game.arts.stats[SCENT_OF_SIN].area;
    let mut color = Color::from_hex(SCENT_COLOR);
    for n in 0..SCENT_MOTES as u32 {
        let h = jitter(n.wrapping_mul(0x9e37_79b9));
        let angle = (h & 0xffff) as f32 / 65536.0 * TAU;
        let dist = ((h >> 16) & 0xff) as f32 / 256.0;
        // Each mote rises for 1.5 s, then starts again at its spot on the ground.
        let life = (game.time / 1.5 + (h >> 24) as f32 / 256.0).fract();
        let ground = pos + Vec2::from_angle(angle) * radius * dist.sqrt();
        color.a = 0.8 * (1.0 - life);
        let p = (ground - vec2(1.0, 2.0 + life * 20.0)).round();
        draw_rectangle(p.x, p.y, 2.0, 2.0, color);
    }
}

/// A spinning sprite (its strip holds one turn) centred `height` above `pos`,
/// interpolated back along `vel`. `turns` counts the turns made so far.
fn draw_spinning(
    id: SpriteId,
    pos: Vec2,
    vel: Vec2,
    alpha: f32,
    height: f32,
    turns: f32,
    tint: Color,
) {
    let frames = assets::frame_ms(id).len();
    let frame = ((turns.rem_euclid(1.0) * frames as f32) as usize).min(frames - 1);
    let (_, src) = sprite(id, frame);
    let pos = pos + vel * (alpha - 1.0) * TICK_DT;
    let corner = pos - vec2((src.w * 0.5).floor(), (src.h * 0.5).floor() + height);
    // Mirrored, the strip turns the other way: scythes spin the way they fly.
    draw_sprite_tinted(id, frame, corner, vel.x < 0.0, tint);
}

/// Reaper's Scythes, spinning as they fly, as big as their area.
fn draw_scythes(game: &Game, alpha: f32) {
    let id = assets::SCYTHE_TIERS[assets::size_tier(game.arts.stats[REAPERS_SCYTHE].area)];
    for (k, sc) in game.arts.scythes.iter().enumerate() {
        let turns = game.time * SCYTHE_SPIN + k as f32 * 0.37;
        draw_spinning(id, sc.pos, sc.vel, alpha, SCYTHE_DRAW_HEIGHT, turns, WHITE);
    }
}

/// Profaned Crosses, burning ones once evolved (Heretic's Brand), as big as their area.
fn draw_crosses(game: &Game, alpha: f32) {
    let tiers = if game.arts.evolved(PROFANED_CROSS) {
        assets::CROSS_BURNING_TIERS
    } else {
        assets::CROSS_TIERS
    };
    let id = tiers[assets::size_tier(game.arts.stats[PROFANED_CROSS].area)];
    for (k, c) in game.arts.crosses.iter().enumerate() {
        let turns = game.time * CROSS_SPIN + k as f32 * 0.37;
        draw_spinning(id, c.pos, c.vel, alpha, CROSS_DRAW_HEIGHT, turns, WHITE);
    }
}

/// Wandering Sigils, their runes turning; they fade at the end of their wander.
/// Broken Seal's are cracked and burning, throbbing before they burst.
fn draw_sigils(game: &Game, alpha: f32) {
    let tiers = if game.arts.evolved(WANDERING_SIGIL) {
        assets::SIGIL_BROKEN_TIERS
    } else {
        assets::SIGIL_TIERS
    };
    let id = tiers[assets::size_tier(game.arts.stats[WANDERING_SIGIL].area)];
    for (k, g) in game.arts.sigils.iter().enumerate() {
        let tint = Color::new(1.0, 1.0, 1.0, (g.life / SIGIL_FADE).min(1.0));
        let turns = game.time * SIGIL_SPIN + k as f32 * 0.37;
        // Interpolated along its real flight; the runes keep turning one way, whichever
        // way it flies, so it is drawn with a rightward velocity and no extra step.
        let at = g.pos + g.vel * (alpha - 1.0) * TICK_DT;
        let vel = vec2(g.vel.x.abs(), g.vel.y);
        draw_spinning(id, at, vel, 1.0, SIGIL_DRAW_HEIGHT, turns, tint);
    }
}

/// Broken Seal blasts: the burst where the sigil was, and a shockwave spreading over
/// the ground to the blast's true radius, gold cooling to crimson, sparks riding it.
fn draw_seal_blasts(game: &Game) {
    let hot = Color::from_hex(0xffe08a);
    let cool = Color::from_hex(0xa8182e);
    let pink = Color::from_hex(0xff6f9a);
    for b in &game.arts.seal_blasts {
        let k = (b.t / SEAL_BLAST_TIME).min(1.0);
        let fade = 1.0 - k;
        let r = b.radius * (1.0 - (1.0 - k).powi(3));
        let at = b.pos.round();
        let mut glow = hot;
        glow.a = 0.18 * fade * fade;
        draw_circle(at.x, at.y, r, glow);
        let mut inner = pink;
        inner.a = 0.6 * fade;
        draw_circle_lines(at.x, at.y, r * 0.7, 1.0, inner);
        let mut wave = Color::new(
            hot.r + (cool.r - hot.r) * k,
            hot.g + (cool.g - hot.g) * k,
            hot.b + (cool.b - hot.b) * k,
            fade.sqrt(),
        );
        draw_circle_lines(at.x, at.y, r, if k < 0.5 { 2.0 } else { 1.0 }, wave);
        for n in 0..SEAL_BLAST_SPARKS {
            let angle = n as f32 * TAU / SEAL_BLAST_SPARKS as f32 + 0.2;
            let p = (at + Vec2::from_angle(angle) * (r + 2.0)).round();
            wave = if n % 2 == 0 { hot } else { pink };
            wave.a = fade;
            draw_rectangle(p.x - 1.0, p.y - 1.0, 2.0, 2.0, wave);
        }
        let frame = assets::anim_frame(SpriteId::SealBurst, b.t, false);
        let burst = at - vec2(16.0, 16.0 + SIGIL_DRAW_HEIGHT);
        draw_sprite(SpriteId::SealBurst, frame, burst, false);
    }
}

/// Bat Swarm: the bats circling her, the far ones (`front` false) behind her, each
/// flapping and bobbing on its own beat. Crimson Host bats are blood red.
fn draw_bats(game: &Game, pos: Vec2, front: bool) {
    let id = if game.arts.evolved(BAT_SWARM) {
        SpriteId::BatHost
    } else {
        SpriteId::Bat
    };
    for (k, off) in game.arts.bats().enumerate() {
        if (off.y >= 0.0) != front {
            continue;
        }
        let beat = k as f32 * 0.29;
        let ground = (pos + off).round();
        let bob = (BAT_BOB * (TAU * 1.3 * game.time + beat * TAU).sin()).round();
        draw_shadow(ground + vec2(0.0, VEYRA_DROP), 4.0, 1.0, 0.4);
        let frame = assets::anim_frame(id, game.time + beat, true);
        let corner = ground - vec2(8.0, 5.0 + BAT_DRAW_HEIGHT + bob);
        draw_sprite(id, frame, corner, false);
    }
}

/// Blood bombs falling from the bats onto their mark, in a short arc.
fn draw_bombs(game: &Game) {
    let frame = assets::anim_frame(SpriteId::BloodBomb, game.time, true);
    for b in game.arts.bombs.iter().filter(|b| !b.landed()) {
        let k = b.t / BAT_DROP_TIME;
        let ground = b.from.lerp(b.to, k);
        let height = BAT_DRAW_HEIGHT * (1.0 - k) + 24.0 * k * (1.0 - k);
        draw_shadow(ground.round(), 2.0, 1.0, 0.5 * k);
        draw_sprite(
            SpriteId::BloodBomb,
            frame,
            (ground - vec2(3.0, 3.0 + height)).round(),
            false,
        );
    }
}

/// Bomb bursts: a dark red splash and a ring of drops spreading on the ground.
fn draw_bomb_blasts(game: &Game) {
    let colors = [Color::from_hex(0xe04454), Color::from_hex(0x5c0c1c)];
    for b in game.arts.bombs.iter().filter(|b| b.landed()) {
        let k = ((b.t - BAT_DROP_TIME) / BAT_BLAST_TIME).min(1.0);
        let spread = 1.0 - (1.0 - k).powi(2);
        let fade = 1.0 - k;
        let r = b.radius * spread;
        let mut splash = Color::from_hex(0xa8182e);
        splash.a = 0.45 * fade;
        draw_ellipse(b.to.x, b.to.y, r, r * 0.5, 0.0, splash);
        for n in 0..BAT_BLAST_SPARKS {
            let angle = n as f32 * TAU / BAT_BLAST_SPARKS as f32 + b.radius;
            let off = Vec2::from_angle(angle) * r;
            // The drops fly up a little before they land.
            let lift = 10.0 * k * (1.0 - k) * 4.0;
            let p = (b.to + vec2(off.x, off.y * 0.5 - lift)).round();
            let mut c = colors[n % 2];
            c.a = fade.min(1.0);
            draw_rectangle(p.x - 1.0, p.y - 1.0, 2.0, 2.0, c);
        }
    }
}

/// Black Mass: the ritual circle under her, flaring up and fading out. Witches'
/// Sabbath draws it in gold.
fn draw_mass_circle(game: &Game, pos: Vec2) {
    let t = game.arts.since_mass;
    if t >= MASS_SHOW {
        return;
    }
    let id = if game.arts.evolved(BLACK_MASS) {
        SpriteId::Sabbath
    } else {
        SpriteId::BlackMass
    };
    let a = (t / 0.08).min(1.0) * (1.0 - t / MASS_SHOW).powf(0.7);
    let frame = assets::anim_frame(id, t, true);
    let (_, src) = sprite(id, frame);
    let corner = pos + vec2(0.0, VEYRA_DROP) - vec2(src.w * 0.5, src.h * 0.5).floor();
    draw_sprite_tinted(id, frame, corner, false, Color::new(1.0, 1.0, 1.0, a));
}

/// Black Mass: everything in view flashes red as the rite strikes.
fn draw_mass_flash(game: &Game, origin: Vec2, view: Vec2) {
    let t = game.arts.since_mass;
    if t >= MASS_FLASH {
        return;
    }
    let mut c = Color::from_hex(MASS_FLASH_COLOR);
    c.a = MASS_FLASH_ALPHA * (1.0 - t / MASS_FLASH).powi(2);
    draw_rectangle(origin.x, origin.y, view.x, view.y, c);
}

/// Gaze of Desire: golden beams from her eyes, shooting out and fading.
fn draw_gazes(game: &Game) {
    let [glow, body, core] = GAZE_COLORS.map(Color::from_hex);
    for g in &game.arts.gazes {
        let k = g.t / GAZE_SHOW;
        let fade = (1.0 - k).min(1.0);
        let reach = (g.t / 0.06).min(1.0);
        let a = (g.from - vec2(0.0, GAZE_DRAW_HEIGHT)).round();
        let b = (g.from + g.dir * g.len * reach - vec2(0.0, CHAIN_DRAW_HEIGHT)).round();
        for (width, mut col) in [(5.0, glow), (3.0, body), (1.0, core)] {
            col.a = fade * if width > 4.0 { 0.6 } else { 1.0 };
            draw_line(a.x, a.y, b.x, b.y, width, col);
        }
    }
}

/// Kiss of Ruin hearts of pink fire, flying head first.
fn draw_bolts(game: &Game, alpha: f32) {
    for b in &game.arts.bolts {
        let pos = b.pos + b.vel * (alpha - 1.0) * TICK_DT;
        draw_flying(SpriteId::HeartBolt, pos, b.vel, BOLT_DRAW_HEIGHT, game.time);
    }
}

/// Brimstone Halo fireballs circling Veyra: the ones on the far side (`front` false)
/// go behind her, the near ones in front.
fn draw_embers(game: &Game, pos: Vec2, front: bool) {
    let a = game.arts.halo_alpha();
    if a <= 0.0 {
        return;
    }
    let frame = assets::anim_frame(SpriteId::Ember, game.time, true);
    let tint = Color::new(1.0, 1.0, 1.0, a);
    for off in game.arts.embers().filter(|o| (o.y >= 0.0) == front) {
        let at = (pos + off).round();
        draw_shadow(at + vec2(0.0, VEYRA_DROP), 5.0, 1.5, 0.6 * a);
        // The ball (centred at 7, 11 in its frame) sits on its point of the orbit,
        // which circles her middle; the flames of its crown rise above.
        let corner = at - vec2(7.0, 11.0);
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
    // Her body sits on her hitbox; `pos` becomes the ground under her from here on.
    let pos = pos + vec2(0.0, VEYRA_DROP);
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

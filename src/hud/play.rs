//! The in-run HUD (design.md 7.14): the level medallion and the soul bar, health with a
//! lagging damage bar, the timer, kills, Art/Sin slots with cooldown sweeps and the boss
//! bar, all in stone mouldings. `HudFx` keeps the motion (counting numbers, flashes, shakes
//! and sparks) in real seconds; the run only hands it the game each frame.

use super::{draw_icon_alpha, format_time, Anchor, Icon};
use crate::assets::{draw_ui_text, measure_ui_text, TextStyle, ICON_SIZE};
use crate::config::*;
use crate::game::Game;
use crate::i18n::tr;
use crate::kit::{self, Rim};
use crate::motion;
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// Margin from the view's edges.
const M: f32 = 6.0;
/// The level medallion, as tall as the soul and health bars with the gap between them.
const MEDAL: f32 = 38.0;
/// Width of a bar's stone moulding.
const BAR_THICK: f32 = 4.0;
const SOUL_H: f32 = 16.0;
const HP_W: f32 = 132.0;
const HP_H: f32 = 18.0;
const BARS_X: f32 = M + MEDAL + 4.0;
const HP_Y: f32 = M + SOUL_H + 4.0;
/// Art and Sin slots: an icon in a rim, with level pips under it.
const SLOT: f32 = ICON_SIZE + 6.0;
const SLOT_STEP: f32 = SLOT + 3.0;
const ARTS_Y: f32 = M + MEDAL + 6.0;
const SINS_Y: f32 = ARTS_Y + SLOT + 8.0;
const PIP: f32 = 2.0;
/// Arts this quick don't sweep: they would flicker all the time.
const SWEEP_FROM: f32 = 0.9;
/// Arts at least this slow flash when they are ready again.
const READY_FLASH_FROM: f32 = 1.5;
const BOSS_W: f32 = 320.0;
const BOSS_H: f32 = 16.0;

/// How long each effect lasts (seconds).
const HIT_TIME: f32 = 0.3;
const LAG_HOLD: f32 = 0.4;
const LEVEL_TIME: f32 = 1.2;
const SOUL_TIME: f32 = 0.35;
const POP_TIME: f32 = 0.6;
const READY_TIME: f32 = 0.3;
const MINUTE_TIME: f32 = 0.8;
const BOSS_INTRO: f32 = 1.2;
/// Kills pop every this many.
const KILL_MILESTONE: u32 = 100;
/// Below this health the edges of the view pulse with the heartbeat.
const VIGNETTE_HP: f32 = crate::scenes::LOW_HP;

const SPARKS: usize = 48;

/// Fills: deep (bottom), body and highlight (top) of a bar.
struct Ramp {
    deep: Color,
    body: Color,
    hi: Color,
}

const SOUL_RAMP: Ramp = Ramp {
    deep: Color::new(0.06, 0.08, 0.24, 1.0),
    body: Color::new(0.16, 0.3, 0.62, 1.0),
    hi: Color::new(0.68, 0.95, 1.0, 1.0),
};
/// Blood: dark venous red with a wet highlight.
const HP_RAMP: Ramp = Ramp {
    deep: Color::new(0.24, 0.0, 0.03, 1.0),
    body: Color::new(0.52, 0.02, 0.06, 1.0),
    hi: Color::new(0.9, 0.26, 0.24, 1.0),
};
/// Red cells carried along the blood: a light rim around a dark dimple.
const CELL_RIM: Color = Color::new(0.92, 0.2, 0.18, 1.0);
const CELL_CORE: Color = Color::new(0.3, 0.0, 0.03, 1.0);
/// Heartbeats per second, calm and at low health.
const PULSE_RATE: f32 = 1.15;
const PULSE_RATE_LOW: f32 = 2.0;
/// The boss bar wears the Watch's royal blue.
const BOSS_RAMP: Ramp = Ramp {
    deep: Color::new(0.1, 0.13, 0.3, 1.0),
    body: Color::new(0.27, 0.38, 0.63, 1.0),
    hi: Color::new(0.56, 0.69, 0.96, 1.0),
};
/// What a hit took, before it drains.
const LAG: Color = Color::new(0.96, 0.62, 0.5, 1.0);
const BONE: Color = Color::new(0.86, 0.8, 0.72, 1.0);

#[derive(Clone, Copy, Default)]
enum SparkKind {
    #[default]
    Soul,
    Gold,
    Ember,
}

#[derive(Clone, Copy, Default)]
struct Spark {
    pos: Vec2,
    vel: Vec2,
    age: f32,
    life: f32,
    kind: SparkKind,
}

/// Motion state of the HUD; everything in real seconds.
pub struct HudFx {
    primed: bool,
    t: f32,
    hp: f32,
    /// The damage bar: what health was before the hit, drained after `LAG_HOLD`.
    lag: f32,
    since_hit: f32,
    since_heal: f32,
    since_beat: f32,
    xp: u32,
    /// The soul bar's drawn fill (0..1) gliding to the real one.
    soul: f32,
    since_soul: f32,
    level: u32,
    since_level: f32,
    kills: u32,
    kills_shown: f32,
    since_milestone: f32,
    minute: u32,
    since_minute: f32,
    /// Per Art slot: the icon shown (to pop when it changes), its charge and flash clocks.
    slot_icon: [Option<usize>; MAX_ARTS],
    since_slot: [f32; MAX_ARTS],
    charge: [f32; MAX_ARTS],
    since_ready: [f32; MAX_ARTS],
    sin_level: [u8; SINS.len()],
    since_sin: [f32; SINS.len()],
    boss: bool,
    since_boss: f32,
    boss_hp: f32,
    boss_lag: f32,
    since_boss_hit: f32,
    /// The blood's heartbeat (0..1 per beat) and how far it has flowed, in pixels.
    pulse: f32,
    flow: f32,
    sparks: [Spark; SPARKS],
    next_spark: usize,
    rng: u32,
}

impl Default for HudFx {
    fn default() -> Self {
        Self {
            primed: false,
            t: 0.0,
            hp: 0.0,
            lag: 0.0,
            since_hit: f32::INFINITY,
            since_heal: f32::INFINITY,
            since_beat: f32::INFINITY,
            xp: 0,
            soul: 0.0,
            since_soul: f32::INFINITY,
            level: 0,
            since_level: f32::INFINITY,
            kills: 0,
            kills_shown: 0.0,
            since_milestone: f32::INFINITY,
            minute: 0,
            since_minute: f32::INFINITY,
            slot_icon: [None; MAX_ARTS],
            since_slot: [f32::INFINITY; MAX_ARTS],
            charge: [1.0; MAX_ARTS],
            since_ready: [f32::INFINITY; MAX_ARTS],
            sin_level: [0; SINS.len()],
            since_sin: [f32::INFINITY; SINS.len()],
            boss: false,
            since_boss: f32::INFINITY,
            boss_hp: 0.0,
            boss_lag: 0.0,
            since_boss_hit: f32::INFINITY,
            pulse: 0.0,
            flow: 0.0,
            sparks: [Spark::default(); SPARKS],
            next_spark: 0,
            rng: 0x9e37_79b9,
        }
    }
}

/// `v` moved toward `target` at `rate` per second (exponential, frame-rate independent).
fn approach(v: f32, target: f32, rate: f32, dt: f32) -> f32 {
    v + (target - v) * (1.0 - (-rate * dt).exp())
}

fn mix(a: Color, b: Color, k: f32) -> Color {
    let k = k.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * k,
        a.g + (b.g - a.g) * k,
        a.b + (b.b - a.b) * k,
        a.a + (b.a - a.a) * k,
    )
}

/// 1 at the start of an effect `len` long, 0 once it is over.
fn fresh(since: f32, len: f32) -> f32 {
    (1.0 - since / len).clamp(0.0, 1.0)
}

fn soul_rect(view: Vec2) -> Rect {
    Rect::new(BARS_X, M, view.x - BARS_X - M, SOUL_H)
}

fn inner(r: Rect) -> Rect {
    Rect::new(
        r.x + BAR_THICK,
        r.y + BAR_THICK,
        r.w - 2.0 * BAR_THICK,
        r.h - 2.0 * BAR_THICK,
    )
}

fn slot_rect(n: usize, y: f32) -> Rect {
    Rect::new(M + n as f32 * SLOT_STEP, y, SLOT, SLOT)
}

impl HudFx {
    fn rand(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }

    fn spark(&mut self, pos: Vec2, vel: Vec2, life: f32, kind: SparkKind) {
        self.sparks[self.next_spark] = Spark {
            pos,
            vel,
            age: 0.0,
            life,
            kind,
        };
        self.next_spark = (self.next_spark + 1) % SPARKS;
    }

    /// `n` sparks bursting out of `rect`'s edge.
    fn burst(&mut self, rect: Rect, n: usize, kind: SparkKind) {
        for _ in 0..n {
            let a = self.rand() * TAU;
            let dir = vec2(a.cos(), a.sin());
            let pos = rect.center() + dir * vec2(rect.w, rect.h) * 0.5;
            let speed = 20.0 + self.rand() * 40.0;
            let life = 0.4 + self.rand() * 0.4;
            self.spark(pos, dir * speed, life, kind);
        }
    }

    /// A heartbeat sounded: the low-health vignette throbs with it.
    pub fn beat(&mut self) {
        self.since_beat = 0.0;
    }

    /// Follows the game by `dt` real seconds: what changed starts its effect.
    pub fn update(&mut self, game: &Game, view: Vec2, dt: f32) {
        let p = &game.player;
        let minute = (game.time / 60.0) as u32;
        if !self.primed {
            self.primed = true;
            self.hp = p.hp;
            self.lag = p.hp;
            self.xp = game.xp;
            self.level = game.level;
            self.soul = game.xp as f32 / xp_to_next(game.level) as f32;
            self.kills = game.kills;
            self.kills_shown = game.kills as f32;
            self.minute = minute;
            self.sin_level = game.upgrades.sins;
        }
        self.t += dt;
        for s in [
            &mut self.since_hit,
            &mut self.since_heal,
            &mut self.since_beat,
            &mut self.since_soul,
            &mut self.since_level,
            &mut self.since_milestone,
            &mut self.since_minute,
            &mut self.since_boss,
            &mut self.since_boss_hit,
        ] {
            *s += dt;
        }
        for s in self.since_slot.iter_mut().chain(&mut self.since_ready) {
            *s += dt;
        }
        for s in &mut self.since_sin {
            *s += dt;
        }

        // Health: a hit shakes the bar and leaves the damage bar behind for a moment.
        if p.hp < self.hp - 0.01 {
            if self.since_hit > LAG_HOLD {
                self.lag = self.lag.max(self.hp);
            }
            self.since_hit = 0.0;
        } else if p.hp > self.hp + 0.01 {
            self.since_heal = 0.0;
        }
        self.hp = p.hp;
        if self.since_hit > LAG_HOLD {
            self.lag = (self.lag - p.max_hp * 0.8 * dt).max(p.hp);
        }
        self.lag = self.lag.max(p.hp);
        // The heart pumps: the blood surges on each beat and coasts between them.
        let rate = if p.hp <= p.max_hp * VIGNETTE_HP {
            PULSE_RATE_LOW
        } else {
            PULSE_RATE
        };
        self.pulse = (self.pulse + rate * dt).fract();
        self.flow += (5.0 + 26.0 * pump(self.pulse)) * dt;

        // Souls: the bar glides to its fill, a level-up flashes and empties it.
        let medal = Rect::new(M, M, MEDAL, MEDAL);
        let target = game.xp as f32 / xp_to_next(game.level) as f32;
        if game.level > self.level {
            self.since_level = 0.0;
            self.soul = 0.0;
            self.burst(medal, 14, SparkKind::Gold);
        } else if game.xp > self.xp {
            self.since_soul = 0.0;
            let bar = inner(soul_rect(view));
            let tip = vec2(bar.x + bar.w * target, bar.y + bar.h * 0.5);
            for _ in 0..2 {
                let vel = vec2(self.rand() * 16.0 - 8.0, -14.0 - self.rand() * 14.0);
                let life = 0.7 + self.rand() * 0.5;
                self.spark(tip, vel, life, SparkKind::Soul);
            }
        }
        self.level = game.level;
        self.xp = game.xp;
        self.soul = approach(self.soul, target, 10.0, dt).min(target);

        // Kills count up; every hundred pops.
        if game.kills / KILL_MILESTONE > self.kills / KILL_MILESTONE {
            self.since_milestone = 0.0;
            let w = measure_ui_text(&game.kills.to_string(), TextStyle::Name).width;
            let r = Rect::new(view.x - M - w, M + SOUL_H + 6.0, w, 18.0);
            self.burst(r, 10, SparkKind::Ember);
        }
        self.kills = game.kills;
        self.kills_shown = approach(self.kills_shown, game.kills as f32, 12.0, dt);
        if (game.kills as f32 - self.kills_shown).abs() < 0.5 {
            self.kills_shown = game.kills as f32;
        }
        if minute > self.minute {
            self.since_minute = 0.0;
        }
        self.minute = minute;

        // Art slots: a new or changed icon pops in, a slow Art flashes when ready again.
        let u = &game.upgrades;
        for (n, art) in u.slots.iter().enumerate() {
            let icon = art.map(|i| Icon::of_art(u, i).frame());
            if icon != self.slot_icon[n] && icon.is_some() {
                self.since_slot[n] = 0.0;
                self.burst(slot_rect(n, ARTS_Y), 8, SparkKind::Gold);
            }
            self.slot_icon[n] = icon;
            let Some(i) = *art else { continue };
            let c = game.arts.charge(i);
            if c >= 1.0 && self.charge[n] < 1.0 && game.arts.stats[i].cooldown >= READY_FLASH_FROM
            {
                self.since_ready[n] = 0.0;
            }
            self.charge[n] = c;
        }
        let mut row = 0;
        for i in 0..SINS.len() {
            let lv = u.sins[i];
            if lv > self.sin_level[i] {
                self.since_sin[i] = 0.0;
                self.burst(slot_rect(row, SINS_Y), 8, SparkKind::Gold);
            }
            self.sin_level[i] = lv;
            row += (lv > 0) as usize;
        }

        // The boss bar fills in when he arrives and keeps a damage bar too.
        match &game.boss {
            Some(b) => {
                let h = b.health();
                if !self.boss {
                    self.boss = true;
                    self.since_boss = 0.0;
                    self.boss_hp = h;
                    self.boss_lag = h;
                }
                if h < self.boss_hp - 0.0005 {
                    self.since_boss_hit = 0.0;
                }
                self.boss_hp = h;
                if self.since_boss_hit > LAG_HOLD {
                    self.boss_lag = approach(self.boss_lag, h, 4.0, dt);
                }
                self.boss_lag = self.boss_lag.max(h);
            }
            None => self.boss = false,
        }

        for s in &mut self.sparks {
            if s.age < s.life {
                s.age += dt;
                s.pos += s.vel * dt;
                s.vel *= (-3.0 * dt).exp();
                // Souls float away; everything else falls.
                s.vel.y += match s.kind {
                    SparkKind::Soul => -12.0,
                    _ => 30.0,
                } * dt;
            }
        }
    }
}

/// A bar's fill over `r` up to `frac`: the ramp in three bands, with a highlight row.
fn fill(r: Rect, frac: f32, ramp: &Ramp, bright: f32, alpha: f32) {
    let w = (r.w * frac.clamp(0.0, 1.0)).round();
    if w <= 0.0 {
        return;
    }
    let lift = |c: Color| kit::fade(mix(c, WHITE, bright), alpha);
    draw_rectangle(r.x, r.y, w, r.h, lift(ramp.body));
    let band = (r.h * 0.3).round().max(1.0);
    draw_rectangle(r.x, r.y + r.h - band, w, band, lift(ramp.deep));
    draw_rectangle(r.x, r.y, w, 1.0, lift(ramp.hi));
    draw_rectangle(r.x, r.y + 1.0, w, band - 1.0, lift(mix(ramp.body, ramp.hi, 0.45)));
}

/// Light rippling along a fill (souls drifting inside the bar).
fn ripples(r: Rect, frac: f32, ramp: &Ramp, t: f32, alpha: f32) {
    let w = (r.w * frac.clamp(0.0, 1.0)).round();
    let mut x = 0.0;
    while x < w {
        let a = ((x * 0.19 - t * 3.2).sin() + (x * 0.071 + t * 1.3).sin()) * 0.5;
        if a > 0.25 {
            let k = (a - 0.25) / 0.75;
            draw_rectangle(r.x + x, r.y, 2.0, r.h - 1.0, kit::fade(ramp.hi, k * 0.4 * alpha));
            let y = r.y + 1.0 + ((x * 0.37 + t * 5.0).sin() * 0.5 + 0.5) * (r.h - 3.0);
            draw_rectangle(r.x + x, y.round(), 2.0, 1.0, kit::fade(WHITE, k * 0.55 * alpha));
        }
        x += 2.0;
    }
}

/// 0..1 noise for item `i` of a decorative set, stable from frame to frame.
fn hash(i: u32, salt: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9e37_79b9) ^ salt.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= x >> 12;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// The heart's push at `phase` (0..1 of a beat): a sharp surge, then a smaller echo.
fn pump(phase: f32) -> f32 {
    let surge = (1.0 - phase / 0.22).max(0.0);
    let echo = (1.0 - (phase - 0.36).abs() / 0.09).max(0.0);
    surge * surge + 0.35 * echo
}

/// Inside the soul bar: spectral mist and wisps (a bright head, a wavering tail) drifting
/// toward the tip, where they melt into it.
fn souls(r: Rect, frac: f32, t: f32) {
    let w = (r.w * frac.clamp(0.0, 1.0)).round();
    let mut x = 0.0;
    while x < w {
        let k = ((x * 0.09 + t * 0.7).sin() + (x * 0.23 - t * 1.1 + 1.3).sin()) * 0.5;
        if k > 0.0 {
            let h = (r.h * (0.4 + 0.5 * k)).round();
            let y = r.y + ((r.h - h) * (0.5 + 0.4 * (x * 0.05 + t).sin())).round();
            draw_rectangle(r.x + x, y, 2.0, h, kit::fade(SOUL_RAMP.hi, k * 0.22));
        }
        x += 2.0;
    }
    let count = (r.w / 22.0) as u32;
    let wave = |s: f32, phase: f32| r.y + r.h * 0.5 - 1.0 + (s * 2.3 + phase).sin() * (r.h * 0.28);
    for i in 0..count {
        let speed = 7.0 + 12.0 * hash(i, 1);
        let phase = hash(i, 2) * TAU;
        let head = (hash(i, 3) * r.w + t * speed) % r.w;
        if head + 2.0 > w {
            continue;
        }
        // Fade in from the left and out into the tip.
        let a = (head / 8.0).min((w - head - 2.0) / 10.0).clamp(0.0, 1.0)
            * (0.75 + 0.25 * (t * 4.0 + phase).sin());
        // The tail: thick near the head, thinning as it wavers behind.
        for j in 1..=10 {
            let tx = head - j as f32;
            if tx < 0.0 {
                break;
            }
            let ty = wave(t - j as f32 * 0.07, phase).round();
            let fade = 1.0 - j as f32 / 11.0;
            let h = if j <= 4 { 2.0 } else { 1.0 };
            let y = if j <= 4 { ty } else { ty + 0.5 };
            draw_rectangle(r.x + tx, y.round(), 1.0, h, kit::fade(SOUL_RAMP.hi, a * fade * 0.75));
        }
        let hy = wave(t, phase).round();
        let (hx, glow) = (r.x + head, kit::fade(SOUL_RAMP.hi, a * 0.35));
        draw_rectangle(hx - 1.0, hy, 4.0, 2.0, glow);
        draw_rectangle(hx, hy - 1.0, 2.0, 4.0, glow);
        draw_rectangle(hx, hy, 2.0, 2.0, kit::fade(SOUL_RAMP.hi, a));
        draw_rectangle(hx + 1.0, hy, 1.0, 1.0, kit::fade(WHITE, a));
    }
}

/// Health as blood in circulation: each heartbeat sends a pressure wave down the bar, red
/// cells stream along it (faster mid-vessel than at the walls), a wet sheen slides on top
/// and the edge laps like a liquid.
fn blood(r: Rect, frac: f32, pulse: f32, flow: f32, t: f32) {
    let w = (r.w * frac.clamp(0.0, 1.0)).round();
    if w <= 0.0 {
        return;
    }
    // Dark eddies carried by the flow, and the pressure wave brightening what it passes.
    let mut x = 0.0;
    while x < w {
        let u = x - flow;
        let e = ((u * 0.17).sin() + (u * 0.061 + 1.7).sin()) * 0.5;
        if e > 0.2 {
            let y = r.y + 2.0 + (((u * 0.09).sin() * 0.5 + 0.5) * (r.h - 5.0)).round();
            draw_rectangle(r.x + x, y, 2.0, 2.0, kit::fade(HP_RAMP.deep, (e - 0.2) * 0.9));
        }
        let k = pump((pulse - x / (r.w * 2.5)).rem_euclid(1.0));
        if k > 0.05 {
            draw_rectangle(r.x + x, r.y + 1.0, 2.0, r.h - 2.0, kit::fade(CELL_RIM, k * 0.4));
        }
        x += 2.0;
    }
    let count = (r.w / 6.0) as u32;
    for i in 0..count {
        let lane = hash(i, 4);
        let y = r.y + 1.0 + (lane * (r.h - 4.0)).round();
        let wall = (lane * 2.0 - 1.0).abs();
        let speed = (1.0 - 0.6 * wall * wall) * (0.85 + 0.3 * hash(i, 5));
        let cx = ((hash(i, 6) * r.w + flow * speed) % r.w).round();
        if cx + 3.0 > w {
            continue;
        }
        let a = if hash(i, 7) < 0.35 { 0.5 } else { 0.95 };
        draw_rectangle(r.x + cx, y + 1.0, 3.0, 1.0, kit::fade(CELL_RIM, a));
        draw_rectangle(r.x + cx + 1.0, y, 1.0, 3.0, kit::fade(CELL_RIM, a));
        draw_rectangle(r.x + cx + 1.0, y + 1.0, 1.0, 1.0, kit::fade(CELL_CORE, a));
    }
    // A wet sheen in streaks drifting with the flow.
    let mut x = 0.0;
    while x < w {
        let k = ((x - flow * 0.8) * 0.13).sin();
        if k > 0.55 {
            let a = (k - 0.55) / 0.45 * 0.45;
            draw_rectangle(r.x + x, r.y + 1.0, 1.0, 1.0, kit::fade(WHITE, a));
        }
        x += 1.0;
    }
    // The edge laps: each row reaches a little further or shorter, swelling on the beat.
    if w < r.w {
        let band = (r.h * 0.3).round().max(1.0);
        let surge = pump(pulse);
        for row in 0..r.h as u32 {
            let y = row as f32;
            let reach = (1.0 + (y * 0.8 - t * 5.0).sin() + surge).round().max(0.0);
            let reach = reach.min(r.w - w);
            if reach <= 0.0 {
                continue;
            }
            let c = if y >= r.h - band {
                HP_RAMP.deep
            } else if row == 0 {
                HP_RAMP.hi
            } else {
                HP_RAMP.body
            };
            draw_rectangle(r.x + w, r.y + y, reach, 1.0, c);
            draw_rectangle(r.x + w + reach - 1.0, r.y + y, 1.0, 1.0, kit::fade(HP_RAMP.hi, 0.4));
        }
    }
}

/// The bright edge at the end of a fill, flaring with `flare`.
fn tip(r: Rect, frac: f32, ramp: &Ramp, flare: f32, t: f32) {
    if frac <= 0.0 || frac >= 1.0 {
        return;
    }
    let x = (r.x + r.w * frac).round();
    let flicker = 0.75 + 0.25 * (t * 23.0).sin();
    let glow = 2.0 + 4.0 * flare;
    draw_rectangle(x - glow, r.y, glow, r.h, kit::fade(ramp.hi, 0.25 * flicker));
    draw_rectangle(x - 2.0, r.y, 2.0, r.h, kit::fade(ramp.hi, 0.85 * flicker));
    draw_rectangle(x - 1.0, r.y, 1.0, r.h, kit::fade(WHITE, 0.6 * flicker + 0.4 * flare));
}

/// The whole in-run HUD.
pub fn draw(game: &Game, fx: &HudFx, view: Vec2) {
    vignette(game, fx, view);
    medal(game, fx);
    soul_bar(fx, view);
    hp_bar(game, fx);
    timer(game, fx, view);
    kills(fx, view);
    slots(game, fx);
    boss_bar(game, fx, view);
    for s in &fx.sparks {
        if s.age >= s.life {
            continue;
        }
        let k = 1.0 - s.age / s.life;
        let c = match s.kind {
            SparkKind::Soul => mix(SOUL_RAMP.hi, WHITE, k * 0.5),
            SparkKind::Gold => mix(kit::pal().gold, kit::pal().gold_lit, k),
            SparkKind::Ember => mix(HP_RAMP.body, kit::pal().gold, k),
        };
        let size = if k > 0.5 { 2.0 } else { 1.0 };
        let mut pos = s.pos;
        if let SparkKind::Soul = s.kind {
            // A wisp: it sways as it rises and trails a fading tail.
            pos.x += (s.age * 9.0 + s.life * 40.0).sin() * 1.5;
            let tail = vec2(pos.x.round(), pos.y.round() + size);
            draw_rectangle(tail.x, tail.y, 1.0, 2.0, kit::fade(c, k * 0.5));
        }
        draw_rectangle(pos.x.round(), pos.y.round(), size, size, kit::fade(c, k));
    }
}

/// Low health: the edges of the view darken red and throb with the heartbeat; any hit
/// flashes them.
fn vignette(game: &Game, fx: &HudFx, view: Vec2) {
    let p = &game.player;
    let low = (1.0 - p.hp / p.max_hp / VIGNETTE_HP).clamp(0.0, 1.0);
    let beat = fresh(fx.since_beat, 0.45);
    let hit = fresh(fx.since_hit, HIT_TIME);
    let a = low * (0.3 + 0.35 * beat) + hit * 0.25;
    if a <= 0.0 {
        return;
    }
    const BANDS: usize = 12;
    for i in 0..BANDS {
        let k = 1.0 - i as f32 / BANDS as f32;
        let c = Color::new(0.45, 0.0, 0.04, a * k * k);
        let d = i as f32 * 3.0;
        let (w, h) = (view.x - 2.0 * d, view.y - 2.0 * d);
        draw_rectangle(d, d, w, 3.0, c);
        draw_rectangle(d, d + h - 3.0, w, 3.0, c);
        draw_rectangle(d, d + 3.0, 3.0, h - 6.0, c);
        draw_rectangle(d + w - 3.0, d + 3.0, 3.0, h - 6.0, c);
    }
}

/// The level in a stone medallion that blazes on a level-up.
fn medal(game: &Game, fx: &HudFx) {
    let r = Rect::new(M, M, MEDAL, MEDAL);
    let up = fresh(fx.since_level, LEVEL_TIME);
    kit::well(r, up > 0.0, 1.0, 0x6c6576);
    let s = game.level.to_string();
    let style = if measure_ui_text(&s, TextStyle::Name).width <= MEDAL - 12.0 {
        TextStyle::Name
    } else {
        TextStyle::Desc
    };
    let p = kit::pal();
    let pop = motion::ease_out_back(1.0 - up).min(1.0);
    let lift = ((1.0 - pop) * 3.0).round();
    let color = mix(p.gold, WHITE, up * 0.8);
    let y = kit::ink_baseline(&s, style, r) - lift;
    kit::text_centered(&s, r.center().x, y, style, color);
    if up > 0.0 {
        // a ring of light opening out of the medallion
        let grow = (1.0 - up) * 10.0;
        let ring = Rect::new(r.x - grow, r.y - grow, r.w + 2.0 * grow, r.h + 2.0 * grow);
        draw_rectangle_lines(ring.x, ring.y, ring.w, ring.h, 1.0, kit::fade(p.gold_lit, up));
    }
}

/// Souls toward the next level: rippling soul-light with a flaring tip; a level-up washes
/// it white before it starts again.
fn soul_bar(fx: &HudFx, view: Vec2) {
    let r = soul_rect(view);
    kit::moulding(r, BAR_THICK as u16, false, 1.0, 0x5017);
    let i = inner(r);
    let up = fresh(fx.since_level, 0.4);
    fill(i, fx.soul, &SOUL_RAMP, 0.0, 1.0);
    souls(i, fx.soul, fx.t);
    tip(i, fx.soul, &SOUL_RAMP, fresh(fx.since_soul, SOUL_TIME), fx.t);
    if up > 0.0 {
        draw_rectangle(i.x, i.y, i.w, i.h, kit::fade(SOUL_RAMP.hi, up * up * 0.85));
    }
}

/// Health: the bar shakes on a hit, the damage bar drains after it, a heal glows, and low
/// health throbs with the heartbeat. The number counts on the bar.
fn hp_bar(game: &Game, fx: &HudFx) {
    let p = &game.player;
    let hit = fresh(fx.since_hit, HIT_TIME);
    let shake = vec2((fx.t * 91.0).sin(), (fx.t * 73.0).cos() * 0.5) * 2.0 * hit;
    let r = Rect::new(BARS_X + shake.x.round(), HP_Y + shake.y.round(), HP_W, HP_H);
    let health = p.hp / p.max_hp;
    let low = health <= VIGNETTE_HP;
    kit::moulding(r, BAR_THICK as u16, low, 1.0, 0x4e47);
    let i = inner(r);
    let lag = fx.lag / p.max_hp;
    if lag > health {
        let a = 0.5 + 0.3 * hit;
        fill(i, lag, &HP_RAMP, 0.0, 0.0);
        let x = (i.x + i.w * health).round();
        let w = (i.x + i.w * lag).round() - x;
        draw_rectangle(x, i.y, w, i.h, kit::fade(LAG, a));
    }
    let beat = if low { fresh(fx.since_beat, 0.45) } else { 0.0 };
    let bright = hit * 0.3 + beat * 0.25 + fresh(fx.since_heal, 0.4) * 0.3;
    fill(i, health, &HP_RAMP, bright, 1.0);
    blood(i, health, fx.pulse, fx.flow, fx.t);
    let s = format!("{} / {}", p.hp.max(0.0).ceil(), p.max_hp.round());
    let color = mix(BONE, Color::new(1.0, 0.6, 0.55, 1.0), hit);
    let y = kit::ink_baseline(&s, TextStyle::Desc, r);
    outlined(&s, r.center().x, y, color);
}

/// `s` centred on `cx` in Display with a dark outline, readable on any fill.
fn outlined(s: &str, cx: f32, y: f32, color: Color) {
    let x = (cx - measure_ui_text(s, TextStyle::Desc).width * 0.5).round();
    let sh = kit::fade(kit::pal().shadow, 0.85);
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (1.0, 1.0)] {
        draw_ui_text(s, x + dx, y + dy, TextStyle::Desc, sh);
    }
    draw_ui_text(s, x, y, TextStyle::Desc, color);
}

/// The run's clock, centred under the soul bar; each new minute flares it.
fn timer(game: &Game, fx: &HudFx, view: Vec2) {
    let s = format_time(game.time);
    let k = fresh(fx.since_minute, MINUTE_TIME);
    let p = kit::pal();
    let color = mix(p.gold, p.gold_lit, 0.3 + 0.7 * k);
    let r = Rect::new(0.0, M + SOUL_H + 2.0, view.x, 22.0);
    let y = kit::ink_baseline(&s, TextStyle::Name, r) - (k * 2.0).round();
    kit::text_centered(&s, view.x * 0.5, y, TextStyle::Name, color);
}

/// A skull, `SKULL[row]` bit 8 on the left.
const SKULL: [u16; 9] = [
    0b001111100,
    0b011111110,
    0b111111111,
    0b110010011,
    0b110010011,
    0b111101111,
    0b011111110,
    0b001010100,
    0b001010100,
];

fn skull(x: f32, y: f32, color: Color) {
    for (row, bits) in SKULL.iter().enumerate() {
        for col in 0..9 {
            if bits >> (8 - col) & 1 == 1 {
                let (px, py) = (x + col as f32, y + row as f32);
                draw_rectangle(px, py + 1.0, 1.0, 1.0, kit::pal().shadow);
                draw_rectangle(px, py, 1.0, 1.0, color);
            }
        }
    }
}

/// Kills on the right, counting up, popping every hundred.
fn kills(fx: &HudFx, view: Vec2) {
    let s = (fx.kills_shown.round() as u32).to_string();
    let k = fresh(fx.since_milestone, POP_TIME);
    let p = kit::pal();
    let w = measure_ui_text(&s, TextStyle::Name).width;
    let x = Anchor::Right.x(view, w, M);
    let r = Rect::new(x, M + SOUL_H + 2.0, w, 22.0);
    let y = kit::ink_baseline(&s, TextStyle::Name, r) - (k * 3.0).round();
    kit::text(&s, x, y, TextStyle::Name, mix(BONE, p.gold_lit, k));
    skull(x - 13.0, (r.center().y - 4.0).round(), mix(BONE, p.gold, k));
}

/// `filled` of `total` level pips centred under a slot.
fn slot_pips(slot: Rect, filled: u32, total: u32, color: Color) {
    let w = total as f32 * (PIP + 1.0) - 1.0;
    let x0 = (slot.center().x - w * 0.5).round();
    let y = slot.bottom() + 2.0;
    for n in 0..total {
        let x = x0 + n as f32 * (PIP + 1.0);
        let c = if n < filled {
            color
        } else {
            Color::new(0.1, 0.07, 0.08, 0.9)
        };
        draw_rectangle(x, y + 1.0, PIP, PIP, kit::pal().shadow);
        draw_rectangle(x, y, PIP, PIP, c);
    }
}

/// Owned Arts (with their cooldown sweeps) and Sins in rimmed slots, each with its level.
fn slots(game: &Game, fx: &HudFx) {
    let u = &game.upgrades;
    let p = kit::pal();
    for (n, &i) in u.slots.iter().flatten().enumerate() {
        let r = slot_rect(n, ARTS_Y);
        let pop = fresh(fx.since_slot[n], POP_TIME);
        let icon = Icon::of_art(u, i);
        let rim = match icon {
            _ if pop > 0.0 => Rim::Lit,
            Icon::Evolution(_) => Rim::Evolved,
            Icon::Union(_) => Rim::Union,
            _ => Rim::Plain,
        };
        kit::slot(r, rim, 1.0);
        let (ix, iy) = (r.x + 3.0, r.y + 3.0 - (pop * 2.0).round());
        draw_icon_alpha(icon, ix, iy, 1.0, 1.0);
        let c = fx.charge[n];
        if c < 1.0 && game.arts.stats[i].cooldown >= SWEEP_FROM {
            let h = (ICON_SIZE * (1.0 - c)).round();
            draw_rectangle(ix, iy, ICON_SIZE, h, Color::new(0.03, 0.01, 0.03, 0.62));
            draw_rectangle(ix, iy + h, ICON_SIZE, 1.0, kit::fade(p.gold, 0.7));
        }
        let ready = fresh(fx.since_ready[n], READY_TIME);
        if ready > 0.0 {
            draw_rectangle(ix, iy, ICON_SIZE, ICON_SIZE, kit::fade(WHITE, ready * 0.55));
        }
        if pop > 0.0 {
            draw_rectangle(ix, iy, ICON_SIZE, ICON_SIZE, kit::fade(p.gold_lit, pop * 0.5));
        }
        slot_pips(r, u.arts[i].into(), ART_MAX_LEVEL.into(), p.gold);
    }
    let owned = (0..SINS.len()).filter(|&i| u.sins[i] > 0);
    for (n, i) in owned.enumerate() {
        let r = slot_rect(n, SINS_Y);
        let pop = fresh(fx.since_sin[i], POP_TIME);
        kit::slot(r, if pop > 0.0 { Rim::Lit } else { Rim::Plain }, 1.0);
        let iy = r.y + 3.0 - (pop * 2.0).round();
        draw_icon_alpha(Icon::Sin(i), r.x + 3.0, iy, 1.0, 1.0);
        if pop > 0.0 {
            draw_rectangle(r.x + 3.0, iy, ICON_SIZE, ICON_SIZE, kit::fade(p.gold_lit, pop * 0.5));
        }
        slot_pips(r, u.sins[i].into(), SINS[i].max_level.into(), p.ruby);
    }
}

/// While a boss lives: his name over a stone-framed bar at the bottom, filling in as he
/// arrives, with a damage bar and gems where his later phases begin (lit once reached).
fn boss_bar(game: &Game, fx: &HudFx, view: Vec2) {
    let Some(b) = &game.boss else {
        return;
    };
    let intro = motion::ease_out_cubic((fx.since_boss / BOSS_INTRO).min(1.0));
    let w = BOSS_W.min(view.x - 60.0);
    let r = Rect::new(Anchor::Center.x(view, w, 0.0), view.y - M - BOSS_H, w, BOSS_H);
    let p = kit::pal();
    let name = tr(b.def.name);
    let ny = r.y - 4.0 + ((1.0 - intro) * 6.0).round();
    kit::text_centered(name, r.center().x, ny, TextStyle::Name, kit::fade(p.gold, intro));
    kit::moulding(r, BAR_THICK as u16, false, intro, 0xb055);
    let i = inner(r);
    let health = fx.boss_hp.min(intro);
    let lag = fx.boss_lag.min(intro);
    if lag > health {
        fill(i, lag, &BOSS_RAMP, 0.0, 0.0);
        let x = (i.x + i.w * health).round();
        let lw = (i.x + i.w * lag).round() - x;
        draw_rectangle(x, i.y, lw, i.h, kit::fade(LAG, 0.85));
    }
    let hit = fresh(fx.since_boss_hit, 0.12);
    fill(i, health, &BOSS_RAMP, hit * 0.5, 1.0);
    ripples(i, health, &BOSS_RAMP, fx.t * 0.8, 0.6);
    tip(i, health, &BOSS_RAMP, 0.0, fx.t);
    for phase in &b.def.phases[1..] {
        let x = (i.x + i.w * phase.below).round();
        let reached = b.health() <= phase.below;
        let c = if reached { p.ruby } else { Color::new(0.2, 0.16, 0.18, 1.0) };
        let y = r.y + 1.0;
        draw_rectangle(x, i.y, 1.0, i.h, kit::fade(p.shadow, intro));
        // a small diamond set into the top moulding
        draw_rectangle(x - 1.0, y - 2.0, 3.0, 5.0, kit::fade(p.shadow, intro));
        draw_rectangle(x - 2.0, y - 1.0, 5.0, 3.0, kit::fade(p.shadow, intro));
        draw_rectangle(x, y - 1.0, 1.0, 3.0, kit::fade(c, intro));
        draw_rectangle(x - 1.0, y, 3.0, 1.0, kit::fade(c, intro));
    }
}

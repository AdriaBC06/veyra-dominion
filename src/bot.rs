//! The autopilot (F12 in a run, and the balance probe): kites the horde, gathers souls
//! and picks upgrades for a focused build. Simulation side: reads `Game`, returns input.

use crate::boss::BossMode;
use crate::config::*;
use crate::game::Game;
use crate::upgrades::Choice;
use macroquad::math::Vec2;
use std::f32::consts::TAU;

/// Bot input: flee the nearby horde, circle around it, drift toward souls and Offerings.
pub fn steer(g: &Game) -> Vec2 {
    let p = g.player.pos;
    let e = &g.enemies;
    let mut away = Vec2::ZERO;
    for i in 0..e.len() {
        let d = p - e.pos(i);
        let l2 = d.length_squared().max(16.0);
        if l2 < 140.0 * 140.0 {
            away += d / l2;
        }
    }
    for s in &g.shots {
        let d = p - s.pos;
        let l2 = d.length_squared().max(16.0);
        if l2 < 60.0 * 60.0 {
            away += d / l2 * 3.0;
        }
    }
    for f in &g.fires {
        let d = p - f.pos;
        let l2 = d.length_squared().max(16.0);
        if l2 < 30.0 * 30.0 {
            away += d / l2 * 3.0;
        }
    }
    away += dodge_boss(g) * 3.0;
    // Close threats dominate; with room to breathe she goes for the souls.
    let urgency = (away.length() * 35.0).clamp(0.0, 1.0);
    let away = away.normalize_or_zero();
    let tangent = away.perp();
    // Souls along the way out count as nearer, so she gathers while she kites.
    let cost = |q: Vec2| {
        let d = q - p;
        d.length() * (2.0 - d.normalize_or_zero().dot(away) * urgency)
    };
    let goal = g
        .offerings
        .first()
        .map(|o| o.pos)
        .or_else(|| {
            g.souls
                .iter()
                .map(|s| s.pos)
                .min_by(|&a, &b| cost(a).total_cmp(&cost(b)))
        })
        .map_or(Vec2::ZERO, |q| (q - p).normalize_or_zero());
    let want = (away * urgency + tangent * 0.5 * urgency + goal * (1.2 - urgency * 0.6))
        .normalize_or_zero();
    around_walls(g, want)
}

/// How far the autopilot looks for walls along each heading, how much it favours a free run
/// ahead and open ground at the end of it (so the horde can't pin her against a wall), and
/// how much enemies along a heading count against it.
const BOT_LOOK: f32 = 96.0;
const BOT_ROOM: f32 = 0.4;
const BOT_OPEN: f32 = 0.8;
const BOT_CROWD: f32 = 0.25;
/// Enemies this close to a heading's path (out to `BOT_LOOK`) crowd it.
const BOT_LANE: f32 = 20.0;
const HEADINGS: usize = 16;

/// The free heading closest to `want`: 16 rays out to `BOT_LOOK`, each scored by how well
/// it agrees with `want`, how far it runs free, how open the ground is where it ends and
/// how few enemies stand along it; a wall right ahead rules it out.
fn around_walls(g: &Game, want: Vec2) -> Vec2 {
    if g.map.is_open() || want == Vec2::ZERO {
        return want;
    }
    let p = g.player.pos;
    let dirs: [Vec2; HEADINGS] =
        std::array::from_fn(|k| Vec2::from_angle(k as f32 * TAU / HEADINGS as f32));
    let mut crowd = [0.0f32; HEADINGS];
    let e = &g.enemies;
    for i in 0..e.len() {
        let d = e.pos(i) - p;
        let l2 = d.length_squared();
        if l2 > BOT_LOOK * BOT_LOOK {
            continue;
        }
        for (k, dir) in dirs.iter().enumerate() {
            let along = d.dot(*dir);
            if along > 0.0 && d.perp_dot(*dir).abs() < BOT_LANE {
                crowd[k] += 1.0 - along / BOT_LOOK;
            }
        }
    }
    let mut best = (f32::MIN, want);
    for (k, &dir) in dirs.iter().enumerate() {
        let mut free = 0.0;
        while free < BOT_LOOK && !g.map.blocks_walk(p + dir * (free + PLAYER_RADIUS + 2.0)) {
            free += 8.0;
        }
        let blocked = if free < 16.0 { 2.0 } else { 0.0 };
        let open = g.map.clearance(p + dir * free) / 12.0;
        let score = dir.dot(want) + BOT_ROOM * free / BOT_LOOK + BOT_OPEN * open
            - BOT_CROWD * crowd[k]
            - blocked;
        if score > best.0 {
            best = (score, dir);
        }
    }
    best.1
}

/// Fights the boss like a player would: closes to `BOT_BOSS_RANGE` so her Arts reach it,
/// but steps sideways off a charge lane and back from a bash.
fn dodge_boss(g: &Game) -> Vec2 {
    let Some(b) = &g.boss else {
        return Vec2::ZERO;
    };
    let p = g.player.pos;
    let d = p - b.pos;
    let def = b.def;
    let dist = d.length().max(4.0);
    let mut away = d / (dist * dist);
    if dist > BOT_BOSS_RANGE {
        away -= d / dist / 256.0;
    }
    match b.mode {
        BossMode::Windup | BossMode::Charge => {
            let along = d.dot(b.dir);
            let side = d.perp_dot(b.dir);
            let lane = ENEMY_DEFS[def.kind].radius() + PLAYER_RADIUS + 10.0;
            let reach = def.charge_speed * def.charge_time;
            if along > -lane && along < reach && side.abs() < lane {
                let out = -b.dir.perp() * if side >= 0.0 { 1.0 } else { -1.0 };
                away += out / 64.0;
            }
        }
        BossMode::Bash => away += d.normalize_or_zero() / 64.0,
        _ => {}
    }
    away
}

/// How close the autopilot keeps to a boss.
const BOT_BOSS_RANGE: f32 = 100.0;

/// A focused build, closer to a player who knows the game: level the Arts she has,
/// take the Sins that evolve them, fill up to four Arts, then other Sins.
pub fn smart_pick(g: &Game) -> usize {
    let u = &g.upgrades;
    let score = |c: &Choice| match *c {
        Choice::Art(a) if u.arts[a] > 0 => 4.0 + u.arts[a] as f32 * 0.1,
        Choice::Art(_) if u.owned_arts() < 4 => 3.0,
        Choice::Art(_) => 0.5,
        Choice::Sin(s) => {
            let pairs = (0..ARTS.len())
                .any(|a| u.arts[a] > 0 && ARTS[a].evolution.as_ref().is_some_and(|e| e.sin == s));
            if pairs {
                3.5
            } else {
                1.0
            }
        }
    };
    (0..g.choices.len())
        .filter(|&i| g.choices[i].is_some())
        .max_by(|&a, &b| score(&g.choices[a].unwrap()).total_cmp(&score(&g.choices[b].unwrap())))
        .unwrap_or(0)
}

/// Who plays a run (F12 cycles it): she, the bot, or the bot with her kept alive.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Autopilot {
    #[default]
    Off,
    Smart,
    God,
}

impl Autopilot {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Smart,
            Self::Smart => Self::God,
            Self::God => Self::Off,
        }
    }

    /// Developer-only label, like the F3 overlay (not translated).
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "",
            Self::Smart => "BOT smart (F12)",
            Self::God => "BOT god (F12)",
        }
    }
}

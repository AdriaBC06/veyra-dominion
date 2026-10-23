//! The autopilot (F12 in a run, and the balance probe): kites the horde, gathers souls
//! and picks upgrades for a focused build. Simulation side: reads `Game`, returns input.

use crate::config::*;
use crate::game::Game;
use crate::upgrades::Choice;
use macroquad::math::Vec2;

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
    (away * urgency + tangent * 0.5 * urgency + goal * (1.2 - urgency * 0.6)).normalize_or_zero()
}

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

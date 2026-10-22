//! Headless balance probe (test only): a kiting bot plays whole runs and prints the
//! level, kills and health per minute. Not a pass/fail test; run it by hand:
//! `cargo test --release balance -- --ignored --nocapture`.

use crate::config::*;
use crate::game::{Game, InputState, SimEvent};
use macroquad::math::Vec2;
use macroquad::rand::{gen_range, srand};

/// Bot input: flee the nearby horde, circle around it, drift toward souls and Offerings.
fn steer(g: &Game) -> Vec2 {
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
    let goal = g
        .offerings
        .first()
        .map(|o| o.pos)
        .or_else(|| {
            g.souls
                .iter()
                .map(|s| s.pos)
                .min_by(|a, b| a.distance_squared(p).total_cmp(&b.distance_squared(p)))
        })
        .map_or(Vec2::ZERO, |q| (q - p).normalize_or_zero());
    (away * urgency + tangent * 0.5 * urgency + goal * (1.2 - urgency)).normalize_or_zero()
}

/// One run until death or `max_secs`; returns the per-minute rows and the death time.
fn play(seed: u64, max_secs: f32, verbose: bool) -> (Option<f32>, u32, u32) {
    srand(seed);
    let mut g = Game::new();
    let mut input = InputState::default();
    let mut next_minute = 60.0;
    loop {
        input.move_dir = steer(&g);
        match g.tick(&input) {
            SimEvent::LevelUp => {
                let n = g.choices.iter().flatten().count();
                g.choose_upgrade(gen_range(0, n));
            }
            SimEvent::Offering => g.accept_offering(),
            SimEvent::Victory => g.start_endless(),
            SimEvent::PlayerDied => return (Some(g.time), g.level, g.kills),
            SimEvent::None => {}
        }
        if g.time >= next_minute {
            if verbose {
                let arts: u32 = g.upgrades.arts.iter().map(|&l| l as u32).sum();
                let sins: u32 = g.upgrades.sins.iter().map(|&l| l as u32).sum();
                println!(
                    "  {:>2}:00 lv {:>3} kills {:>5} hp {:>5.0}/{:<4.0} alive {:>4} arts {:>2} sins {:>2}",
                    (g.time / 60.0).round(),
                    g.level,
                    g.kills,
                    g.player.hp,
                    g.player.max_hp,
                    g.enemies.len(),
                    arts,
                    sins
                );
            }
            next_minute += 60.0;
        }
        if g.time >= max_secs {
            return (None, g.level, g.kills);
        }
    }
}

#[test]
#[ignore]
fn balance() {
    let runs: u64 = std::env::var("BALANCE_RUNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    let max = VICTORY_TIME + 120.0;
    let mut deaths = Vec::new();
    for seed in 1..=runs {
        println!("run {seed}");
        let (died, level, kills) = play(seed, max, true);
        match died {
            Some(t) => println!(
                "  died at {}:{:02}, lv {level}, kills {kills}",
                t as u32 / 60,
                t as u32 % 60
            ),
            None => println!("  alive at the end, lv {level}, kills {kills}"),
        }
        deaths.push(died.unwrap_or(max));
    }
    deaths.sort_by(f32::total_cmp);
    println!("median survival {:.0} s", deaths[deaths.len() / 2]);
}

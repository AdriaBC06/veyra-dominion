//! Headless balance probe (test only): a kiting bot plays whole runs and prints the
//! level, kills and health per minute. Not a pass/fail test; run it by hand:
//! `cargo test --release balance -- --ignored --nocapture`. Picks are random unless
//! `BALANCE_BOT=smart` (a focused build, see `smart_pick`); `BALANCE_GOD=1` keeps her
//! alive to read the XP curve over the whole map.

use crate::config::*;
use crate::game::{Game, InputState, SimEvent};
use crate::upgrades::Choice;
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
fn smart_pick(g: &Game) -> usize {
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

fn smart_bot() -> bool {
    std::env::var("BALANCE_BOT").is_ok_and(|s| s == "smart")
}

/// One run until death or `max_secs`; returns the per-minute rows and the death time.
fn play(seed: u64, max_secs: f32, verbose: bool) -> (Option<f32>, u32, u32) {
    srand(seed);
    let mut g = Game::new();
    let only = only_art();
    if let Some(art) = only {
        // Swap the starting Hellflare for the Art under test.
        g.upgrades.arts[HELLFLARE] = 0;
        g.upgrades.arts[art] = 1;
        g.upgrades.slots[0] = Some(art);
        g.stats = g.upgrades.stats();
        g.arts.refresh(&g.upgrades);
    }
    let mut input = InputState::default();
    let mut next_minute = 60.0;
    let god = std::env::var("BALANCE_GOD").is_ok();
    loop {
        if god {
            g.player.hp = g.player.max_hp;
        }
        input.move_dir = steer(&g);
        match g.tick(&input) {
            SimEvent::LevelUp => {
                let n = g.choices.iter().flatten().count();
                let slot = match only {
                    // Only that Art and Sins; a random pick otherwise.
                    Some(art) => g
                        .choices
                        .iter()
                        .position(|c| *c == Some(Choice::Art(art)))
                        .or_else(|| {
                            g.choices
                                .iter()
                                .position(|c| matches!(c, Some(Choice::Sin(_))))
                        })
                        .unwrap_or(n),
                    None if smart_bot() => smart_pick(&g),
                    None => gen_range(0, n),
                };
                g.choose_upgrade(slot);
            }
            SimEvent::Offering => g.accept_offering(),
            SimEvent::Victory => g.start_endless(),
            SimEvent::PlayerDied => {
                print_build(&g);
                return (Some(g.time), g.level, g.kills);
            }
            SimEvent::None => {}
        }
        if g.time >= next_minute {
            if verbose {
                let arts: u32 = g.upgrades.arts.iter().map(|&l| l as u32).sum();
                let sins: u32 = g.upgrades.sins.iter().map(|&l| l as u32).sum();
                let ground: u32 = g.souls.iter().map(|s| s.xp).sum();
                println!(
                    "  {:>2}:00 lv {:>3} kills {:>5} hp {:>5.0}/{:<4.0} alive {:>4} arts {:>2} sins {:>2} ground xp {:>5}",
                    (g.time / 60.0).round(),
                    g.level,
                    g.kills,
                    g.player.hp,
                    g.player.max_hp,
                    g.enemies.len(),
                    arts,
                    sins,
                    ground
                );
            }
            next_minute += 60.0;
        }
        if g.time >= max_secs {
            print_build(&g);
            return (None, g.level, g.kills);
        }
    }
}

/// Her Arts (`index:level`, `*` evolved) in slot order.
fn print_build(g: &Game) {
    let u = &g.upgrades;
    let arts: Vec<String> = (0..ARTS.len())
        .filter(|&a| u.arts[a] > 0)
        .map(|a| format!("{a}:{}{}", u.arts[a], if u.evolved[a] { "*" } else { "" }))
        .collect();
    println!("  arts {}", arts.join(" "));
}

/// `BALANCE_ONLY=<index into ARTS>`: the bot starts with that Art instead of
/// Hellflare and never takes another, to see whether it carries a run alone.
fn only_art() -> Option<usize> {
    std::env::var("BALANCE_ONLY").ok()?.parse().ok()
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

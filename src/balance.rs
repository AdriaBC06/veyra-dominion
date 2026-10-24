//! Headless balance probe (test only): a kiting bot plays whole runs and prints the
//! level, kills and health per minute. Not a pass/fail test; run it by hand:
//! `cargo test --release balance -- --ignored --nocapture`. Picks are random unless
//! `BALANCE_BOT=smart` (a focused build, see `bot::smart_pick`); `BALANCE_GOD=1` keeps her
//! alive to read the XP curve over the whole map. Runs go on up to `BOSS_WINDOW` past the
//! map's end, to see whether and how fast the bot kills the boss.

use crate::bot::{smart_pick, steer};
use crate::config::*;
use crate::game::{Game, InputState, SimEvent};
use crate::upgrades::Choice;
use macroquad::rand::{gen_range, srand};

fn smart_bot() -> bool {
    std::env::var("BALANCE_BOT").is_ok_and(|s| s == "smart")
}

/// How long past `MAP_TIME` a run may fight the boss.
const BOSS_WINDOW: f32 = 300.0;

/// How a run ended: death time, boss kill time, level and kills.
struct Outcome {
    died: Option<f32>,
    won: Option<f32>,
    level: u32,
    kills: u32,
}

/// One run until death, the boss's death or `max_secs`, printing a row per minute.
fn play(seed: u64, max_secs: f32, verbose: bool) -> Outcome {
    srand(seed);
    // BALANCE_OPEN=1 plays on open ground, to compare against Ashford's layout.
    let mut g = if std::env::var("BALANCE_OPEN").is_ok() {
        Game::new()
    } else {
        Game::on_map(ASHFORD_MAP)
    };
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
            SimEvent::Victory => {
                print_build(&g);
                return Outcome {
                    died: None,
                    won: Some(g.time),
                    level: g.level,
                    kills: g.kills,
                };
            }
            SimEvent::PlayerDied => {
                print_build(&g);
                return Outcome {
                    died: Some(g.time),
                    won: None,
                    level: g.level,
                    kills: g.kills,
                };
            }
            SimEvent::None => {}
        }
        if g.time >= next_minute {
            if verbose {
                let arts: u32 = g.upgrades.arts.iter().map(|&l| l as u32).sum();
                let sins: u32 = g.upgrades.sins.iter().map(|&l| l as u32).sum();
                let ground: u32 = g.souls.iter().map(|s| s.xp).sum();
                let boss = g.boss.as_ref().map_or(String::new(), |b| {
                    format!(" boss {:.0}% ph {}", b.health() * 100.0, b.phase + 1)
                });
                println!(
                    "  {:>2}:00 lv {:>3} kills {:>5} hp {:>5.0}/{:<4.0} alive {:>4} arts {:>2} sins {:>2} ground xp {:>5}{boss}",
                    (g.time / 60.0).round(),
                    g.level,
                    g.kills,
                    g.player.hp,
                    g.player.max_hp,
                    g.enemies.len(),
                    arts,
                    sins,
                    ground,
                );
            }
            next_minute += 60.0;
        }
        if g.time >= max_secs {
            print_build(&g);
            return Outcome {
                died: None,
                won: None,
                level: g.level,
                kills: g.kills,
            };
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
    let max = MAP_TIME + BOSS_WINDOW;
    let clock = |t: f32| format!("{}:{:02}", t as u32 / 60, t as u32 % 60);
    let (mut deaths, mut fights) = (Vec::new(), Vec::new());
    for seed in 1..=runs {
        println!("run {seed}");
        let o = play(seed, max, true);
        let (level, kills) = (o.level, o.kills);
        match (o.died, o.won) {
            (_, Some(t)) => {
                println!(
                    "  boss down at {} ({:.0} s), lv {level}, kills {kills}",
                    clock(t),
                    t - MAP_TIME
                );
                fights.push(t - MAP_TIME);
            }
            (Some(t), _) => println!("  died at {}, lv {level}, kills {kills}", clock(t)),
            (None, None) => println!("  boss still up at the end, lv {level}, kills {kills}"),
        }
        deaths.push(o.died.unwrap_or(max));
    }
    deaths.sort_by(f32::total_cmp);
    fights.sort_by(f32::total_cmp);
    println!("median survival {:.0} s", deaths[deaths.len() / 2]);
    match fights.get(fights.len() / 2) {
        Some(m) => println!(
            "boss killed in {} of {runs} runs, median fight {m:.0} s",
            fights.len()
        ),
        None => println!("boss never killed"),
    }
}

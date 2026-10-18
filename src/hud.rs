//! Screen-space UI drawn into the logical frame (call with the UI camera set).

use crate::assets::{draw_ui_text, measure_ui_text, TextStyle};
use crate::config::*;
use crate::game::Game;
use macroquad::prelude::*;

const TITLE_COLOR: Color = Color::new(0.88, 0.19, 0.29, 1.0);

fn draw_text_centered(text: &str, y: f32, style: TextStyle, color: Color) {
    let dims = measure_ui_text(text, style);
    draw_ui_text(text, (LOGICAL_W - dims.width) * 0.5, y, style, color);
}

/// `m:ss` for a time in seconds.
pub fn format_time(t: f32) -> String {
    let s = t.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

const HP_COLOR: Color = Color::new(0.78, 0.16, 0.22, 1.0);
const XP_COLOR: Color = Color::new(0.45, 0.72, 0.95, 1.0);
const BAR_BG: Color = Color::new(0.0, 0.0, 0.0, 0.55);

fn draw_bar(x: f32, y: f32, w: f32, h: f32, frac: f32, color: Color) {
    draw_rectangle(x, y, w, h, BAR_BG);
    draw_rectangle(x, y, (w * frac.clamp(0.0, 1.0)).round(), h, color);
}

/// XP bar across the top, HP bar under it, timer centred, level and kills on the right.
pub fn draw_hud(game: &Game) {
    let p = &game.player;
    draw_bar(
        0.0,
        0.0,
        LOGICAL_W,
        4.0,
        game.xp as f32 / xp_to_next(game.level) as f32,
        XP_COLOR,
    );
    draw_bar(6.0, 9.0, 100.0, 8.0, p.hp / p.max_hp, HP_COLOR);
    draw_rectangle_lines(6.0, 9.0, 100.0, 8.0, 1.0, BLACK);
    let hp = format!("{}/{}", p.hp.ceil(), p.max_hp);
    draw_ui_text(&hp, 112.0, 16.0, TextStyle::Body, WHITE);
    draw_text_centered(&format_time(game.time), 22.0, TextStyle::Large, WHITE);
    let right = format!("LV {}   KILLS {}", game.level, game.kills);
    let dims = measure_ui_text(&right, TextStyle::Body);
    draw_ui_text(
        &right,
        LOGICAL_W - dims.width - 6.0,
        16.0,
        TextStyle::Body,
        WHITE,
    );
}

/// F3 overlay: frame timing and entity counts.
pub fn draw_debug(game: &Game, fps: i32, update_ms: f32) {
    let mut lines = vec![
        format!("FPS {fps}   update {update_ms:.2} ms"),
        format!("enemies {}/{MAX_ENEMIES}", game.enemies.len()),
    ];
    for (k, def) in ENEMY_DEFS.iter().enumerate() {
        let n = game
            .enemies
            .kind
            .iter()
            .filter(|&&kind| kind as usize == k)
            .count();
        lines.push(format!("  {} {n}", def.name));
    }
    lines.push(format!(
        "proj {}  souls {}  fades {}",
        game.projectiles.len(),
        game.souls.len(),
        game.fades.len()
    ));
    lines.push(format!("spawn {:.1}/s", Game::spawn_rate(game.time)));
    let h = lines.len() as f32 * 10.0 + 6.0;
    draw_rectangle(4.0, 26.0, 200.0, h, Color::new(0.0, 0.0, 0.0, 0.6));
    for (i, line) in lines.iter().enumerate() {
        draw_ui_text(line, 8.0, 38.0 + i as f32 * 10.0, TextStyle::Body, LIME);
    }
}

/// Dims the frame and shows a title with lines of text below it.
pub fn draw_center_screen(title: &str, lines: &[&str]) {
    draw_center_screen_styled(title, TextStyle::Title, lines);
}

/// Title screen: like `draw_center_screen` with the game title in the logo style.
pub fn draw_title_screen(lines: &[&str]) {
    draw_center_screen_styled(GAME_TITLE, TextStyle::Logo, lines);
}

fn draw_center_screen_styled(title: &str, style: TextStyle, lines: &[&str]) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_W,
        LOGICAL_H,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    let mut y = (LOGICAL_H * 0.4).round();
    draw_text_centered(title, y, style, TITLE_COLOR);
    y += 30.0;
    for line in lines {
        draw_text_centered(line, y, TextStyle::Body, WHITE);
        y += 14.0;
    }
}

/// Level-up screen: up to three upgrade cards, `selected` highlighted.
pub fn draw_level_up(game: &Game, selected: usize) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_W,
        LOGICAL_H,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    draw_text_centered("Dominion grows", 70.0, TextStyle::Title, TITLE_COLOR);
    let (w, h, gap) = (180.0, 110.0, 16.0);
    let x0 = ((LOGICAL_W - 3.0 * w - 2.0 * gap) * 0.5).round();
    let y = 110.0;
    for (slot, choice) in game.choices.iter().enumerate() {
        let Some(idx) = *choice else { continue };
        let def = &UPGRADES[idx];
        let x = x0 + slot as f32 * (w + gap);
        let border = if slot == selected { TITLE_COLOR } else { GRAY };
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.04, 0.1, 0.95));
        draw_rectangle_lines(x, y, w, h, 2.0, border);
        let num = (slot + 1).to_string();
        draw_ui_text(&num, x + 8.0, y + 18.0, TextStyle::Body, GRAY);
        draw_ui_text(def.name, x + 20.0, y + 20.0, TextStyle::Heading, WHITE);
        draw_ui_text(def.desc, x + 8.0, y + 50.0, TextStyle::Body, LIGHTGRAY);
        let lv = format!("Lv {} / {}", game.upgrades.levels[idx], def.max_level);
        draw_ui_text(&lv, x + 8.0, y + h - 10.0, TextStyle::Body, GRAY);
    }
    draw_text_centered(
        "1/2/3 or arrows + Enter",
        y + h + 30.0,
        TextStyle::Body,
        WHITE,
    );
}

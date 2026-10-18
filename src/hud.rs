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
    draw_ui_text(&hp, 112.0, 21.0, TextStyle::Body, WHITE);
    draw_text_centered(&format_time(game.time), 21.0, TextStyle::Body, WHITE);
    let right = format!("LV {}   KILLS {}", game.level, game.kills);
    let dims = measure_ui_text(&right, TextStyle::Body);
    draw_ui_text(
        &right,
        LOGICAL_W - dims.width - 6.0,
        21.0,
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
    let h = lines.len() as f32 * 11.0 + 6.0;
    draw_rectangle(4.0, 28.0, 200.0, h, Color::new(0.0, 0.0, 0.0, 0.6));
    for (i, line) in lines.iter().enumerate() {
        draw_ui_text(line, 8.0, 40.0 + i as f32 * 11.0, TextStyle::Small, LIME);
    }
}

/// Dims the frame and shows a title with lines of text below it.
pub fn draw_center_screen(title: &str, lines: &[&str]) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_W,
        LOGICAL_H,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    let mut y = (LOGICAL_H * 0.4).round();
    draw_text_centered(title, y, TextStyle::Title, TITLE_COLOR);
    y += 32.0;
    for line in lines {
        draw_text_centered(line, y, TextStyle::Body, WHITE);
        y += 18.0;
    }
}

/// Level-up screen: up to three stacked upgrade cards, `selected` highlighted.
pub fn draw_level_up(game: &Game, selected: usize) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_W,
        LOGICAL_H,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    draw_text_centered("Dominion grows", 78.0, TextStyle::Title, TITLE_COLOR);
    let (w, h, gap) = (420.0, 56.0, 8.0);
    let x = ((LOGICAL_W - w) * 0.5).round();
    let y0 = 100.0;
    for (slot, choice) in game.choices.iter().enumerate() {
        let Some(idx) = *choice else { continue };
        let def = &UPGRADES[idx];
        let y = y0 + slot as f32 * (h + gap);
        let border = if slot == selected { TITLE_COLOR } else { GRAY };
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.04, 0.1, 0.95));
        draw_rectangle_lines(x, y, w, h, 2.0, border);
        let num = (slot + 1).to_string();
        draw_ui_text(&num, x + 10.0, y + 22.0, TextStyle::Body, GRAY);
        draw_ui_text(def.name, x + 28.0, y + 22.0, TextStyle::Body, WHITE);
        let lv = format!("Lv {} / {}", game.upgrades.levels[idx], def.max_level);
        let lv_w = measure_ui_text(&lv, TextStyle::Body).width;
        draw_ui_text(&lv, x + w - lv_w - 10.0, y + 22.0, TextStyle::Body, GRAY);
        draw_ui_text(def.desc, x + 28.0, y + 45.0, TextStyle::Body, LIGHTGRAY);
    }
    draw_text_centered(
        "1/2/3 or arrows + Enter",
        y0 + 3.0 * (h + gap) + 22.0,
        TextStyle::Body,
        WHITE,
    );
}

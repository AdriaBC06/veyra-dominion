//! Screen-space UI drawn into the logical frame (call with the UI camera set).

use crate::config::*;
use crate::game::Game;
use macroquad::prelude::*;

const TITLE_COLOR: Color = Color::new(0.88, 0.19, 0.29, 1.0);

fn draw_text_centered(text: &str, y: f32, size: u16, color: Color) {
    let dims = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        ((LOGICAL_W - dims.width) * 0.5).round(),
        y,
        size as f32,
        color,
    );
}

/// `m:ss` for a time in seconds.
pub fn format_time(t: f32) -> String {
    let s = t.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

/// Minimal readout until the full HUD lands (T12).
pub fn draw_hud(game: &Game) {
    let p = &game.player;
    let text = format!(
        "HP {}/{}   LV {} ({}/{})   KILLS {}   {}",
        p.hp.ceil(),
        p.max_hp,
        game.level,
        game.xp,
        xp_to_next(game.level),
        game.kills,
        format_time(game.time)
    );
    draw_text(&text, 6.0, 14.0, 16.0, WHITE);
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
    draw_text_centered(title, y, 32, TITLE_COLOR);
    y += 30.0;
    for line in lines {
        draw_text_centered(line, y, 16, WHITE);
        y += 18.0;
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
    draw_text_centered("Dominion grows", 70.0, 32, TITLE_COLOR);
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
        draw_text((slot + 1).to_string(), x + 8.0, y + 18.0, 16.0, GRAY);
        draw_text(def.name, x + 24.0, y + 18.0, 16.0, WHITE);
        draw_text(def.desc, x + 8.0, y + 50.0, 16.0, LIGHTGRAY);
        let lv = format!("Lv {} / {}", game.upgrades.levels[idx], def.max_level);
        draw_text(&lv, x + 8.0, y + h - 10.0, 16.0, GRAY);
    }
    draw_text_centered("1/2/3 or arrows + Enter", y + h + 30.0, 16, WHITE);
}

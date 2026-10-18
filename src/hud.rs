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
        "HP {}/{}   KILLS {}   {}",
        p.hp.ceil(),
        p.max_hp,
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

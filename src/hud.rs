//! Screen-space UI drawn into the logical frame (call with the UI camera set).

use crate::assets::{draw_ui_text, measure_ui_text, TextStyle};
use crate::config::*;
use crate::game::Game;
use crate::i18n::{tr, trf, Key};
use crate::ui::Focus;
use macroquad::prelude::*;

const TITLE_COLOR: Color = Color::new(0.88, 0.19, 0.29, 1.0);

/// Horizontal edge of the view a widget is attached to.
#[derive(Clone, Copy)]
pub enum Anchor {
    Center,
    Right,
}

impl Anchor {
    /// Left x of a widget `width` wide, `margin` in from the anchored edge.
    pub fn x(self, view: Vec2, width: f32, margin: f32) -> f32 {
        match self {
            Anchor::Center => ((view.x - width) * 0.5).round(),
            Anchor::Right => (view.x - width - margin).round(),
        }
    }
}

fn draw_text_centered(view: Vec2, text: &str, y: f32, style: TextStyle, color: Color) {
    let dims = measure_ui_text(text, style);
    let x = Anchor::Center.x(view, dims.width, 0.0);
    draw_ui_text(text, x, y, style, color);
}

fn draw_dim(view: Vec2) {
    draw_rectangle(0.0, 0.0, view.x, view.y, Color::new(0.0, 0.0, 0.0, 0.6));
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
pub fn draw_hud(game: &Game, view: Vec2) {
    let p = &game.player;
    draw_bar(
        0.0,
        0.0,
        view.x,
        4.0,
        game.xp as f32 / xp_to_next(game.level) as f32,
        XP_COLOR,
    );
    draw_bar(6.0, 9.0, 100.0, 8.0, p.hp / p.max_hp, HP_COLOR);
    draw_rectangle_lines(6.0, 9.0, 100.0, 8.0, 1.0, BLACK);
    let hp = format!("{}/{}", p.hp.ceil(), p.max_hp);
    draw_ui_text(&hp, 112.0, 21.0, TextStyle::Body, WHITE);
    draw_text_centered(view, &format_time(game.time), 21.0, TextStyle::Body, WHITE);
    let right = trf(Key::HudRight, &[&game.level, &game.kills]);
    let dims = measure_ui_text(&right, TextStyle::Body);
    let x = Anchor::Right.x(view, dims.width, 6.0);
    draw_ui_text(&right, x, 21.0, TextStyle::Body, WHITE);
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
pub fn draw_center_screen(view: Vec2, title: &str, lines: &[&str]) {
    draw_dim(view);
    let mut y = (view.y * 0.4).round();
    draw_text_centered(view, title, y, TextStyle::Title, TITLE_COLOR);
    y += 32.0;
    for line in lines {
        draw_text_centered(view, line, y, TextStyle::Body, WHITE);
        y += 18.0;
    }
}

const CARD_W: f32 = 420.0;
const CARD_H: f32 = 56.0;
const CARD_GAP: f32 = 8.0;
const CARDS_Y: f32 = 100.0;

/// Hit rects of the level-up cards, one per choice slot.
pub fn level_up_cards(view: Vec2) -> [Rect; 3] {
    let x = Anchor::Center.x(view, CARD_W, 0.0);
    std::array::from_fn(|slot| {
        let y = CARDS_Y + slot as f32 * (CARD_H + CARD_GAP);
        Rect::new(x, y, CARD_W, CARD_H)
    })
}

/// Level-up screen: up to three stacked upgrade cards, the highlighted one outlined.
pub fn draw_level_up(game: &Game, view: Vec2, focus: &Focus) {
    draw_dim(view);
    draw_text_centered(
        view,
        tr(Key::LevelUpTitle),
        78.0,
        TextStyle::Title,
        TITLE_COLOR,
    );
    let cards = level_up_cards(view);
    for (slot, choice) in game.choices.iter().enumerate() {
        let Some(idx) = *choice else { continue };
        let def = &UPGRADES[idx];
        let Rect { x, y, w, h } = cards[slot];
        let border = if focus.is_highlighted(slot) {
            TITLE_COLOR
        } else {
            GRAY
        };
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.04, 0.1, 0.95));
        draw_rectangle_lines(x, y, w, h, 2.0, border);
        let num = (slot + 1).to_string();
        draw_ui_text(&num, x + 10.0, y + 22.0, TextStyle::Body, GRAY);
        draw_ui_text(tr(def.name), x + 28.0, y + 22.0, TextStyle::Body, WHITE);
        let lv = trf(
            Key::LevelUpLevel,
            &[&game.upgrades.levels[idx], &def.max_level],
        );
        let lv_w = measure_ui_text(&lv, TextStyle::Body).width;
        draw_ui_text(&lv, x + w - lv_w - 10.0, y + 22.0, TextStyle::Body, GRAY);
        draw_ui_text(tr(def.desc), x + 28.0, y + 45.0, TextStyle::Body, LIGHTGRAY);
    }
    draw_text_centered(
        view,
        tr(Key::LevelUpHelp),
        CARDS_Y + 3.0 * (CARD_H + CARD_GAP) + 22.0,
        TextStyle::Body,
        WHITE,
    );
}

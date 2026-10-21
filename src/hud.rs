//! Screen-space UI drawn into the logical frame (call with the UI camera set).

use crate::assets::{draw_ui_text, measure_ui_text, TextStyle};
use crate::config::*;
use crate::game::{Game, Reward};
use crate::i18n::{tr, trf, Key};
use crate::ui::Focus;
use crate::upgrades::Choice;
use macroquad::prelude::*;

pub const TITLE_COLOR: Color = Color::new(0.88, 0.19, 0.29, 1.0);

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

pub fn draw_text_centered(view: Vec2, text: &str, y: f32, style: TextStyle, color: Color) {
    let dims = measure_ui_text(text, style);
    let x = Anchor::Center.x(view, dims.width, 0.0);
    draw_ui_text(text, x, y, style, color);
}

pub fn draw_dim(view: Vec2) {
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
        game.arts.projectiles.len(),
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

const SCREEN_TITLE_Y: f32 = 96.0;

/// Dims the frame and shows a title with lines of text below it.
pub fn draw_center_screen(view: Vec2, title: &str, lines: &[&str]) {
    draw_dim(view);
    draw_center_text(view, title, lines);
}

/// The title and lines of `draw_center_screen`, without the dim.
pub fn draw_center_text(view: Vec2, title: &str, lines: &[&str]) {
    let mut y = SCREEN_TITLE_Y;
    draw_text_centered(view, title, y, TextStyle::Title, TITLE_COLOR);
    y += 32.0;
    for line in lines {
        draw_text_centered(view, line, y, TextStyle::Body, WHITE);
        y += 18.0;
    }
}

/// Top of the button column under a centre screen with `lines` lines of text.
pub fn center_screen_buttons(lines: usize) -> f32 {
    SCREEN_TITLE_Y + 20.0 + 24.0 * lines as f32
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
        let Some(choice) = *choice else { continue };
        let (name, lv, desc) = card_text(game, choice);
        let num = (slot + 1).to_string();
        let lit = focus.is_highlighted(slot);
        draw_card(cards[slot], &num, name, &lv, &desc, lit);
    }
    draw_text_centered(
        view,
        tr(Key::LevelUpHelp),
        CARDS_Y + 3.0 * (CARD_H + CARD_GAP) + 22.0,
        TextStyle::Body,
        WHITE,
    );
}

/// A card: `num` in the corner, name with a tag on the right, a description below.
fn draw_card(rect: Rect, num: &str, name: &str, tag: &str, desc: &str, lit: bool) {
    let Rect { x, y, w, h } = rect;
    let border = if lit { TITLE_COLOR } else { GRAY };
    draw_rectangle(x, y, w, h, Color::new(0.08, 0.04, 0.1, 0.95));
    draw_rectangle_lines(x, y, w, h, 2.0, border);
    draw_ui_text(num, x + 10.0, y + 22.0, TextStyle::Body, GRAY);
    draw_ui_text(name, x + 28.0, y + 22.0, TextStyle::Body, WHITE);
    let tag_w = measure_ui_text(tag, TextStyle::Body).width;
    draw_ui_text(tag, x + w - tag_w - 10.0, y + 22.0, TextStyle::Body, GRAY);
    draw_ui_text(desc, x + 28.0, y + 45.0, TextStyle::Body, LIGHTGRAY);
}

/// Name, level tag and description of a level-up card. A new Art or Sin shows what
/// it does; an owned Art shows what its next level adds.
fn card_text(game: &Game, choice: Choice) -> (&'static str, String, String) {
    match choice {
        Choice::Sin(i) => {
            let def = &SINS[i];
            let lv = game.upgrades.sins[i];
            let tag = if lv == 0 {
                tr(Key::LevelUpNew).to_string()
            } else {
                trf(Key::LevelUpLevel, &[&lv, &def.max_level])
            };
            (tr(def.name), tag, tr(def.desc).to_string())
        }
        Choice::Art(i) => {
            let def = &ARTS[i];
            let lv = game.upgrades.arts[i];
            if lv == 0 {
                let new = tr(Key::LevelUpNew).to_string();
                return (tr(def.name), new, tr(def.desc).to_string());
            }
            let tag = trf(Key::LevelUpLevel, &[&lv, &ART_MAX_LEVEL]);
            (
                tr(def.name),
                tag,
                gain_text(def, &def.gains[lv as usize - 1]),
            )
        }
    }
}

/// The two cards on the Abyssal Offering screen: what it holds, and its sacrifice.
pub fn offering_cards(view: Vec2) -> [Rect; 2] {
    let cards = level_up_cards(view);
    [cards[1], cards[2]]
}

/// Abyssal Offering screen: what it holds on the first card, the sacrifice (a minor
/// reward) on the second.
pub fn draw_offering(game: &Game, view: Vec2, ready: bool, focus: &Focus) {
    draw_dim(view);
    draw_text_centered(
        view,
        tr(Key::OfferingTitle),
        78.0,
        TextStyle::Title,
        TITLE_COLOR,
    );
    let Some(reward) = game.reward else { return };
    let (line, name, tag, desc) = match reward {
        Reward::Evolution(i) => {
            let evo = &ARTS[i].evolution;
            let line = trf(Key::OfferingEvolved, &[&tr(ARTS[i].name)]);
            (
                line,
                tr(evo.name).to_string(),
                String::new(),
                tr(evo.desc).to_string(),
            )
        }
        // Not taken yet: show the level it would reach and what that gives.
        Reward::Gift(choice) => {
            let (name, tag, desc) = card_text(game, choice);
            (
                tr(Key::OfferingGift).to_string(),
                name.to_string(),
                tag,
                desc,
            )
        }
        Reward::Heal => {
            let hp = trf(Key::OfferingHealDesc, &[&OFFERING_HEAL]);
            (
                tr(Key::OfferingHeal).to_string(),
                hp,
                String::new(),
                String::new(),
            )
        }
    };
    draw_text_centered(view, &line, CARDS_Y + 22.0, TextStyle::Body, WHITE);
    let [accept, sacrifice] = offering_cards(view);
    let accept_tag = tr(Key::OfferingAccept);
    let tag = if tag.is_empty() {
        accept_tag.to_string()
    } else {
        tag
    };
    draw_card(accept, "1", &name, &tag, &desc, focus.is_highlighted(0));
    let gain = trf(
        Key::OfferingSacrificeDesc,
        &[&SACRIFICE_HEAL, &SACRIFICE_XP],
    );
    let lit = focus.is_highlighted(1);
    draw_card(sacrifice, "2", tr(Key::OfferingSacrifice), "", &gain, lit);
    if ready {
        let y = CARDS_Y + 3.0 * (CARD_H + CARD_GAP) + 22.0;
        draw_text_centered(view, tr(Key::OfferingHelp), y, TextStyle::Body, WHITE);
    }
}

/// "+1 heart, +4 damage": every non-zero part of a level gain.
fn gain_text(def: &ArtDef, g: &ArtGain) -> String {
    let pct = |f: f32| (f * 100.0).round() as u32;
    let parts = [
        (g.amount > 0, def.amount_key, g.amount.to_string()),
        (g.damage > 0.0, Key::ArtGainDamage, g.damage.to_string()),
        (
            g.cooldown > 0.0,
            Key::ArtGainCooldown,
            pct(g.cooldown).to_string(),
        ),
        (g.area > 0.0, Key::ArtGainArea, pct(g.area).to_string()),
        (g.speed > 0.0, Key::ArtGainSpeed, pct(g.speed).to_string()),
        (g.pierce > 0, Key::ArtGainPierce, g.pierce.to_string()),
        (
            g.duration > 0.0,
            Key::ArtGainDuration,
            g.duration.to_string(),
        ),
    ];
    let texts: Vec<String> = parts
        .iter()
        .filter(|p| p.0)
        .map(|(_, key, n)| trf(*key, &[n]))
        .collect();
    texts.join(", ")
}

// --- Debug panel (F2) ------------------------------------------------------
// Developer-only, like the F3 overlay: labels stay in English, names go through `tr`.

const DBG_TOP: f32 = 44.0;
const DBG_ROW: f32 = 18.0;
const DBG_ROW_H: f32 = 16.0;
const DBG_NAME_W: f32 = 230.0;
const DBG_EVO_W: f32 = 134.0;
const DBG_SIN_W: f32 = 230.0;
const DBG_OWNED: Color = Color::new(0.42, 0.06, 0.16, 0.95);
const DBG_FREE: Color = Color::new(0.08, 0.06, 0.1, 0.92);
const DBG_GOLD: Color = Color::new(0.91, 0.69, 0.29, 1.0);

/// Hit rects of the debug panel: one row per Art, then one Evolve button per Art,
/// then one row per Sin.
pub fn debug_rects(view: Vec2) -> Vec<Rect> {
    let x0 = Anchor::Center.x(view, DBG_NAME_W + DBG_EVO_W + 10.0 + DBG_SIN_W, 0.0);
    let row = |k: usize| DBG_TOP + 14.0 + k as f32 * DBG_ROW;
    let arts = (0..ARTS.len()).map(|k| Rect::new(x0, row(k), DBG_NAME_W, DBG_ROW_H));
    let evos = (0..ARTS.len())
        .map(|k| Rect::new(x0 + DBG_NAME_W + 4.0, row(k), DBG_EVO_W - 4.0, DBG_ROW_H));
    let sin_x = x0 + DBG_NAME_W + DBG_EVO_W + 10.0;
    let sins = (0..SINS.len()).map(|k| Rect::new(sin_x, row(k), DBG_SIN_W, DBG_ROW_H));
    arts.chain(evos).chain(sins).collect()
}

/// A panel row: filled when owned, dimmed when it can't be taken, outlined when focused.
fn debug_row(r: Rect, left: &str, right: &str, owned: bool, dim: bool, lit: bool, color: Color) {
    draw_rectangle(r.x, r.y, r.w, r.h, if owned { DBG_OWNED } else { DBG_FREE });
    let border = if lit {
        WHITE
    } else if owned {
        color
    } else {
        DARKGRAY
    };
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, border);
    let text = if dim {
        DARKGRAY
    } else if owned {
        color
    } else {
        GRAY
    };
    let y = r.y + 12.0;
    draw_ui_text(left, r.x + 5.0, y, TextStyle::Small, text);
    let w = measure_ui_text(right, TextStyle::Small).width;
    draw_ui_text(right, r.x + r.w - w - 5.0, y, TextStyle::Small, text);
}

/// Debug panel: every Art and Sin, what is owned and at what level, the slot limits,
/// and an Evolve button per Art (lit when it can evolve: top level and its Sin owned).
pub fn draw_debug_panel(game: &Game, view: Vec2, focus: &Focus) {
    draw_dim(view);
    let u = &game.upgrades;
    let rects = debug_rects(view);
    let (n, arts_full, sins_full) = (
        ARTS.len(),
        u.owned_arts() >= MAX_ARTS,
        u.owned_sins() >= MAX_SINS,
    );
    let header = |r: Rect, text: String, full: bool| {
        let color = if full { TITLE_COLOR } else { WHITE };
        draw_ui_text(&text, r.x, DBG_TOP + 8.0, TextStyle::Small, color);
    };
    let full = |f: bool| if f { "  FULL" } else { "" };
    header(
        rects[0],
        format!("ARTS {}/{MAX_ARTS}{}", u.owned_arts(), full(arts_full)),
        arts_full,
    );
    header(
        rects[2 * n],
        format!("SINS {}/{MAX_SINS}{}", u.owned_sins(), full(sins_full)),
        sins_full,
    );
    for (i, def) in ARTS.iter().enumerate() {
        let lv = u.arts[i];
        let color = if u.evolved[i] { DBG_GOLD } else { WHITE };
        let level = if lv > 0 {
            format!("{lv}/{ART_MAX_LEVEL}")
        } else {
            "-".to_string()
        };
        let dim = lv == 0 && arts_full;
        debug_row(
            rects[i],
            tr(def.name),
            &level,
            lv > 0,
            dim,
            focus.is_highlighted(i),
            color,
        );
        let sin = tr(SINS[def.evolution.sin].name);
        let (label, owned, dim, color) = if u.evolved[i] {
            ("EVOLVED".to_string(), true, false, DBG_GOLD)
        } else if u.can_evolve(i) {
            ("EVOLVE".to_string(), true, false, WHITE)
        } else {
            (format!("+{sin}"), false, true, GRAY)
        };
        let lit = focus.is_highlighted(n + i);
        debug_row(rects[n + i], &label, "", owned, dim, lit, color);
    }
    for (i, def) in SINS.iter().enumerate() {
        let lv = u.sins[i];
        let level = if lv > 0 {
            format!("{lv}/{}", def.max_level)
        } else {
            "-".to_string()
        };
        let dim = lv == 0 && sins_full;
        let lit = focus.is_highlighted(2 * n + i);
        debug_row(
            rects[2 * n + i],
            tr(def.name),
            &level,
            lv > 0,
            dim,
            lit,
            WHITE,
        );
    }
    let help_y = DBG_TOP + 14.0 + SINS.len() as f32 * DBG_ROW + 14.0;
    for (k, line) in [
        "Click / Enter / Right: level up    Right-click / Left: level down",
        "Evolve needs Lv 8 and the Sin shown    F2 / Esc: close",
    ]
    .iter()
    .enumerate()
    {
        let w = measure_ui_text(line, TextStyle::Small).width;
        draw_ui_text(
            line,
            ((view.x - w) * 0.5).round(),
            help_y + k as f32 * 12.0,
            TextStyle::Small,
            LIGHTGRAY,
        );
    }
}

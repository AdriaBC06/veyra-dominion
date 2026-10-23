//! Screen-space UI drawn into the logical frame (call with the UI camera set).

use crate::assets::{draw_ui_text, measure_ui_text, sprite, SpriteId, TextStyle, ICON_SIZE};
use crate::config::*;
use crate::game::{Game, Reward};
use crate::i18n::{tr, trf, Key};
use crate::ui::Focus;
use crate::upgrades::{Choice, Upgrades};
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

/// An icon in the `Icons` strip (`assets/icons.px.js` writes it in this order).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Art(usize),
    /// Art `i`'s evolution (only Arts that have one get a frame).
    Evolution(usize),
    Union(usize),
    Sin(usize),
}

/// How many of the first `n` Arts have an evolution.
const fn evolutions_before(n: usize) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < n {
        if ARTS[i].evolution.is_some() {
            count += 1;
        }
        i += 1;
    }
    count
}

const FIRST_EVOLUTION_ICON: usize = ARTS.len();
const FIRST_UNION_ICON: usize = FIRST_EVOLUTION_ICON + evolutions_before(ARTS.len());
const FIRST_SIN_ICON: usize = FIRST_UNION_ICON + UNIONS.len();
pub const ICON_COUNT: usize = FIRST_SIN_ICON + SINS.len();

impl Icon {
    pub const fn frame(self) -> usize {
        match self {
            Icon::Art(i) => i,
            Icon::Evolution(i) => FIRST_EVOLUTION_ICON + evolutions_before(i),
            Icon::Union(u) => FIRST_UNION_ICON + u,
            Icon::Sin(i) => FIRST_SIN_ICON + i,
        }
    }

    /// What owned Art `i` shows: its union, else its evolution, else itself.
    pub fn of_art(upgrades: &Upgrades, i: usize) -> Icon {
        match upgrades.union_of(i) {
            Some(u) => Icon::Union(u),
            None if upgrades.evolved[i] => Icon::Evolution(i),
            None => Icon::Art(i),
        }
    }

    fn of_choice(choice: Choice) -> Icon {
        match choice {
            Choice::Art(i) => Icon::Art(i),
            Choice::Sin(i) => Icon::Sin(i),
        }
    }
}

/// Draws `icon` with its top-left at (`x`, `y`), `scale` times its pixel size.
pub fn draw_icon(icon: Icon, x: f32, y: f32, scale: f32) {
    let (tex, src) = sprite(SpriteId::Icons, icon.frame());
    let params = DrawTextureParams {
        source: Some(src),
        dest_size: Some(Vec2::splat(ICON_SIZE * scale)),
        ..Default::default()
    };
    draw_texture_ex(&tex, x, y, WHITE, params);
}

const HP_COLOR: Color = Color::new(0.78, 0.16, 0.22, 1.0);
const XP_COLOR: Color = Color::new(0.45, 0.72, 0.95, 1.0);
const BAR_BG: Color = Color::new(0.0, 0.0, 0.0, 0.55);
/// The boss bar wears the Watch's royal blue.
const BOSS_COLOR: Color = Color::new(0.27, 0.38, 0.63, 1.0);
const BOSS_BAR_W: f32 = 300.0;
const BOSS_BAR_H: f32 = 6.0;
/// Bar top and name baseline, up from the bottom edge of the view.
const BOSS_BAR_UP: f32 = 14.0;
const BOSS_NAME_UP: f32 = 19.0;

fn draw_bar(x: f32, y: f32, w: f32, h: f32, frac: f32, color: Color) {
    draw_rectangle(x, y, w, h, BAR_BG);
    draw_rectangle(x, y, (w * frac.clamp(0.0, 1.0)).round(), h, color);
}

/// The HUD's rows of owned Arts (slot order) and Sins, under the HP bar.
const HUD_ICONS_X: f32 = 6.0;
const HUD_ICONS_Y: f32 = 26.0;
const HUD_ICON_STEP: f32 = ICON_SIZE + 2.0;

/// XP bar across the top, HP bar under it, timer centred, level and kills on the right;
/// owned Arts and Sins below the HP bar.
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
    let u = &game.upgrades;
    let x = |n: usize| HUD_ICONS_X + n as f32 * HUD_ICON_STEP;
    for (n, &i) in u.slots.iter().flatten().enumerate() {
        draw_icon(Icon::of_art(u, i), x(n), HUD_ICONS_Y, 1.0);
    }
    let sins = (0..SINS.len()).filter(|&i| u.sins[i] > 0);
    for (n, i) in sins.enumerate() {
        draw_icon(Icon::Sin(i), x(n), HUD_ICONS_Y + HUD_ICON_STEP, 1.0);
    }
    draw_boss_bar(game, view);
}

/// While a boss lives: its name over a health bar along the bottom, notched where its
/// later phases begin.
fn draw_boss_bar(game: &Game, view: Vec2) {
    let Some(b) = &game.boss else {
        return;
    };
    let w = BOSS_BAR_W.min(view.x - 40.0);
    let x = Anchor::Center.x(view, w, 0.0);
    let y = view.y - BOSS_BAR_UP;
    draw_text_centered(
        view,
        tr(b.def.name),
        view.y - BOSS_NAME_UP,
        TextStyle::Body,
        WHITE,
    );
    draw_bar(x, y, w, BOSS_BAR_H, b.health(), BOSS_COLOR);
    for phase in &b.def.phases[1..] {
        let nx = (x + w * phase.below).round();
        draw_line(nx, y, nx, y + BOSS_BAR_H, 1.0, BLACK);
    }
    draw_rectangle_lines(x, y, w, BOSS_BAR_H, 1.0, BLACK);
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
    draw_rectangle(4.0, 64.0, 200.0, h, Color::new(0.0, 0.0, 0.0, 0.6));
    for (i, line) in lines.iter().enumerate() {
        draw_ui_text(line, 8.0, 76.0 + i as f32 * 11.0, TextStyle::Small, LIME);
    }
}

/// F12: which bot is playing, top right under the HUD (developer-only, not translated).
pub fn draw_autopilot(view: Vec2, label: &str) {
    let w = measure_ui_text(label, TextStyle::Small).width;
    draw_ui_text(label, view.x - w - 8.0, 76.0, TextStyle::Small, LIME);
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

/// Cards take the view's width up to this, less a margin on each side.
const CARD_MAX_W: f32 = 520.0;
const CARD_MARGIN: f32 = 12.0;
/// Tall enough for the name and two lines of description.
const CARD_H: f32 = 70.0;
const CARD_GAP: f32 = 6.0;
const CARDS_Y: f32 = 94.0;

/// Hit rects of the level-up cards, one per choice slot.
pub fn level_up_cards(view: Vec2) -> [Rect; 3] {
    let w = (view.x - 2.0 * CARD_MARGIN).min(CARD_MAX_W);
    let x = Anchor::Center.x(view, w, 0.0);
    std::array::from_fn(|slot| {
        let y = CARDS_Y + slot as f32 * (CARD_H + CARD_GAP);
        Rect::new(x, y, w, CARD_H)
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
        let icon = Some(Icon::of_choice(choice));
        draw_card(cards[slot], &num, icon, name, &lv, &desc, lit);
    }
    draw_text_centered(
        view,
        tr(Key::LevelUpHelp),
        CARDS_Y + 3.0 * (CARD_H + CARD_GAP) + 22.0,
        TextStyle::Body,
        WHITE,
    );
}

/// Card icons are drawn at twice their pixel size, centred in the card's height.
const CARD_ICON_SCALE: f32 = 2.0;
const CARD_ICON_X: f32 = 26.0;
const CARD_TEXT_X: f32 = CARD_ICON_X + ICON_SIZE * CARD_ICON_SCALE + 8.0;

/// A card: `num` in the corner, then the icon, the name with a tag on the right and a
/// description below.
fn draw_card(
    rect: Rect,
    num: &str,
    icon: Option<Icon>,
    name: &str,
    tag: &str,
    desc: &str,
    lit: bool,
) {
    let Rect { x, y, w, h } = rect;
    let border = if lit { TITLE_COLOR } else { GRAY };
    draw_rectangle(x, y, w, h, Color::new(0.08, 0.04, 0.1, 0.95));
    draw_rectangle_lines(x, y, w, h, 2.0, border);
    draw_ui_text(num, x + 10.0, y + 22.0, TextStyle::Body, GRAY);
    if let Some(icon) = icon {
        let size = ICON_SIZE * CARD_ICON_SCALE;
        let iy = (y + (h - size) * 0.5).round();
        draw_icon(icon, x + CARD_ICON_X, iy, CARD_ICON_SCALE);
    }
    draw_ui_text(name, x + CARD_TEXT_X, y + 22.0, TextStyle::Body, WHITE);
    let tag_w = measure_ui_text(tag, TextStyle::Body).width;
    draw_ui_text(tag, x + w - tag_w - 10.0, y + 22.0, TextStyle::Body, GRAY);
    let (first, second) = wrap_two(desc, w - CARD_TEXT_X - 10.0);
    draw_ui_text(first, x + CARD_TEXT_X, y + 44.0, TextStyle::Body, LIGHTGRAY);
    draw_ui_text(
        second,
        x + CARD_TEXT_X,
        y + 62.0,
        TextStyle::Body,
        LIGHTGRAY,
    );
}

/// Splits `text` at a space into two lines when it is wider than `width`: the first
/// takes as many words as fit.
fn wrap_two(text: &str, width: f32) -> (&str, &str) {
    let fits = |t: &str| measure_ui_text(t, TextStyle::Body).width <= width;
    if fits(text) {
        return (text, "");
    }
    let cut = text
        .match_indices(' ')
        .map(|(i, _)| i)
        .take_while(|&i| fits(&text[..i]))
        .last();
    match cut {
        Some(i) => (&text[..i], &text[i + 1..]),
        None => (text, ""),
    }
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
    let (line, icon, name, tag, desc) = match reward {
        Reward::Evolution(i) => {
            let line = trf(Key::OfferingEvolved, &[&tr(ARTS[i].name)]);
            let evo = ARTS[i]
                .evolution
                .as_ref()
                .expect("only Arts with one evolve");
            let (name, desc) = (tr(evo.name), tr(evo.desc));
            let icon = Some(Icon::Evolution(i));
            (
                line,
                icon,
                name.to_string(),
                String::new(),
                desc.to_string(),
            )
        }
        Reward::Union(u) => {
            let def = &UNIONS[u];
            let [a, b] = def.arts.map(|i| tr(ARTS[i].name));
            (
                trf(Key::OfferingUnited, &[&a, &b]),
                Some(Icon::Union(u)),
                tr(def.name).to_string(),
                String::new(),
                tr(def.desc).to_string(),
            )
        }
        // Not taken yet: show the level it would reach and what that gives.
        Reward::Gift(choice) => {
            let (name, tag, desc) = card_text(game, choice);
            (
                tr(Key::OfferingGift).to_string(),
                Some(Icon::of_choice(choice)),
                name.to_string(),
                tag,
                desc,
            )
        }
        Reward::Heal => {
            let hp = trf(Key::OfferingHealDesc, &[&OFFERING_HEAL]);
            (
                tr(Key::OfferingHeal).to_string(),
                None,
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
    draw_card(
        accept,
        "1",
        icon,
        &name,
        &tag,
        &desc,
        focus.is_highlighted(0),
    );
    let gain = trf(
        Key::OfferingSacrificeDesc,
        &[&SACRIFICE_HEAL, &SACRIFICE_XP],
    );
    let lit = focus.is_highlighted(1);
    let sacrifice_name = tr(Key::OfferingSacrifice);
    draw_card(sacrifice, "2", None, sacrifice_name, "", &gain, lit);
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

/// The union whose Unite button sits on Art `art`'s row (an Art without an evolution
/// of its own).
pub fn debug_union(art: usize) -> usize {
    (0..UNIONS.len())
        .find(|&k| UNIONS[k].arts.contains(&art))
        .unwrap_or(0)
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
        let union = u.union_of(i);
        let color = if u.evolved[i] || union.is_some() {
            DBG_GOLD
        } else {
            WHITE
        };
        // The union's first Art carries its name; the second is out of the slots.
        let name = match union {
            Some(k) if UNIONS[k].arts[0] == i => tr(UNIONS[k].name),
            _ => tr(def.name),
        };
        let level = if lv > 0 {
            format!("{lv}/{ART_MAX_LEVEL}")
        } else {
            "-".to_string()
        };
        let dim = lv == 0 && arts_full;
        debug_row(
            rects[i],
            name,
            &level,
            lv > 0,
            dim,
            focus.is_highlighted(i),
            color,
        );
        // An Art with no evolution of its own has a Unite button instead.
        let (label, owned, dim, color) = match &def.evolution {
            Some(_) if u.evolved[i] => ("EVOLVED".to_string(), true, false, DBG_GOLD),
            Some(_) if u.can_evolve(i) => ("EVOLVE".to_string(), true, false, WHITE),
            Some(evo) => (format!("+{}", tr(SINS[evo.sin].name)), false, true, GRAY),
            None => {
                let k = debug_union(i);
                let other = UNIONS[k].arts.into_iter().find(|&a| a != i).unwrap_or(i);
                if u.united[k] {
                    ("UNITED".to_string(), true, false, DBG_GOLD)
                } else if u.can_unite(k) {
                    ("UNITE".to_string(), true, false, WHITE)
                } else {
                    (format!("+{}", tr(ARTS[other].name)), false, true, GRAY)
                }
            }
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
        "Evolve needs Lv 8 and the Sin shown    Unite needs both Arts at Lv 8",
        "F2 / Esc: close",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_frames_cover_the_strip_once() {
        let mut seen = [false; ICON_COUNT];
        let evolving = (0..ARTS.len()).filter(|&i| ARTS[i].evolution.is_some());
        let icons = (0..ARTS.len())
            .map(Icon::Art)
            .chain(evolving.map(Icon::Evolution))
            .chain((0..UNIONS.len()).map(Icon::Union))
            .chain((0..SINS.len()).map(Icon::Sin));
        for icon in icons {
            let f = icon.frame();
            assert!(f < ICON_COUNT && !seen[f], "{icon:?}");
            seen[f] = true;
        }
        assert!(seen.iter().all(|&s| s));
        assert_eq!(Icon::Evolution(HELLFLARE).frame(), ARTS.len());
        assert_eq!(Icon::Sin(0).frame(), ICON_COUNT - SINS.len());
    }
}

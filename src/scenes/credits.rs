//! Credits: a slowly scrolling list of names. The full licence texts (OFL for the
//! fonts, MIT and friends for macroquad and its dependencies) ship in the `licenses/`
//! folder next to the game, with the EULA, not on screen.

use super::Nav;
use crate::assets::{measure_ui_text, TextStyle};
use crate::config::GAME_TITLE;
use crate::hud::Anchor;
use crate::i18n::{self, tr, Key, Lang};
use crate::kit;
use crate::ui::UiInput;
use macroquad::prelude::*;

const AUTHOR: &str = "Adrià Bonnin Catalán";
/// The game is proprietary (see `LICENSE`); `©` is not pre-rasterised, hence `(c)`.
const COPYRIGHT_YEAR: u32 = 2026;
const TOP_Y: f32 = 70.0;
/// Seconds before the list starts scrolling on its own.
const SCROLL_DELAY: f32 = 2.0;
const SCROLL_SPEED: f32 = 24.0;
/// Scroll per wheel notch or arrow press.
const SCROLL_STEP: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Title,
    /// Room for the rule under the title (it hangs 6..17 px below the title's baseline).
    Rule,
    Heading,
    Name,
    Gap,
}

impl Kind {
    fn height(self) -> f32 {
        match self {
            Kind::Title => 40.0,
            Kind::Rule => 18.0,
            Kind::Heading => 28.0,
            Kind::Name => 17.0,
            Kind::Gap => 12.0,
        }
    }
}

pub struct Credits {
    lines: Vec<(Kind, String)>,
    lang: Lang,
    scroll: f32,
    time: f32,
}

impl Credits {
    pub fn new() -> Self {
        Self {
            lines: build_lines(),
            lang: i18n::lang(),
            scroll: 0.0,
            time: 0.0,
        }
    }

    fn content_height(&self) -> f32 {
        self.lines.iter().map(|(k, _)| k.height()).sum()
    }

    pub fn update(&mut self, input: &UiInput, view: Vec2) -> Option<Nav> {
        if input.back || input.confirm || input.clicked {
            return Some(Nav::Title);
        }
        if self.lang != i18n::lang() {
            self.lines = build_lines();
            self.lang = i18n::lang();
        }
        self.time += get_frame_time();
        let manual = input.wheel != 0.0 || input.up || input.down;
        if manual {
            // Taking control stops the automatic scroll for a while.
            self.time = 0.0;
            let up = input.wheel > 0.0 || input.up;
            self.scroll += if up { -SCROLL_STEP } else { SCROLL_STEP };
        } else if self.time > SCROLL_DELAY {
            self.scroll += SCROLL_SPEED * get_frame_time();
        }
        let max = (TOP_Y + self.content_height() - view.y * 0.5).max(0.0);
        self.scroll = self.scroll.clamp(0.0, max);
        None
    }

    pub fn draw(&self, view: Vec2) {
        kit::dim(view, 1.0);
        let p = kit::pal();
        let cx = view.x * 0.5;
        let mut y = TOP_Y - self.scroll.round();
        for (kind, text) in &self.lines {
            y += kind.height();
            if y < -10.0 || y > view.y + 40.0 {
                continue;
            }
            match kind {
                Kind::Title => {
                    kit::title(text, cx, y, 1.0);
                }
                Kind::Heading => kit::text_centered(text, cx, y, TextStyle::Name, p.gold),
                Kind::Name => kit::text_centered(text, cx, y, TextStyle::Desc, p.label),
                Kind::Rule | Kind::Gap => {}
            }
        }
        let help = tr(Key::CreditsHelp);
        let w = measure_ui_text(help, TextStyle::Desc).width;
        let bar = view.y - 20.0;
        draw_rectangle(0.0, bar, view.x, 20.0, p.shadow);
        kit::text(help, Anchor::Right.x(view, w, 8.0), bar + 15.0, TextStyle::Desc, p.dim);
    }
}

fn build_lines() -> Vec<(Kind, String)> {
    let mut lines = vec![
        (Kind::Title, GAME_TITLE.to_owned()),
        (Kind::Rule, String::new()),
        (Kind::Name, format!("(c) {COPYRIGHT_YEAR} {AUTHOR}")),
        (Kind::Name, tr(Key::CreditsRights).to_owned()),
        (Kind::Gap, String::new()),
    ];
    let mut section = |heading: Key, names: &[&str]| {
        lines.push((Kind::Heading, tr(heading).to_owned()));
        lines.extend(names.iter().map(|n| (Kind::Name, (*n).to_owned())));
        lines.push((Kind::Gap, String::new()));
    };
    section(Key::CreditsDesign, &[AUTHOR]);
    section(Key::CreditsArt, &[AUTHOR]);
    section(Key::CreditsMusic, &[AUTHOR, tr(Key::CreditsSynth)]);
    let own_fonts = format!("Veyra Blackletter, Veyra Display - {AUTHOR}");
    section(
        Key::CreditsFonts,
        &[
            &own_fonts,
            "Jacquard 24 - Sarah Cadigan-Fried",
            "SIL Open Font License 1.1",
        ],
    );
    section(
        Key::CreditsEngine,
        &["macroquad - Fedor Logachev", "MIT / Apache-2.0"],
    );
    section(Key::CreditsThanks, &[tr(Key::CreditsThanksYou)]);
    section(
        Key::CreditsLicences,
        &[tr(Key::CreditsLicenceFiles), tr(Key::CreditsEula)],
    );
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::EXTRA_GLYPHS;

    #[test]
    fn author_is_in_the_font_cache() {
        for c in AUTHOR.chars() {
            assert!(
                (' '..='~').contains(&c) || EXTRA_GLYPHS.contains(c),
                "{c:?}"
            );
        }
    }
}

//! Credits: a slowly scrolling list of names, then the licence texts of the
//! embedded fonts (OFL) and of macroquad (MIT) in the small face.

use super::Nav;
use crate::assets::{draw_ui_text, measure_ui_text, TextStyle};
use crate::config::GAME_TITLE;
use crate::hud::{self, Anchor};
use crate::i18n::{self, tr, Key, Lang};
use crate::ui::{self, UiInput};
use macroquad::prelude::*;

const AUTHOR: &str = "AdriaBC06";
const OFL_JACQUARD: &str = include_str!("../../assets/fonts/OFL-Jacquard24.txt");
const OFL_JERSEY: &str = include_str!("../../assets/fonts/OFL-Jersey15.txt");
const OFL_PRESS_START: &str = include_str!("../../assets/fonts/OFL-PressStart2P.txt");
const MACROQUAD_MIT: &str = include_str!("../../assets/licenses/macroquad-LICENSE-MIT.txt");

/// Width the licence texts are wrapped to; fits the narrowest view.
const WRAP_W: f32 = 440.0;
const TOP_Y: f32 = 70.0;
/// Seconds before the list starts scrolling on its own.
const SCROLL_DELAY: f32 = 2.0;
const SCROLL_SPEED: f32 = 24.0;
/// Scroll per wheel notch or arrow press.
const SCROLL_STEP: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Title,
    Heading,
    Name,
    Small,
    Gap,
}

impl Kind {
    fn height(self) -> f32 {
        match self {
            Kind::Title => 40.0,
            Kind::Heading | Kind::Name => 20.0,
            Kind::Small => 11.0,
            Kind::Gap => 14.0,
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
        hud::draw_dim(view);
        draw_rectangle(0.0, 0.0, view.x, view.y, Color::new(0.0, 0.0, 0.0, 0.3));
        let x = Anchor::Center.x(view, WRAP_W, 0.0);
        let mut y = TOP_Y - self.scroll.round();
        for (kind, text) in &self.lines {
            y += kind.height();
            if y < -10.0 || y > view.y + 40.0 {
                continue;
            }
            let (style, color) = match kind {
                Kind::Title => (TextStyle::Title, hud::TITLE_COLOR),
                Kind::Heading => (TextStyle::Body, ui::ACCENT),
                Kind::Name => (TextStyle::Body, WHITE),
                Kind::Small => (TextStyle::Small, LIGHTGRAY),
                Kind::Gap => continue,
            };
            if *kind == Kind::Small {
                draw_ui_text(text, x, y, style, color);
            } else {
                hud::draw_text_centered(view, text, y, style, color);
            }
        }
        let help = tr(Key::CreditsHelp);
        let w = measure_ui_text(help, TextStyle::Small).width;
        let bar = view.y - 16.0;
        draw_rectangle(0.0, bar, view.x, 16.0, BLACK);
        draw_ui_text(
            help,
            Anchor::Right.x(view, w, 6.0),
            bar + 12.0,
            TextStyle::Small,
            GRAY,
        );
    }
}

fn build_lines() -> Vec<(Kind, String)> {
    let mut lines = vec![
        (Kind::Title, GAME_TITLE.to_owned()),
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
    section(
        Key::CreditsFonts,
        &[
            "Jacquard 24, Jersey 15 - Sarah Cadigan-Fried",
            "Press Start 2P - Cody \"CodeMan38\" Boisclair",
            "SIL Open Font License 1.1",
        ],
    );
    section(
        Key::CreditsEngine,
        &["macroquad - Fedor Logachev", "MIT / Apache-2.0"],
    );
    section(Key::CreditsThanks, &[tr(Key::CreditsThanksYou)]);
    lines.push((Kind::Heading, tr(Key::CreditsLicences).to_owned()));
    lines.push((Kind::Gap, String::new()));

    let measure = |s: &str| measure_ui_text(s, TextStyle::Small).width;
    let mut licence = |text: &str| {
        for paragraph in paragraphs(text) {
            for line in wrap(&paragraph, WRAP_W, measure) {
                lines.push((Kind::Small, line));
            }
            lines.push((Kind::Gap, String::new()));
        }
    };
    // The three OFL files differ only in their first (copyright) line.
    for ofl in [OFL_JACQUARD, OFL_JERSEY, OFL_PRESS_START] {
        licence(ofl.lines().next().unwrap_or_default());
    }
    licence(OFL_JERSEY.split_once('\n').map_or("", |(_, rest)| rest));
    licence(MACROQUAD_MIT);
    lines
}

/// Paragraphs of a plain-text file: blank lines separate them, single breaks join.
fn paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|p| !p.is_empty())
        .collect()
}

/// Greedy word wrap to `max_w`, with `measure` giving a string's width.
fn wrap(text: &str, max_w: f32, measure: impl Fn(&str) -> f32) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && measure(&candidate) > max_w {
            out.push(std::mem::replace(&mut line, word.to_owned()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_keeps_lines_within_width() {
        let measure = |s: &str| s.len() as f32;
        let lines = wrap("aa bbb cc dddddddd e", 6.0, measure);
        assert_eq!(lines, ["aa bbb", "cc", "dddddddd", "e"]);
        assert!(wrap("", 6.0, measure).is_empty());
    }

    #[test]
    fn paragraphs_join_single_breaks() {
        let p = paragraphs("one\ntwo\n\n\nthree  four\n");
        assert_eq!(p, ["one two", "three four"]);
    }

    #[test]
    fn licence_texts_are_ascii() {
        for text in [OFL_JACQUARD, OFL_JERSEY, OFL_PRESS_START, MACROQUAD_MIT] {
            assert!(text.is_ascii());
        }
    }
}

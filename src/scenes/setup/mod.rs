//! First-time setup (V19g), shown on the very first launch (no settings file yet): language,
//! blood, display and volume, one page each, all reachable later in Settings. Picking an option
//! (a click or Enter, never a hover) burns the others away, then the next page opens. While the
//! Blood page is up its own stone soaks or dries to the highlighted mode. Skip keeps the defaults.

mod page;

use super::Nav;
use crate::config::{SETUP_BLOOD_PREVIEW, SETUP_BLOOD_RATE};
use crate::i18n::{self, Lang};
use crate::save::{BloodMode, Settings};
use crate::screen::{ScaleMode, Screen};
use crate::ui::{Action, Focus, Overlay, Slider, UiInput};
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Step {
    Language,
    Blood,
    Display,
    Volume,
}

const STEPS: [Step; 4] = [Step::Language, Step::Blood, Step::Display, Step::Volume];

/// The Display page: two rows of two cards, the scale mode and window or fullscreen, each
/// row picked on its own and confirmed together.
const SCALES: [ScaleMode; 2] = [ScaleMode::Fill, ScaleMode::PixelPerfect];
const FULLSCREEN: [bool; 2] = [false, true];

/// After a pick: the other options burn, the picked one is held a moment, the page fades out.
const BURN: f32 = 0.75;
const HOLD: f32 = 0.3;
const FADE: f32 = 0.25;
const VOLUME_STEP: f32 = 0.1;

pub struct SetupScene {
    step: usize,
    focus: Focus,
    overlay: Overlay,
    /// The option picked on this page and the seconds since.
    picked: Option<(usize, f32)>,
    /// The Display page's choice in each row (scale, fullscreen), as a card column.
    display: [usize; 2],
    /// Blood shown on the pages, gliding to the highlighted (then the chosen) mode's preview.
    blood: f32,
}

impl SetupScene {
    pub fn new(settings: &Settings) -> Self {
        let mut s = Self {
            step: 0,
            focus: Focus::default(),
            overlay: Overlay::default(),
            picked: None,
            display: [0, 0],
            blood: 0.0,
        };
        s.open(0, settings, None);
        s
    }

    fn step(&self) -> Step {
        STEPS[self.step]
    }

    /// Opens page `step` with the focus on the current value.
    fn open(&mut self, step: usize, settings: &Settings, screen: Option<&Screen>) {
        self.step = step;
        self.picked = None;
        self.overlay = Overlay::default();
        self.focus = Focus::default();
        self.focus.index = match self.step() {
            Step::Language => Lang::ALL
                .iter()
                .position(|&l| l == i18n::lang())
                .unwrap_or(0),
            Step::Blood => BloodMode::ALL
                .iter()
                .position(|&m| m == settings.blood)
                .unwrap_or(0),
            Step::Display => {
                let (mode, full) = screen.map_or((settings.scale_mode, settings.fullscreen), |s| {
                    (s.mode, s.fullscreen())
                });
                self.display = [
                    SCALES.iter().position(|&m| m == mode).unwrap_or(0),
                    FULLSCREEN.iter().position(|&f| f == full).unwrap_or(0),
                ];
                self.display[0]
            }
            Step::Volume => 0,
        };
    }

    /// Options on this page (the Volume page has its slider instead).
    fn options(&self) -> usize {
        match self.step() {
            Step::Language => Lang::ALL.len(),
            Step::Blood => BloodMode::ALL.len(),
            Step::Display => SCALES.len() + FULLSCREEN.len(),
            Step::Volume => 1,
        }
    }

    /// Whether option `i` stays when the page is confirmed (the rest burn).
    fn kept(&self, i: usize) -> bool {
        match self.step() {
            Step::Display => self.display[i / 2] == i % 2,
            _ => self.picked.is_some_and(|(p, _)| p == i),
        }
    }

    /// Focusable rects: the options (or the volume slider), then Skip / Confirm / Begin.
    fn items(&self, view: Vec2) -> Vec<Rect> {
        let mut items = page::option_rects(self.step(), view);
        items.push(page::bottom_button(view));
        items
    }

    /// The blood level the pages should show now.
    fn blood_target(&self, settings: &Settings) -> f32 {
        let preview = |m: BloodMode| {
            let i = BloodMode::ALL.iter().position(|&x| x == m).unwrap_or(0);
            SETUP_BLOOD_PREVIEW[i]
        };
        match self.step() {
            Step::Language => 0.0,
            Step::Blood => {
                let i = self
                    .picked
                    .map(|(i, _)| i)
                    .or(self.focus.highlight().filter(|&i| i < self.options()))
                    .unwrap_or(self.focus.index.min(self.options() - 1));
                SETUP_BLOOD_PREVIEW[i]
            }
            _ => preview(settings.blood),
        }
    }

    pub fn update(
        &mut self,
        input: &UiInput,
        view: Vec2,
        settings: &mut Settings,
        screen: &mut Screen,
    ) -> Option<Nav> {
        let dt = get_frame_time();
        let target = self.blood_target(settings);
        self.blood += (target - self.blood).clamp(-SETUP_BLOOD_RATE * dt, SETUP_BLOOD_RATE * dt);

        if let Some((_, since)) = &mut self.picked {
            *since += dt;
            self.overlay.t += dt;
            if *since < BURN + HOLD + FADE {
                return None;
            }
            self.open(self.step + 1, settings, Some(screen));
            return None;
        }

        let items = self.items(view);
        let button = items.len() - 1;
        let n = self.options();
        let action = self.focus.update(input, &items);
        self.overlay.update(input, &self.focus, &items, dt);
        match action {
            Action::Back if self.step > 0 => {
                self.open(self.step - 1, settings, Some(screen));
            }
            // the Display page's Confirm burns the unpicked card of each row
            Action::Confirm(i) if i == button && self.step() == Step::Display => {
                self.picked = Some((i, 0.0));
            }
            // Skip, or Begin on the last page
            Action::Confirm(i) if i == button => return Some(Nav::Title),
            _ if self.step() == Step::Volume => {
                let mut slider = Slider {
                    rect: items[0],
                    label: "",
                    value: settings.master,
                    step: VOLUME_STEP,
                };
                slider.handle(0, action, input);
                settings.master = (slider.value * 100.0).round() / 100.0;
            }
            // a Display card picks its column in its row, and the screen follows at once
            Action::Confirm(i) if i < n && self.step() == Step::Display => {
                self.display[i / 2] = i % 2;
                let (mode, full) = (SCALES[self.display[0]], FULLSCREEN[self.display[1]]);
                if screen.mode != mode {
                    screen.toggle_mode();
                }
                if screen.fullscreen() != full {
                    screen.toggle_fullscreen();
                }
            }
            Action::Adjust(i, _) if i < n && self.step() == Step::Display => {
                self.focus.index = i ^ 1;
            }
            // left/right walk the options in a row
            Action::Adjust(i, d) if i < n => {
                self.focus.index = (i as i32 + d as i32).rem_euclid(n as i32) as usize;
            }
            Action::Confirm(i) if i < n => {
                match self.step() {
                    Step::Language => i18n::set_lang(Lang::ALL[i]),
                    Step::Blood => settings.blood = BloodMode::ALL[i],
                    _ => {}
                }
                self.picked = Some((i, 0.0));
            }
            _ => {}
        }
        None
    }

    pub fn draw(&self, view: Vec2, settings: &Settings, screen: &Screen) {
        page::draw(self, view, settings, screen);
    }
}

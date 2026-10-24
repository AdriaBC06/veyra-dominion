//! Settings screen: tabs for audio, video, gameplay, a controls sheet and data
//! (lifetime stats and a two-step progress reset). Changes apply at once; the
//! caller writes `settings.txt` when the screen is left.

use super::Nav;
use crate::assets::{measure_ui_text, TextStyle};
use crate::hud::{self, Anchor};
use crate::i18n::{self, tr, trf, Key};
use crate::kit;
use crate::save::{BloodMode, SaveData, Settings};
use crate::screen::{ScaleMode, Screen};
use crate::ui::{self, Action, Button, Choice, Focus, Overlay, Slider, Tabs, Toggle, UiInput};
use macroquad::prelude::*;

/// Width of the rows; the stone panel adds `PANEL_PAD` on each side.
const PANEL_W: f32 = 440.0;
const PANEL_PAD: f32 = 30.0;
/// The panel runs from `PANEL_Y` (room above for its skull) to `PANEL_BOTTOM` above the
/// view's bottom; the Back button sits `BACK_PAD` above the panel's bottom edge.
const PANEL_Y: f32 = 14.0;
const PANEL_BOTTOM: f32 = 8.0;
const BACK_PAD: f32 = 30.0;
const TITLE_Y: f32 = PANEL_Y + 50.0;
const TABS_Y: f32 = PANEL_Y + 77.0;
const TAB_H: f32 = 28.0;
const ROWS_Y: f32 = TABS_Y + TAB_H + 10.0;
/// Rows are as tall as buttons: room for a label's capitals and descenders in the frame.
const ROW_H: f32 = ui::BUTTON_H;
const ROW_GAP: f32 = 6.0;
/// Lines of the controls sheet and the stats (Display).
const LINE_H: f32 = 17.0;
const VOLUME_STEP: f32 = 0.1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Row {
    Master,
    Music,
    Sfx,
    Fullscreen,
    Scale,
    Language,
    Blood,
    MouseMove,
    Reset,
}

const TABS: [(Key, &[Row]); 5] = [
    (Key::TabAudio, &[Row::Master, Row::Music, Row::Sfx]),
    (Key::TabVideo, &[Row::Fullscreen, Row::Scale]),
    (Key::TabGame, &[Row::Language, Row::Blood, Row::MouseMove]),
    (Key::TabControls, &[]),
    (Key::TabData, &[Row::Reset]),
];

/// Controls sheet: action, then its keys as a translated key or a literal.
const CONTROLS: [(Key, Option<Key>, &str); 8] = [
    (Key::CtlMove, Some(Key::CtlMoveKeys), ""),
    (Key::CtlMouse, Some(Key::CtlMouseKeys), ""),
    (Key::CtlPause, None, "Esc"),
    (Key::CtlPick, Some(Key::CtlPickKeys), ""),
    (Key::CtlConfirm, None, "Enter / Esc"),
    (Key::SetFullscreen, None, "F11"),
    (Key::SetScale, None, "F10"),
    (Key::SetLanguage, None, "F9"),
];

pub struct SettingsScene {
    tab: usize,
    focus: Focus,
    /// Opened from the pause menu, so going back resumes the run.
    from_run: bool,
    /// The reset button was pressed once and waits for the second press.
    reset_armed: bool,
    reset_done: bool,
    overlay: Overlay,
}

impl SettingsScene {
    pub fn new(from_run: bool) -> Self {
        Self {
            tab: 0,
            focus: Focus::default(),
            from_run,
            reset_armed: false,
            reset_done: false,
            overlay: Overlay::default(),
        }
    }

    /// Opened from the pause menu (the run stays behind it).
    pub fn in_run(&self) -> bool {
        self.from_run
    }

    fn rows(&self) -> &'static [Row] {
        TABS[self.tab].1
    }

    /// Focusable rects: the tab row, the current tab's rows, then Back.
    fn items(&self, view: Vec2) -> Vec<Rect> {
        let x = Anchor::Center.x(view, PANEL_W, 0.0);
        let mut items = vec![Rect::new(x, TABS_Y, PANEL_W, TAB_H)];
        items.extend((0..self.rows().len()).map(|k| {
            let y = ROWS_Y + k as f32 * (ROW_H + ROW_GAP);
            Rect::new(x, y, PANEL_W, ROW_H)
        }));
        let back_y = view.y - PANEL_BOTTOM - BACK_PAD - ui::BUTTON_H;
        let back = ui::button_column(view, back_y, 1)[0];
        items.push(back);
        items
    }

    pub fn update(
        &mut self,
        input: &UiInput,
        view: Vec2,
        settings: &mut Settings,
        screen: &mut Screen,
        save: &mut SaveData,
    ) -> Option<Nav> {
        let items = self.items(view);
        let back = items.len() - 1;
        let action = self.focus.update(input, &items);
        self.overlay
            .update(input, &self.focus, &items, get_frame_time());
        if action == Action::Back && self.reset_armed {
            self.reset_armed = false;
            return None;
        }
        if action == Action::Back || action == Action::Confirm(back) {
            return Some(if self.from_run {
                Nav::ResumeRun
            } else {
                Nav::Title
            });
        }

        let labels = TABS.map(|(k, _)| tr(k));
        let mut tabs = Tabs {
            rect: items[0],
            labels: &labels,
            active: self.tab,
        };
        if tabs.handle(0, action, input) {
            self.tab = tabs.active;
            self.reset_armed = false;
            self.reset_done = false;
            // `items` still holds the old tab's rows; lay out the new ones next frame.
            return None;
        }

        for (k, &row) in self.rows().iter().enumerate() {
            let (i, rect) = (k + 1, items[k + 1]);
            let volume = match row {
                Row::Master => Some(&mut settings.master),
                Row::Music => Some(&mut settings.music),
                Row::Sfx => Some(&mut settings.sfx),
                _ => None,
            };
            if let Some(value) = volume {
                let mut slider = Slider {
                    rect,
                    label: "",
                    value: *value,
                    step: VOLUME_STEP,
                };
                slider.handle(i, action, input);
                // Keep keyboard steps on round tenths.
                *value = (slider.value * 100.0).round() / 100.0;
                continue;
            }
            match row {
                Row::Fullscreen => {
                    let mut t = toggle(rect, screen.fullscreen());
                    if t.handle(i, action) {
                        screen.toggle_fullscreen();
                    }
                }
                Row::Scale if Choice::handle(i, action) != 0 => screen.toggle_mode(),
                Row::Language if Choice::handle(i, action) != 0 => {
                    i18n::set_lang(i18n::lang().next());
                }
                Row::Blood => {
                    let step = Choice::handle(i, action);
                    if step != 0 {
                        settings.blood = settings.blood.step(step);
                    }
                }
                Row::MouseMove => {
                    let mut t = toggle(rect, settings.mouse_move);
                    if t.handle(i, action) {
                        settings.mouse_move = t.on;
                    }
                }
                Row::Reset => {
                    if action == Action::Confirm(i) {
                        if self.reset_armed {
                            *save = SaveData::default();
                            save.store();
                        }
                        self.reset_done = self.reset_armed;
                        self.reset_armed = !self.reset_armed;
                    } else if self.focus.index != i {
                        self.reset_armed = false;
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn draw(&self, view: Vec2, settings: &Settings, screen: &Screen, save: &SaveData) {
        kit::dim(view, 1.0);
        let items = self.items(view);
        let panel = Rect::new(
            items[0].x - PANEL_PAD,
            PANEL_Y,
            PANEL_W + 2.0 * PANEL_PAD,
            view.y - PANEL_Y - PANEL_BOTTOM,
        );
        kit::panel(panel, false, 1.0, kit::seed_of("settings"));
        kit::title(tr(Key::MenuSettings), view.x * 0.5, TITLE_Y, 1.0);
        let lit = |i: usize| self.focus.is_highlighted(i);
        let labels = TABS.map(|(k, _)| tr(k));
        Tabs {
            rect: items[0],
            labels: &labels,
            active: self.tab,
        }
        .draw(lit(0));

        for (k, &row) in self.rows().iter().enumerate() {
            let (i, rect) = (k + 1, items[k + 1]);
            let slider = |label, value| Slider {
                rect,
                label: tr(label),
                value,
                step: VOLUME_STEP,
            };
            match row {
                Row::Master => slider(Key::SetMaster, settings.master).draw(lit(i)),
                Row::Music => slider(Key::SetMusic, settings.music).draw(lit(i)),
                Row::Sfx => slider(Key::SetSfx, settings.sfx).draw(lit(i)),
                Row::Fullscreen => Toggle {
                    label: tr(Key::SetFullscreen),
                    ..toggle(rect, screen.fullscreen())
                }
                .draw(lit(i)),
                Row::Scale => Choice {
                    rect,
                    label: tr(Key::SetScale),
                    value: tr(match screen.mode {
                        ScaleMode::PixelPerfect => Key::ScalePixel,
                        ScaleMode::Fill => Key::ScaleFill,
                    }),
                }
                .draw(lit(i)),
                Row::Language => Choice {
                    rect,
                    label: tr(Key::SetLanguage),
                    value: i18n::lang().name(),
                }
                .draw(lit(i)),
                Row::Blood => Choice {
                    rect,
                    label: tr(Key::SetBlood),
                    value: tr(match settings.blood {
                        BloodMode::Full => Key::BloodFull,
                        BloodMode::Reduced => Key::BloodReduced,
                        BloodMode::Off => Key::BloodOff,
                    }),
                }
                .draw(lit(i)),
                Row::MouseMove => Toggle {
                    label: tr(Key::SetMouseMove),
                    ..toggle(rect, settings.mouse_move)
                }
                .draw(lit(i)),
                Row::Reset => {
                    let key = if self.reset_armed {
                        Key::ResetConfirm
                    } else {
                        Key::ResetProgress
                    };
                    Button {
                        rect,
                        label: tr(key),
                    }
                    .draw(lit(i));
                }
            }
        }

        let x = items[0].x;
        let p = kit::pal();
        match TABS[self.tab].0 {
            Key::TabControls => {
                for (n, (action, keys, literal)) in CONTROLS.iter().enumerate() {
                    let keys = keys.map_or(*literal, tr);
                    let y = ROWS_Y + 12.0 + n as f32 * LINE_H;
                    kit::text(tr(*action), x + 12.0, y, TextStyle::Desc, p.label);
                    let w = measure_ui_text(keys, TextStyle::Desc).width;
                    kit::text(keys, x + PANEL_W - w - 12.0, y, TextStyle::Desc, p.gold);
                }
            }
            Key::TabData => {
                let lines = [
                    trf(Key::DataRuns, &[&save.runs]),
                    trf(Key::DataKills, &[&save.total_kills]),
                    trf(Key::DataBest, &[&hud::format_time(save.best_time)]),
                ];
                let mut y = ROWS_Y + ROW_H + 22.0;
                for line in &lines {
                    kit::text(line, x + 12.0, y, TextStyle::Desc, p.desc);
                    y += LINE_H;
                }
                if self.reset_done {
                    let done = tr(Key::ResetDone);
                    kit::text(done, x + 12.0, y + 6.0, TextStyle::Desc, p.ruby);
                }
            }
            _ => {}
        }

        let back = items[items.len() - 1];
        Button {
            rect: back,
            label: tr(Key::MenuBack),
        }
        .draw(lit(items.len() - 1));
        if let Some(i) = self.focus.highlight().filter(|&i| i < items.len()) {
            kit::cursors(self.overlay.cursor_rect(items[i]), self.overlay.t, 1.0);
            kit::sparks(items[i], self.overlay.t, 1.0);
        }
    }
}

fn toggle(rect: Rect, on: bool) -> Toggle<'static> {
    Toggle {
        rect,
        label: "",
        on,
    }
}

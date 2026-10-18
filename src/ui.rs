//! Shared UI layer: one focus model driven by mouse (hover selects, click confirms,
//! wheel scrolls) and keyboard (arrows/WASD + Enter/Esc). Logic is separate from
//! drawing so it can be tested headless with hand-made `UiInput`s.

use crate::assets::{draw_ui_text, measure_ui_text, TextStyle};
use crate::audio::{self, Sfx};
use crate::i18n::{tr, Key};
use crate::screen::Layout;
use macroquad::prelude::*;

pub const ACCENT: Color = Color::new(0.88, 0.19, 0.29, 1.0);
const PANEL: Color = Color::new(0.08, 0.04, 0.1, 0.95);
pub const BUTTON_W: f32 = 220.0;
pub const BUTTON_H: f32 = 28.0;
pub const BUTTON_GAP: f32 = 6.0;

/// One frame of UI input, already in logical frame coordinates.
#[derive(Clone, Copy, Default, Debug)]
pub struct UiInput {
    pub mouse: Vec2,
    pub mouse_down: bool,
    pub clicked: bool,
    /// Wheel direction: positive scrolls up.
    pub wheel: f32,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub confirm: bool,
    pub back: bool,
}

impl UiInput {
    pub fn read(layout: Layout) -> Self {
        let pressed = |keys: &[KeyCode]| keys.iter().any(|&k| is_key_pressed(k));
        let wheel = mouse_wheel().1;
        Self {
            mouse: layout.to_logical(mouse_position().into()),
            mouse_down: is_mouse_button_down(MouseButton::Left),
            clicked: is_mouse_button_pressed(MouseButton::Left),
            // `signum` of 0.0 is 1.0, so a still wheel must be checked first.
            wheel: if wheel == 0.0 { 0.0 } else { wheel.signum() },
            up: pressed(&[KeyCode::Up, KeyCode::W]),
            down: pressed(&[KeyCode::Down, KeyCode::S]),
            left: pressed(&[KeyCode::Left, KeyCode::A]),
            right: pressed(&[KeyCode::Right, KeyCode::D]),
            confirm: pressed(&[KeyCode::Enter, KeyCode::KpEnter, KeyCode::Space]),
            back: pressed(&[KeyCode::Escape]),
        }
    }

    fn any_key(&self) -> bool {
        self.up || self.down || self.left || self.right || self.confirm || self.back
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Device {
    Keyboard,
    Mouse,
}

/// What the focused item should do this frame.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    None,
    Confirm(usize),
    /// Left (-1) or right (+1) on the focused item.
    Adjust(usize, f32),
    /// Mouse held over an item; `f32` is the horizontal position inside it (0..1).
    Pointer(usize, f32),
    Back,
}

/// Single focus model shared by every screen: one focused item among `items` rects.
#[derive(Clone, Copy, Debug)]
pub struct Focus {
    pub index: usize,
    pub device: Device,
    /// Item under the mouse, if any.
    pub hovered: Option<usize>,
    last_mouse: Option<Vec2>,
}

impl Default for Focus {
    fn default() -> Self {
        Self {
            index: 0,
            device: Device::Keyboard,
            hovered: None,
            last_mouse: None,
        }
    }
}

impl Focus {
    /// Moves the focus and reports the action for this frame, cueing the UI
    /// sound for a focus move, confirm or back.
    pub fn update(&mut self, input: &UiInput, items: &[Rect]) -> Action {
        let before = self.index.min(items.len().saturating_sub(1));
        let action = self.step(input, items);
        let sound = match action {
            Action::Confirm(_) => Some(Sfx::UiConfirm),
            Action::Back => Some(Sfx::UiBack),
            Action::Adjust(..) => Some(Sfx::UiMove),
            _ if self.index != before => Some(Sfx::UiMove),
            _ => None,
        };
        if let Some(sfx) = sound {
            audio::cue(sfx);
        }
        action
    }

    fn step(&mut self, input: &UiInput, items: &[Rect]) -> Action {
        let n = items.len();
        if n == 0 {
            return if input.back {
                Action::Back
            } else {
                Action::None
            };
        }
        self.index = self.index.min(n - 1);
        self.hovered = items.iter().position(|r| r.contains(input.mouse));
        let moved = self.last_mouse.is_some_and(|m| m != input.mouse);
        self.last_mouse = Some(input.mouse);
        if moved || input.clicked {
            self.device = Device::Mouse;
            if let Some(i) = self.hovered {
                self.index = i;
            }
        }
        if input.any_key() {
            self.device = Device::Keyboard;
        }

        if input.up {
            self.index = (self.index + n - 1) % n;
        }
        if input.down {
            self.index = (self.index + 1) % n;
        }
        if input.back {
            return Action::Back;
        }
        if input.confirm {
            return Action::Confirm(self.index);
        }
        if input.left || input.right {
            return Action::Adjust(self.index, if input.left { -1.0 } else { 1.0 });
        }
        if let Some(i) = self.hovered {
            if input.clicked {
                return Action::Confirm(i);
            }
            if input.mouse_down {
                let r = items[i];
                return Action::Pointer(i, ((input.mouse.x - r.x) / r.w).clamp(0.0, 1.0));
            }
        }
        Action::None
    }

    /// Item to highlight: the focused one with the keyboard, the hovered one with the mouse.
    pub fn highlight(&self) -> Option<usize> {
        match self.device {
            Device::Keyboard => Some(self.index),
            Device::Mouse => self.hovered,
        }
    }

    pub fn is_highlighted(&self, i: usize) -> bool {
        self.highlight() == Some(i)
    }
}

impl Action {
    /// Horizontal pointer position (0..1) on item `i`: a click is both a confirm and the
    /// start of a drag, so click-or-drag widgets read both.
    pub fn pointer(self, i: usize, input: &UiInput, rect: Rect) -> Option<f32> {
        match self {
            Action::Pointer(j, f) if j == i => Some(f),
            Action::Confirm(j) if j == i && input.clicked => {
                Some(((input.mouse.x - rect.x) / rect.w).clamp(0.0, 1.0))
            }
            _ => None,
        }
    }
}

fn draw_frame(rect: Rect, highlighted: bool) {
    let border = if highlighted { ACCENT } else { GRAY };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, border);
}

/// Label baseline for a one-line widget of `rect`.
fn baseline(rect: Rect) -> f32 {
    (rect.y + rect.h * 0.5 + 7.0).round()
}

pub struct Button<'a> {
    pub rect: Rect,
    pub label: &'a str,
}

impl Button<'_> {
    pub fn draw(&self, highlighted: bool) {
        draw_frame(self.rect, highlighted);
        let w = measure_ui_text(self.label, TextStyle::Body).width;
        let x = (self.rect.x + (self.rect.w - w) * 0.5).round();
        draw_ui_text(self.label, x, baseline(self.rect), TextStyle::Body, WHITE);
    }
}

/// A centred column of `n` buttons starting at `top`.
pub fn button_column(view: Vec2, top: f32, n: usize) -> Vec<Rect> {
    let x = ((view.x - BUTTON_W) * 0.5).round();
    (0..n)
        .map(|i| {
            Rect::new(
                x,
                top + i as f32 * (BUTTON_H + BUTTON_GAP),
                BUTTON_W,
                BUTTON_H,
            )
        })
        .collect()
}

pub fn draw_buttons(rects: &[Rect], labels: &[&str], focus: &Focus) {
    for (i, (&rect, label)) in rects.iter().zip(labels).enumerate() {
        Button { rect, label }.draw(focus.is_highlighted(i));
    }
}

/// Label on the left, value on the right.
fn draw_labeled(rect: Rect, label: &str, value: &str, color: Color, highlighted: bool) {
    draw_frame(rect, highlighted);
    let y = baseline(rect);
    draw_ui_text(label, rect.x + 10.0, y, TextStyle::Body, WHITE);
    let w = measure_ui_text(value, TextStyle::Body).width;
    draw_ui_text(value, rect.right() - w - 10.0, y, TextStyle::Body, color);
}

pub struct Toggle<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub on: bool,
}

impl Toggle<'_> {
    /// Flips on confirm/click or left/right. Returns true when the value changed.
    pub fn handle(&mut self, i: usize, action: Action) -> bool {
        let hit = matches!(action, Action::Confirm(j) | Action::Adjust(j, _) if j == i);
        if hit {
            self.on = !self.on;
        }
        hit
    }

    pub fn draw(&self, highlighted: bool) {
        let value = tr(if self.on { Key::On } else { Key::Off });
        let color = if self.on { ACCENT } else { LIGHTGRAY };
        draw_labeled(self.rect, self.label, value, color, highlighted);
    }
}

/// A label with a value picked from a list: confirm/click or right steps forward, left back.
pub struct Choice<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub value: &'a str,
}

impl Choice<'_> {
    /// Step to apply to the value this frame (0, -1 or +1).
    pub fn handle(i: usize, action: Action) -> i32 {
        match action {
            Action::Confirm(j) if j == i => 1,
            Action::Adjust(j, d) if j == i => d as i32,
            _ => 0,
        }
    }

    pub fn draw(&self, highlighted: bool) {
        draw_labeled(self.rect, self.label, self.value, ACCENT, highlighted);
    }
}

/// Value in 0..1; left/right step it, click or drag on the bar sets it.
pub struct Slider<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub value: f32,
    pub step: f32,
}

impl Slider<'_> {
    fn bar(&self) -> Rect {
        let w = (self.rect.w * 0.45).round();
        Rect::new(
            self.rect.right() - w - 10.0,
            self.rect.center().y - 3.0,
            w,
            6.0,
        )
    }

    pub fn handle(&mut self, i: usize, action: Action, input: &UiInput) -> bool {
        let old = self.value;
        if let Action::Adjust(j, d) = action {
            if j == i {
                self.value += d * self.step;
            }
        }
        let bar = self.bar();
        let on_bar = Rect::new(bar.x, self.rect.y, bar.w, self.rect.h);
        if on_bar.contains(input.mouse) && action.pointer(i, input, on_bar).is_some() {
            self.value = (input.mouse.x - bar.x) / bar.w;
        }
        self.value = self.value.clamp(0.0, 1.0);
        self.value != old
    }

    pub fn draw(&self, highlighted: bool) {
        draw_frame(self.rect, highlighted);
        draw_ui_text(
            self.label,
            self.rect.x + 10.0,
            baseline(self.rect),
            TextStyle::Body,
            WHITE,
        );
        let bar = self.bar();
        draw_rectangle(bar.x, bar.y, bar.w, bar.h, Color::new(0.0, 0.0, 0.0, 0.6));
        draw_rectangle(bar.x, bar.y, (bar.w * self.value).round(), bar.h, ACCENT);
        let knob = (bar.x + bar.w * self.value).round();
        draw_rectangle(knob - 2.0, bar.y - 3.0, 4.0, bar.h + 6.0, WHITE);
    }
}

/// A row of tab headers. Left/right on the focused row or a click on a header switches.
pub struct Tabs<'a> {
    pub rect: Rect,
    pub labels: &'a [&'a str],
    pub active: usize,
}

impl Tabs<'_> {
    pub fn header(&self, k: usize) -> Rect {
        let w = self.rect.w / self.labels.len() as f32;
        Rect::new(self.rect.x + w * k as f32, self.rect.y, w, self.rect.h)
    }

    pub fn handle(&mut self, i: usize, action: Action, input: &UiInput) -> bool {
        let n = self.labels.len();
        let old = self.active;
        match action {
            Action::Adjust(j, d) if j == i => {
                self.active = (self.active as i32 + d as i32).rem_euclid(n as i32) as usize;
            }
            Action::Confirm(j) if j == i && input.clicked => {
                if let Some(k) = (0..n).find(|&k| self.header(k).contains(input.mouse)) {
                    self.active = k;
                }
            }
            _ => {}
        }
        self.active != old
    }

    pub fn draw(&self, highlighted: bool) {
        for (k, label) in self.labels.iter().enumerate() {
            let r = self.header(k);
            let active = k == self.active;
            draw_frame(r, active && highlighted);
            let w = measure_ui_text(label, TextStyle::Body).width;
            let color = if active { ACCENT } else { GRAY };
            draw_ui_text(
                label,
                (r.x + (r.w - w) * 0.5).round(),
                baseline(r),
                TextStyle::Body,
                color,
            );
        }
    }
}

/// Vertical list of equal rows inside `rect`, scrolled by the wheel and kept on focus.
#[allow(dead_code)] // first used by the long lists of the Altar and Codex (V20+)
pub struct ListView {
    pub rect: Rect,
    pub row_h: f32,
    pub gap: f32,
    pub len: usize,
    /// First visible row.
    pub scroll: usize,
}

#[allow(dead_code)]
impl ListView {
    pub fn visible_rows(&self) -> usize {
        (((self.rect.h + self.gap) / (self.row_h + self.gap)) as usize).max(1)
    }

    fn max_scroll(&self) -> usize {
        self.len.saturating_sub(self.visible_rows())
    }

    /// Wheel scrolls; keyboard focus changes scroll just enough to keep `focus` visible.
    pub fn update(&mut self, input: &UiInput, focus: &Focus) {
        if input.wheel != 0.0 && self.rect.contains(input.mouse) {
            self.scroll = if input.wheel > 0.0 {
                self.scroll.saturating_sub(1)
            } else {
                self.scroll + 1
            };
        }
        if focus.device == Device::Keyboard {
            let rows = self.visible_rows();
            if focus.index < self.scroll {
                self.scroll = focus.index;
            } else if focus.index >= self.scroll + rows {
                self.scroll = focus.index + 1 - rows;
            }
        }
        self.scroll = self.scroll.min(self.max_scroll());
    }

    /// Rects of every row; rows scrolled out of view get an empty rect (never hit).
    pub fn rows(&self) -> impl Iterator<Item = Rect> + '_ {
        let rows = self.visible_rows();
        (0..self.len).map(move |i| {
            if (self.scroll..self.scroll + rows).contains(&i) {
                let y = self.rect.y + (i - self.scroll) as f32 * (self.row_h + self.gap);
                Rect::new(self.rect.x, y, self.rect.w, self.row_h)
            } else {
                Rect::new(0.0, 0.0, 0.0, 0.0)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<Rect> {
        (0..3)
            .map(|i| Rect::new(0.0, i as f32 * 20.0, 100.0, 16.0))
            .collect()
    }

    fn at(x: f32, y: f32) -> UiInput {
        UiInput {
            mouse: vec2(x, y),
            ..Default::default()
        }
    }

    #[test]
    fn focus_cues_ui_sounds() {
        let mut f = Focus::default();
        let items = rows();
        audio::take_cues();
        f.update(&at(500.0, 500.0), &items);
        assert!(audio::take_cues().is_empty(), "idle frame is silent");
        let key = |down, confirm, back| UiInput {
            down,
            confirm,
            back,
            ..at(500.0, 500.0)
        };
        f.update(&key(true, false, false), &items);
        assert_eq!(audio::take_cues(), [Sfx::UiMove]);
        f.update(&key(false, true, false), &items);
        assert_eq!(audio::take_cues(), [Sfx::UiConfirm]);
        f.update(&key(false, false, true), &items);
        assert_eq!(audio::take_cues(), [Sfx::UiBack]);
    }

    #[test]
    fn keyboard_moves_and_wraps() {
        let mut f = Focus::default();
        let items = rows();
        let up = UiInput {
            up: true,
            ..Default::default()
        };
        assert_eq!(f.update(&up, &items), Action::None);
        assert_eq!(f.index, 2);
        let enter = UiInput {
            confirm: true,
            ..Default::default()
        };
        assert_eq!(f.update(&enter, &items), Action::Confirm(2));
        assert_eq!(f.highlight(), Some(2));
    }

    #[test]
    fn hover_selects_and_click_confirms() {
        let mut f = Focus::default();
        let items = rows();
        f.update(&at(500.0, 500.0), &items);
        assert_eq!(f.update(&at(10.0, 25.0), &items), Action::None);
        assert_eq!(
            (f.index, f.device, f.highlight()),
            (1, Device::Mouse, Some(1))
        );
        let click = UiInput {
            clicked: true,
            mouse_down: true,
            ..at(10.0, 45.0)
        };
        assert_eq!(f.update(&click, &items), Action::Confirm(2));
        // Mouse off every item: nothing highlighted, focus remembered.
        f.update(&at(500.0, 0.0), &items);
        assert_eq!((f.index, f.highlight()), (2, None));
        // A key brings the keyboard highlight back.
        let down = UiInput {
            down: true,
            ..at(500.0, 0.0)
        };
        f.update(&down, &items);
        assert_eq!((f.index, f.highlight()), (0, Some(0)));
    }

    #[test]
    fn still_mouse_does_not_steal_keyboard_focus() {
        let mut f = Focus::default();
        let items = rows();
        f.update(&at(10.0, 5.0), &items);
        let down = UiInput {
            down: true,
            ..at(10.0, 5.0)
        };
        f.update(&down, &items);
        f.update(&at(10.0, 5.0), &items);
        assert_eq!((f.index, f.device), (1, Device::Keyboard));
    }

    #[test]
    fn toggle_and_slider_respond() {
        let mut t = Toggle {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            label: "",
            on: false,
        };
        assert!(t.handle(0, Action::Adjust(0, 1.0)));
        assert!(t.on);
        assert!(!t.handle(0, Action::Confirm(1)));
        assert_eq!(Choice::handle(2, Action::Adjust(2, -1.0)), -1);
        assert_eq!(Choice::handle(2, Action::Confirm(2)), 1);
        assert_eq!(Choice::handle(2, Action::Confirm(0)), 0);
        let mut s = Slider {
            rect: Rect::new(0.0, 0.0, 200.0, 20.0),
            label: "",
            value: 0.95,
            step: 0.1,
        };
        assert!(s.handle(0, Action::Adjust(0, 1.0), &UiInput::default()));
        assert_eq!(s.value, 1.0);
        let bar = s.bar();
        let drag = UiInput {
            mouse_down: true,
            ..at(bar.x + bar.w * 0.25, 10.0)
        };
        let action = Action::Pointer(0, 0.0);
        s.handle(0, action, &drag);
        assert!((s.value - 0.25).abs() < 1e-4);
    }

    #[test]
    fn tabs_cycle_and_click() {
        let mut t = Tabs {
            rect: Rect::new(0.0, 0.0, 300.0, 20.0),
            labels: &["a", "b", "c"],
            active: 0,
        };
        t.handle(0, Action::Adjust(0, -1.0), &UiInput::default());
        assert_eq!(t.active, 2);
        let click = UiInput {
            clicked: true,
            ..at(150.0, 10.0)
        };
        t.handle(0, Action::Confirm(0), &click);
        assert_eq!(t.active, 1);
    }

    #[test]
    fn button_column_is_centred_and_stacked() {
        let r = button_column(vec2(640.0, 360.0), 100.0, 3);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].x + r[0].w * 0.5, 320.0);
        assert_eq!(r[2].y, 100.0 + 2.0 * (BUTTON_H + BUTTON_GAP));
    }

    #[test]
    fn list_scrolls_with_wheel_and_follows_focus() {
        let mut list = ListView {
            rect: Rect::new(0.0, 0.0, 100.0, 50.0),
            row_h: 20.0,
            gap: 5.0,
            len: 6,
            scroll: 0,
        };
        assert_eq!(list.visible_rows(), 2);
        let mut f = Focus {
            device: Device::Mouse,
            ..Default::default()
        };
        for _ in 0..10 {
            let wheel = UiInput {
                wheel: -1.0,
                ..at(10.0, 10.0)
            };
            list.update(&wheel, &f);
        }
        assert_eq!(list.scroll, 4);
        f.device = Device::Keyboard;
        f.index = 1;
        list.update(&UiInput::default(), &f);
        assert_eq!(list.scroll, 1);
        let rects: Vec<Rect> = list.rows().collect();
        assert_eq!(rects[0].w, 0.0);
        assert_eq!(rects[1].y, 0.0);
        assert_eq!(rects[2].y, 25.0);
    }
}

//! Window adaptation: the logical frame keeps a fixed height, takes its width from the
//! window aspect ratio and is scaled into the window (integer or fractional scale).

use crate::config::*;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScaleMode {
    /// Integer scale with thin bars; every logical pixel is the same size.
    PixelPerfect,
    /// Fractional scale that fills the window (nearest filtering).
    Fill,
}

/// Where the logical frame goes in the window.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Layout {
    /// Logical frame size in logical pixels.
    pub view: Vec2,
    /// Window pixels per logical pixel.
    pub scale: f32,
    /// Top-left corner of the frame in the window.
    pub offset: Vec2,
}

impl Layout {
    pub fn new(window: Vec2, mode: ScaleMode) -> Self {
        let snap = |s: f32| match mode {
            ScaleMode::PixelPerfect if s >= 1.0 => s.floor(),
            _ => s,
        };
        let mut scale = snap(window.y / LOGICAL_H);
        let w = (window.x / scale)
            .floor()
            .clamp(MIN_LOGICAL_W, MAX_LOGICAL_W);
        // Windows narrower than 4:3 shrink the scale and get bars above and below.
        scale = snap(scale.min(window.x / w));
        let view = vec2(w, LOGICAL_H);
        let offset = ((window - view * scale) * 0.5).max(Vec2::ZERO).floor();
        Self {
            view,
            scale,
            offset,
        }
    }

    /// Converts a window position (e.g. the mouse) to logical frame coordinates.
    #[allow(dead_code)] // used by the mouse UI layer (V5)
    pub fn to_logical(self, p: Vec2) -> Vec2 {
        (p - self.offset) / self.scale
    }
}

/// Owns the render target and follows window resizes, scale mode and fullscreen.
pub struct Screen {
    target: RenderTarget,
    pub layout: Layout,
    pub mode: ScaleMode,
    fullscreen: bool,
    /// Window size to restore when leaving fullscreen.
    windowed: Vec2,
}

impl Screen {
    /// `windowed` is the size to restore when leaving fullscreen (from the settings).
    pub fn new(mode: ScaleMode, fullscreen: bool, windowed: Vec2) -> Self {
        let layout = Layout::new(window_size(), mode);
        Self {
            target: make_target(layout.view),
            layout,
            mode,
            fullscreen,
            windowed,
        }
    }

    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Current window size, or the remembered one while in fullscreen.
    pub fn windowed_size(&self) -> Vec2 {
        if self.fullscreen {
            self.windowed
        } else {
            window_size()
        }
    }

    pub fn view(&self) -> Vec2 {
        self.layout.view
    }

    /// Recomputes the layout; the target is recreated when the logical size changes.
    pub fn update(&mut self) {
        let layout = Layout::new(window_size(), self.mode);
        if layout.view != self.layout.view {
            self.target = make_target(layout.view);
        }
        self.layout = layout;
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            ScaleMode::PixelPerfect => ScaleMode::Fill,
            ScaleMode::Fill => ScaleMode::PixelPerfect,
        };
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
        if self.fullscreen {
            self.windowed = window_size();
            set_fullscreen(true);
        } else {
            set_fullscreen(false);
            request_new_screen_size(self.windowed.x, self.windowed.y);
        }
    }

    /// Camera that draws into the logical frame with `origin` at its top-left corner.
    /// Render targets are stored flipped, hence the inverted y zoom.
    pub fn camera(&self, origin: Vec2) -> Camera2D {
        let view = self.layout.view;
        let mut cam = Camera2D::from_display_rect(Rect::new(origin.x, origin.y, view.x, view.y));
        cam.zoom.y = -cam.zoom.y;
        cam.render_target = Some(self.target.clone());
        cam
    }

    /// Draws the logical frame into the window.
    pub fn present(&self) {
        set_default_camera();
        clear_background(BLACK);
        let Layout {
            view,
            scale,
            offset,
        } = self.layout;
        draw_texture_ex(
            &self.target.texture,
            offset.x,
            offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(view * scale),
                ..Default::default()
            },
        );
    }
}

fn window_size() -> Vec2 {
    vec2(screen_width(), screen_height())
}

fn make_target(view: Vec2) -> RenderTarget {
    let target = render_target(view.x as u32, view.y as u32);
    target.texture.set_filter(FilterMode::Nearest);
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pp(w: f32, h: f32) -> Layout {
        Layout::new(vec2(w, h), ScaleMode::PixelPerfect)
    }

    fn fill(w: f32, h: f32) -> Layout {
        Layout::new(vec2(w, h), ScaleMode::Fill)
    }

    fn assert_fits(l: Layout, window: Vec2) {
        let size = l.view * l.scale;
        assert!(l.offset.x >= 0.0 && l.offset.y >= 0.0, "{l:?}");
        assert!(
            size.x <= window.x + 0.01 && size.y <= window.y + 0.01,
            "{l:?}"
        );
        assert!((MIN_LOGICAL_W..=MAX_LOGICAL_W).contains(&l.view.x));
        assert_eq!(l.view.y, LOGICAL_H);
    }

    #[test]
    fn sixteen_nine_fills_exactly() {
        let l = pp(1920.0, 1080.0);
        assert_eq!(
            (l.view, l.scale, l.offset),
            (vec2(640.0, 360.0), 3.0, Vec2::ZERO)
        );
        assert_eq!(fill(1280.0, 720.0).view.x, 640.0);
    }

    #[test]
    fn sixteen_ten() {
        let l = pp(1920.0, 1200.0);
        assert_eq!((l.view.x, l.scale, l.offset), (640.0, 3.0, vec2(0.0, 60.0)));
        let l = fill(1920.0, 1200.0);
        assert_eq!(l.view.x, 576.0);
        assert!(l.scale * LOGICAL_H > 1199.0);
    }

    #[test]
    fn ultrawide_sees_more_world() {
        let l = pp(2560.0, 1080.0);
        assert_eq!((l.view.x, l.scale), (853.0, 3.0));
        assert!(l.offset.x <= 1.0);
        assert_eq!(pp(3440.0, 1440.0).view.x, 860.0);
        // Wider than 21:9 clamps and adds side bars.
        let l = pp(4000.0, 1000.0);
        assert_eq!((l.view.x, l.scale), (MAX_LOGICAL_W, 2.0));
    }

    #[test]
    fn four_three() {
        let l = pp(1024.0, 768.0);
        assert_eq!((l.view.x, l.scale, l.offset), (512.0, 2.0, vec2(0.0, 24.0)));
        let l = fill(1024.0, 768.0);
        assert_eq!(l.view.x, 480.0);
        assert_eq!(l.offset, Vec2::ZERO);
    }

    #[test]
    fn every_window_fits() {
        for (w, h) in [
            (1920.0, 1080.0),
            (1920.0, 1200.0),
            (2560.0, 1080.0),
            (1024.0, 768.0),
            (800.0, 600.0),
            (1080.0, 1920.0),
            (320.0, 200.0),
            (5000.0, 400.0),
        ] {
            let window = vec2(w, h);
            assert_fits(pp(w, h), window);
            assert_fits(fill(w, h), window);
        }
    }

    #[test]
    fn mouse_maps_to_logical() {
        let l = pp(1920.0, 1200.0);
        assert_eq!(l.to_logical(vec2(0.0, 60.0)), Vec2::ZERO);
        assert_eq!(l.to_logical(vec2(1920.0, 1140.0)), vec2(640.0, 360.0));
    }
}

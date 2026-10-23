//! UI motion in one place (design.md 7.14), so every screen shares the same timing: easing
//! curves, the open/deal timings and a smoothed value for things that glide (the focus
//! cursor, draining bars). Times are real seconds of UI time, never simulation time.

/// A panel opens (slide + fade) in this long.
pub const OPEN: f32 = 0.18;
/// How far a panel slides up while it opens.
pub const OPEN_SLIDE: f32 = 8.0;
/// Cards are dealt one after another, this far apart, each taking `DEAL`.
pub const DEAL_STAGGER: f32 = 0.06;
pub const DEAL: f32 = 0.24;
/// How far below its place a card starts.
pub const DEAL_SLIDE: f32 = 22.0;
/// The selected item's glow breathes at this rate (Hz).
pub const PULSE_HZ: f32 = 1.4;
/// Glide rate of the focus cursor (per second, exponential).
pub const GLIDE: f32 = 22.0;

pub fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

/// Overshoots a little past 1 before settling (cards landing).
pub fn ease_out_back(t: f32) -> f32 {
    const S: f32 = 1.4;
    let u = t.clamp(0.0, 1.0) - 1.0;
    1.0 + (S + 1.0) * u * u * u + S * u * u
}

pub fn ease_in_out_sine(t: f32) -> f32 {
    0.5 - 0.5 * (t.clamp(0.0, 1.0) * std::f32::consts::PI).cos()
}

/// Progress 0..1 of something `len` seconds long that started `delay` seconds after `t = 0`.
pub fn progress(t: f32, delay: f32, len: f32) -> f32 {
    ((t - delay) / len).clamp(0.0, 1.0)
}

/// Opening of a panel at time `t`: (alpha, y offset in pixels, whole).
pub fn open(t: f32) -> (f32, f32) {
    let k = ease_out_cubic(progress(t, 0.0, OPEN));
    (k, ((1.0 - k) * OPEN_SLIDE).round())
}

/// Card `i` of a dealt row at time `t`: (alpha, y offset in pixels, whole).
pub fn deal(t: f32, i: usize) -> (f32, f32) {
    let p = progress(t, i as f32 * DEAL_STAGGER, DEAL);
    let alpha = ease_out_cubic((p * 2.0).min(1.0));
    (alpha, ((1.0 - ease_out_back(p)) * DEAL_SLIDE).round())
}

/// Soft breathing 0..1 for the selected item at time `t`.
pub fn pulse(t: f32) -> f32 {
    ease_in_out_sine(0.5 + 0.5 * (t * PULSE_HZ * std::f32::consts::TAU).sin())
}

/// A value that glides toward its target, frame-rate independent.
#[derive(Clone, Copy, Debug, Default)]
pub struct Glide {
    pub value: Option<f32>,
}

impl Glide {
    /// Moves toward `target` (snaps on the first call) and returns the new value.
    pub fn update(&mut self, target: f32, rate: f32, dt: f32) -> f32 {
        let v = match self.value {
            Some(v) => {
                let k = 1.0 - (-rate * dt).exp();
                let v = v + (target - v) * k;
                if (target - v).abs() < 0.25 {
                    target
                } else {
                    v
                }
            }
            None => target,
        };
        self.value = Some(v);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_start_at_0_and_end_at_1() {
        for f in [ease_out_cubic, ease_out_back, ease_in_out_sine] {
            assert!(f(0.0).abs() < 1e-6);
            assert!((f(1.0) - 1.0).abs() < 1e-6);
        }
        assert!((0..100).any(|i| ease_out_back(i as f32 / 100.0) > 1.0), "overshoots");
    }

    #[test]
    fn open_and_deal_settle() {
        assert_eq!(open(0.0), (0.0, OPEN_SLIDE));
        assert_eq!(open(OPEN), (1.0, 0.0));
        let last = 2.0 * DEAL_STAGGER + DEAL;
        assert_eq!(deal(last, 2), (1.0, 0.0));
        assert_eq!(deal(DEAL_STAGGER, 1).0, 0.0, "staggered");
    }

    #[test]
    fn glide_snaps_first_then_converges() {
        let mut g = Glide::default();
        assert_eq!(g.update(10.0, GLIDE, 0.016), 10.0);
        let mid = g.update(40.0, GLIDE, 0.016);
        assert!(mid > 10.0 && mid < 40.0);
        for _ in 0..60 {
            g.update(40.0, GLIDE, 0.016);
        }
        assert_eq!(g.value, Some(40.0));
    }
}

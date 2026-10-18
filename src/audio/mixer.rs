//! Decides which effects actually play: a voice cap and a minimum gap per effect,
//! a random pitch variant (jitter) and the soul pickup combo. Pure logic on a
//! clock passed in by the caller; `std` only.

use super::sfx::{Sfx, COMBO_STEPS};

/// Upper bound for `Sfx::cap`.
const MAX_VOICES: usize = 8;
/// Soul pickups closer together than this keep climbing the combo ladder.
pub const COMBO_WINDOW: f64 = 0.6;

#[derive(Clone, Copy)]
struct Slot {
    /// When each voice of this effect stops sounding.
    ends: [f64; MAX_VOICES],
    last_start: f64,
}

pub struct Mixer {
    slots: [Slot; Sfx::COUNT],
    combo: usize,
    last_soul: f64,
    rng: u32,
}

impl Default for Mixer {
    fn default() -> Self {
        Self {
            slots: [Slot {
                ends: [f64::NEG_INFINITY; MAX_VOICES],
                last_start: f64::NEG_INFINITY,
            }; Sfx::COUNT],
            combo: 0,
            last_soul: f64::NEG_INFINITY,
            rng: 0x9E37_79B9,
        }
    }
}

impl Mixer {
    /// Asks to start `sfx` at time `now` (seconds). Returns the variant to play,
    /// or `None` if the effect is at its cap or was started too recently.
    pub fn trigger(&mut self, sfx: Sfx, now: f64) -> Option<usize> {
        let variant = match sfx {
            Sfx::Soul => self.soul_step(now),
            _ => self.random(sfx.variants().len()),
        };
        let slot = &mut self.slots[sfx as usize];
        if now - slot.last_start < f64::from(sfx.gap()) {
            return None;
        }
        let cap = sfx.cap().min(MAX_VOICES);
        let free = slot.ends[..cap].iter().position(|&end| end <= now)?;
        slot.ends[free] = now + f64::from(sfx.length());
        slot.last_start = now;
        Some(variant)
    }

    /// Voices of `sfx` still sounding at `now`.
    #[cfg(test)]
    fn playing(&self, sfx: Sfx, now: f64) -> usize {
        self.slots[sfx as usize]
            .ends
            .iter()
            .filter(|&&e| e > now)
            .count()
    }

    /// The combo climbs while pickups keep coming, even ones dropped by the cap.
    fn soul_step(&mut self, now: f64) -> usize {
        self.combo = if now - self.last_soul < COMBO_WINDOW {
            (self.combo + 1).min(COMBO_STEPS.len() - 1)
        } else {
            0
        };
        self.last_soul = now;
        self.combo
    }

    /// Xorshift32, uniform enough for picking among a handful of variants.
    fn random(&mut self, n: usize) -> usize {
        if n <= 1 {
            return 0;
        }
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x as usize % n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_voices_and_frees_them_when_done() {
        let mut m = Mixer::default();
        let sfx = Sfx::KillArmored;
        let step = f64::from(sfx.gap()) + 0.001;
        let mut t = 0.0;
        let mut started = 0;
        while t < f64::from(sfx.length()) * 0.5 {
            started += m.trigger(sfx, t).is_some() as usize;
            t += step;
        }
        assert_eq!(started, sfx.cap());
        assert_eq!(m.playing(sfx, t), sfx.cap());
        let later = t + f64::from(sfx.length());
        assert_eq!(m.playing(sfx, later), 0);
        assert!(m.trigger(sfx, later).is_some());
    }

    #[test]
    fn gap_blocks_same_frame_repeats() {
        let mut m = Mixer::default();
        assert!(m.trigger(Sfx::KillBeast, 1.0).is_some());
        assert!(m.trigger(Sfx::KillBeast, 1.0).is_none());
        assert!(m.trigger(Sfx::KillHuman, 1.0).is_some(), "per-effect state");
        assert!(m.trigger(Sfx::KillBeast, 1.0 + 0.05).is_some());
    }

    #[test]
    fn jitter_uses_every_variant() {
        let mut m = Mixer::default();
        let n = Sfx::Fire.variants().len();
        let mut seen = vec![false; n];
        for i in 0..200 {
            let v = m.trigger(Sfx::Fire, i as f64).unwrap();
            seen[v] = true;
        }
        assert!(seen.iter().all(|&s| s));
        assert_eq!(m.trigger(Sfx::LevelUp, 0.0), Some(0));
    }

    #[test]
    fn soul_combo_climbs_caps_and_resets() {
        let mut m = Mixer::default();
        let steps: Vec<Option<usize>> = (0..10)
            .map(|i| m.trigger(Sfx::Soul, f64::from(i) * 0.2))
            .collect();
        let expected: Vec<Option<usize>> = [0, 1, 2, 3, 4, 5, 6, 7, 7, 7]
            .iter()
            .map(|&s| Some(s))
            .collect();
        assert_eq!(steps, expected);
        assert_eq!(m.trigger(Sfx::Soul, 1.8 + COMBO_WINDOW + 0.1), Some(0));
    }
}

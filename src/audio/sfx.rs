//! Sound effect presets: short sfxr-style patches for the chip synth, rendered to
//! PCM at load time in a few pitch variants each (jitter, or the soul combo
//! ladder). `std` only, like the synth, so `render_audio` can export them.

use super::synth::{Channel, Echo, Instrument, Macro, Synth, Vibrato, SAMPLE_RATE};
use Channel::*;

/// Rendered effects are normalised to this peak; `Sfx::gain` sets the mix level.
const PEAK: f32 = 0.9;
/// Short fade at the very end so a cut-off tail never clicks.
const TAIL_FADE: f32 = 0.005;

/// Random pitch offsets (semitones) for effects that repeat a lot.
const JITTER: [f32; 5] = [-0.5, -0.25, 0.0, 0.25, 0.5];
/// Soul pickups climb a minor pentatonic while the combo lasts.
pub const COMBO_STEPS: [f32; 8] = [0.0, 3.0, 5.0, 7.0, 10.0, 12.0, 15.0, 17.0];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Fire,
    Hit,
    KillBeast,
    KillHuman,
    KillArmored,
    Soul,
    Hurt,
    Death,
    LevelUp,
    UiMove,
    UiConfirm,
    UiBack,
}

impl Sfx {
    pub const COUNT: usize = 12;
    pub const ALL: [Sfx; Sfx::COUNT] = [
        Sfx::Fire,
        Sfx::Hit,
        Sfx::KillBeast,
        Sfx::KillHuman,
        Sfx::KillArmored,
        Sfx::Soul,
        Sfx::Hurt,
        Sfx::Death,
        Sfx::LevelUp,
        Sfx::UiMove,
        Sfx::UiConfirm,
        Sfx::UiBack,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Sfx::Fire => "fire",
            Sfx::Hit => "hit",
            Sfx::KillBeast => "kill_beast",
            Sfx::KillHuman => "kill_human",
            Sfx::KillArmored => "kill_armored",
            Sfx::Soul => "soul",
            Sfx::Hurt => "hurt",
            Sfx::Death => "death",
            Sfx::LevelUp => "level_up",
            Sfx::UiMove => "ui_move",
            Sfx::UiConfirm => "ui_confirm",
            Sfx::UiBack => "ui_back",
        }
    }

    /// Pitch offsets in semitones, one pre-rendered variant each.
    pub fn variants(self) -> &'static [f32] {
        match self {
            Sfx::Fire | Sfx::Hit | Sfx::KillBeast | Sfx::KillHuman | Sfx::KillArmored => &JITTER,
            Sfx::Hurt => &JITTER[1..4],
            Sfx::Soul => &COMBO_STEPS,
            _ => &[0.0],
        }
    }

    /// Rendered length in seconds (the same for every variant).
    pub fn length(self) -> f32 {
        match self {
            Sfx::Fire => 0.22,
            Sfx::Hit => 0.1,
            Sfx::KillBeast => 0.16,
            Sfx::KillHuman => 0.2,
            Sfx::KillArmored => 0.45,
            Sfx::Soul => 0.18,
            Sfx::Hurt => 0.28,
            Sfx::Death => 1.5,
            Sfx::LevelUp => 1.1,
            Sfx::UiMove => 0.07,
            Sfx::UiConfirm | Sfx::UiBack => 0.18,
        }
    }

    /// Most copies that may sound at once.
    pub fn cap(self) -> usize {
        match self {
            Sfx::Hit | Sfx::KillBeast | Sfx::KillHuman | Sfx::KillArmored => 4,
            Sfx::Fire | Sfx::Soul => 3,
            Sfx::UiMove | Sfx::UiConfirm | Sfx::UiBack => 2,
            Sfx::Hurt | Sfx::Death | Sfx::LevelUp => 1,
        }
    }

    /// Minimum seconds between two starts, so a burst doesn't stack in phase.
    pub fn gap(self) -> f32 {
        match self {
            Sfx::Hit | Sfx::UiMove => 0.03,
            Sfx::KillBeast | Sfx::KillHuman | Sfx::KillArmored | Sfx::Soul => 0.04,
            Sfx::Fire => 0.05,
            _ => 0.0,
        }
    }

    /// Playback level before the volume settings.
    pub fn gain(self) -> f32 {
        match self {
            Sfx::Fire | Sfx::Soul | Sfx::UiMove => 0.3,
            Sfx::Hit | Sfx::UiConfirm | Sfx::UiBack => 0.35,
            Sfx::KillBeast => 0.4,
            Sfx::KillHuman | Sfx::KillArmored => 0.45,
            Sfx::LevelUp => 0.55,
            Sfx::Hurt | Sfx::Death => 0.6,
        }
    }
}

struct Note {
    at: f32,
    ch: Channel,
    note: f32,
    inst: usize,
    vel: f32,
}

struct Patch {
    instruments: Vec<Instrument>,
    notes: Vec<Note>,
    echo: Echo,
}

fn note(at: f32, ch: Channel, note: f32, inst: usize, vel: f32) -> Note {
    Note {
        at,
        ch,
        note,
        inst,
        vel,
    }
}

/// An instrument with only a volume envelope (it ends silent and holds there).
fn env(volume: &[i8]) -> Instrument {
    Instrument {
        volume: Macro::new(volume),
        ..Instrument::default()
    }
}

fn with_duty(mut inst: Instrument, duty: i8) -> Instrument {
    inst.duty = Macro::new(&[duty]);
    inst
}

/// A linear fade from `from` to 0 over `ticks` macro ticks.
fn fade(from: i8, ticks: usize) -> Vec<i8> {
    (0..=ticks)
        .map(|i| (f32::from(from) * (1.0 - i as f32 / ticks as f32)).round() as i8)
        .collect()
}

fn patch(sfx: Sfx) -> Patch {
    let dry = Echo::OFF;
    match sfx {
        // Dark fireball: a falling thin pulse over a breath of noise.
        Sfx::Fire => Patch {
            instruments: vec![
                Instrument {
                    sweep: -36.0,
                    ..with_duty(env(&[8, 7, 6, 5, 4, 3, 2, 1, 0]), 0)
                },
                Instrument {
                    sweep: -30.0,
                    ..env(&[6, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0])
                },
            ],
            notes: vec![
                note(0.0, Pulse1, 64.0, 0, 1.0),
                note(0.0, Noise, 70.0, 1, 1.0),
            ],
            echo: dry,
        },
        Sfx::Hit => Patch {
            instruments: vec![
                env(&[13, 8, 4, 2, 0]),
                Instrument {
                    sweep: -60.0,
                    ..with_duty(env(&[10, 6, 3, 0]), 1)
                },
            ],
            notes: vec![
                note(0.0, Noise, 96.0, 0, 1.0),
                note(0.0, Pulse1, 52.0, 1, 1.0),
            ],
            echo: dry,
        },
        // Rat squeak.
        Sfx::KillBeast => Patch {
            instruments: vec![
                Instrument {
                    arp: Macro::new(&[0, 4, 7, 4, 0, -5]),
                    ..with_duty(env(&[11, 12, 10, 8, 6, 4, 2, 0]), 0)
                },
                env(&[6, 3, 0]),
            ],
            notes: vec![
                note(0.0, Pulse1, 86.0, 0, 1.0),
                note(0.0, Noise, 100.0, 1, 1.0),
            ],
            echo: dry,
        },
        // Grunt and thud.
        Sfx::KillHuman => Patch {
            instruments: vec![
                Instrument {
                    sweep: -24.0,
                    ..env(&[15, 15, 12, 9, 6, 3, 0])
                },
                Instrument {
                    pitch: Macro::new(&[0, -8, -16, -24]),
                    ..env(&[9, 8, 6, 4, 2, 0])
                },
                env(&[10, 6, 3, 1, 0]),
            ],
            notes: vec![
                note(0.0, Triangle, 50.0, 0, 1.0),
                note(0.0, Pulse1, 57.0, 1, 1.0),
                note(0.0, Noise, 80.0, 2, 1.0),
            ],
            echo: dry,
        },
        // Armour clang: two pulses a tritone apart ring over metallic short noise.
        Sfx::KillArmored => {
            let ring = [14, 12, 11, 10, 9, 8, 7, 6, 5, 4, 4, 3, 3, 2, 2, 1, 1, 0];
            Patch {
                instruments: vec![
                    with_duty(env(&ring), 1),
                    with_duty(env(&ring), 0),
                    with_duty(env(&[12, 6, 2, 0]), 1),
                ],
                notes: vec![
                    note(0.0, Pulse1, 81.0, 0, 1.0),
                    note(0.0, Pulse2, 87.0, 1, 0.7),
                    note(0.0, Noise, 90.0, 2, 1.0),
                ],
                echo: Echo {
                    delay: 0.07,
                    feedback: 0.3,
                    mix: 0.3,
                },
            }
        }
        Sfx::Soul => Patch {
            instruments: vec![
                Instrument {
                    arp: Macro::new(&[0, 7, 12]),
                    duty: Macro::new(&[2, 1]),
                    ..env(&[9, 12, 10, 8, 6, 4, 2, 1, 0])
                },
                env(&[6, 5, 4, 3, 2, 1, 0]),
            ],
            notes: vec![
                note(0.0, Pulse1, 76.0, 0, 1.0),
                note(0.0, Wave, 88.0, 1, 1.0),
            ],
            echo: dry,
        },
        Sfx::Hurt => Patch {
            instruments: vec![
                Instrument {
                    arp: Macro::new(&[0, -2, -4, -6, -8, -10]),
                    ..env(&[15, 14, 12, 10, 8, 6, 4, 2, 0])
                },
                env(&[14, 10, 7, 4, 2, 0]),
                Instrument {
                    sweep: -20.0,
                    ..env(&[15, 12, 8, 4, 0])
                },
            ],
            notes: vec![
                note(0.0, Pulse1, 58.0, 0, 1.0),
                note(0.0, Noise, 84.0, 1, 1.0),
                note(0.0, Triangle, 40.0, 2, 1.0),
            ],
            echo: dry,
        },
        // Veyra's minor 2nd falling apart, with a long breath of smoke.
        Sfx::Death => {
            let fall = Instrument {
                sweep: -20.0,
                vibrato: Vibrato {
                    depth: 0.4,
                    rate_hz: 8.0,
                    delay: 0.1,
                },
                ..with_duty(env(&fade(15, 60)), 1)
            };
            Patch {
                instruments: vec![
                    fall.clone(),
                    with_duty(fall, 0),
                    env(&fade(15, 50)),
                    env(&fade(10, 40)),
                ],
                notes: vec![
                    note(0.0, Pulse1, 69.0, 0, 1.0),
                    note(0.0, Pulse2, 70.0, 1, 0.6),
                    note(0.0, Triangle, 38.0, 2, 1.0),
                    note(0.0, Noise, 60.0, 3, 1.0),
                ],
                echo: Echo {
                    delay: 0.18,
                    feedback: 0.35,
                    mix: 0.3,
                },
            }
        }
        // D Phrygian dominant arpeggio ending on the leitmotif's b2 hook.
        Sfx::LevelUp => {
            let mut ring = vec![14, 13, 12, 11, 11, 10, 10, 9, 9, 8, 8, 7, 7, 6, 6, 5];
            ring.extend(fade(5, 20));
            let lead = with_duty(env(&ring), 1);
            let arp = [62.0, 66.0, 69.0, 74.0, 75.0, 74.0];
            let times = [0.0, 0.07, 0.14, 0.21, 0.28, 0.38];
            let mut notes: Vec<Note> = arp
                .iter()
                .zip(times)
                .flat_map(|(&n, t)| {
                    [
                        note(t, Pulse1, n, 0, 1.0),
                        note(t, Pulse2, n - 12.0, 1, 0.5),
                    ]
                })
                .collect();
            notes.push(note(0.0, Triangle, 50.0, 2, 1.0));
            Patch {
                instruments: vec![lead.clone(), with_duty(lead, 0), env(&fade(15, 50))],
                notes,
                echo: Echo {
                    delay: 0.12,
                    feedback: 0.35,
                    mix: 0.35,
                },
            }
        }
        Sfx::UiMove => Patch {
            instruments: vec![with_duty(env(&[8, 5, 2, 0]), 0)],
            notes: vec![note(0.0, Pulse1, 88.0, 0, 1.0)],
            echo: dry,
        },
        Sfx::UiConfirm | Sfx::UiBack => {
            let (a, b) = if sfx == Sfx::UiConfirm {
                (79.0, 86.0)
            } else {
                (84.0, 77.0)
            };
            Patch {
                instruments: vec![with_duty(env(&[11, 9, 7, 5, 3, 1, 0]), 1)],
                notes: vec![note(0.0, Pulse1, a, 0, 1.0), note(0.05, Pulse1, b, 0, 1.0)],
                echo: dry,
            }
        }
    }
}

/// Renders `sfx` shifted by `pitch` semitones, normalised to `PEAK`.
pub fn render(sfx: Sfx, pitch: f32) -> Vec<f32> {
    let p = patch(sfx);
    let sr = SAMPLE_RATE as f32;
    let mut synth = Synth::new(SAMPLE_RATE, p.instruments);
    synth.set_echo(p.echo);
    let mut out = vec![0.0; (sfx.length() * sr) as usize];
    let mut pos = 0;
    for n in &p.notes {
        let at = ((n.at * sr) as usize).clamp(pos, out.len());
        synth.render(&mut out[pos..at]);
        pos = at;
        synth.note_on(n.ch, n.note + pitch, n.inst, n.vel);
    }
    synth.render(&mut out[pos..]);

    let fade_len = ((TAIL_FADE * sr) as usize).min(out.len());
    let start = out.len() - fade_len;
    for (i, s) in out[start..].iter_mut().enumerate() {
        *s *= 1.0 - i as f32 / (fade_len - 1).max(1) as f32;
    }
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.0 {
        out.iter_mut().for_each(|s| *s *= PEAK / peak);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(s: &[f32]) -> f32 {
        (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
    }

    #[test]
    fn all_lists_every_sfx_once() {
        for (i, sfx) in Sfx::ALL.iter().enumerate() {
            assert_eq!(*sfx as usize, i);
        }
    }

    #[test]
    fn every_variant_renders_audible_and_in_range() {
        for sfx in Sfx::ALL {
            assert!(sfx.cap() >= 1 && sfx.gain() > 0.0 && sfx.gain() <= 1.0);
            for &p in sfx.variants() {
                let out = render(sfx, p);
                let name = sfx.name();
                assert_eq!(out.len(), (sfx.length() * SAMPLE_RATE as f32) as usize);
                assert!(out.iter().all(|s| s.abs() <= PEAK + 1e-4), "{name}");
                assert!(rms(&out) > 0.03, "{name} {p}: rms {}", rms(&out));
                assert_eq!(*out.last().unwrap(), 0.0, "{name}");
            }
        }
    }

    #[test]
    fn variants_differ_and_soul_ladder_climbs() {
        let a = render(Sfx::Hit, JITTER[0]);
        let b = render(Sfx::Hit, JITTER[4]);
        assert_ne!(a, b);
        assert!(COMBO_STEPS.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(Sfx::Soul.variants(), &COMBO_STEPS);
    }

    #[test]
    fn rendering_is_deterministic() {
        assert_eq!(
            render(Sfx::KillArmored, 0.25),
            render(Sfx::KillArmored, 0.25)
        );
    }
}

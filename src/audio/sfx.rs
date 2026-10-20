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

/// Mix tiers by peak short-term loudness at full volume (see `render_audio`):
/// UI and Hellflare sit far back (about -30 dB) because they repeat constantly,
/// combat in the middle (-25 to -20), events up front (-18 to -14), level-up
/// being the reference at about -16.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Fire,
    Lash,
    Kiss,
    Allure,
    Hellgate,
    Chain,
    Gust,
    Hit,
    HitArmored,
    KillBeast,
    KillHuman,
    KillArmored,
    Soul,
    Hurt,
    Heartbeat,
    Death,
    LevelUp,
    UpgradePick,
    Horn,
    Pause,
    UiMove,
    UiConfirm,
    UiBack,
}

impl Sfx {
    pub const COUNT: usize = 23;
    pub const ALL: [Sfx; Sfx::COUNT] = [
        Sfx::Fire,
        Sfx::Lash,
        Sfx::Kiss,
        Sfx::Allure,
        Sfx::Hellgate,
        Sfx::Chain,
        Sfx::Gust,
        Sfx::Hit,
        Sfx::HitArmored,
        Sfx::KillBeast,
        Sfx::KillHuman,
        Sfx::KillArmored,
        Sfx::Soul,
        Sfx::Hurt,
        Sfx::Heartbeat,
        Sfx::Death,
        Sfx::LevelUp,
        Sfx::UpgradePick,
        Sfx::Horn,
        Sfx::Pause,
        Sfx::UiMove,
        Sfx::UiConfirm,
        Sfx::UiBack,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Sfx::Fire => "fire",
            Sfx::Lash => "lash",
            Sfx::Kiss => "kiss",
            Sfx::Allure => "allure",
            Sfx::Hellgate => "hellgate",
            Sfx::Chain => "chain",
            Sfx::Gust => "gust",
            Sfx::Hit => "hit",
            Sfx::HitArmored => "hit_armored",
            Sfx::KillBeast => "kill_beast",
            Sfx::KillHuman => "kill_human",
            Sfx::KillArmored => "kill_armored",
            Sfx::Soul => "soul",
            Sfx::Hurt => "hurt",
            Sfx::Heartbeat => "heartbeat",
            Sfx::Death => "death",
            Sfx::LevelUp => "level_up",
            Sfx::UpgradePick => "upgrade_pick",
            Sfx::Horn => "horn",
            Sfx::Pause => "pause",
            Sfx::UiMove => "ui_move",
            Sfx::UiConfirm => "ui_confirm",
            Sfx::UiBack => "ui_back",
        }
    }

    /// Pitch offsets in semitones, one pre-rendered variant each.
    pub fn variants(self) -> &'static [f32] {
        match self {
            Sfx::Fire
            | Sfx::Lash
            | Sfx::Chain
            | Sfx::Gust
            | Sfx::Hit
            | Sfx::HitArmored
            | Sfx::KillBeast
            | Sfx::KillHuman
            | Sfx::KillArmored => &JITTER,
            Sfx::Hurt | Sfx::UiMove | Sfx::Kiss | Sfx::Allure | Sfx::Hellgate => &JITTER[1..4],
            Sfx::Soul => &COMBO_STEPS,
            _ => &[0.0],
        }
    }

    /// Rendered length in seconds (the same for every variant).
    pub fn length(self) -> f32 {
        match self {
            Sfx::Fire => 0.2,
            Sfx::Lash => 0.24,
            Sfx::Kiss => 0.34,
            Sfx::Allure => 0.7,
            Sfx::Hellgate => 0.5,
            Sfx::Chain => 0.3,
            Sfx::Gust => 0.4,
            Sfx::Hit => 0.1,
            Sfx::HitArmored => 0.16,
            Sfx::KillBeast => 0.16,
            Sfx::KillHuman => 0.2,
            Sfx::KillArmored => 0.45,
            Sfx::Soul => 0.18,
            Sfx::Hurt => 0.4,
            Sfx::Heartbeat => 0.5,
            Sfx::Death => 1.5,
            Sfx::LevelUp => 1.1,
            Sfx::UpgradePick => 0.8,
            Sfx::Horn => 2.4,
            Sfx::Pause => 0.6,
            Sfx::UiMove => 0.06,
            Sfx::UiConfirm | Sfx::UiBack => 0.25,
        }
    }

    /// Most copies that may sound at once.
    pub fn cap(self) -> usize {
        match self {
            Sfx::Hit | Sfx::KillBeast | Sfx::KillHuman | Sfx::KillArmored => 4,
            Sfx::Fire | Sfx::Soul | Sfx::HitArmored => 3,
            Sfx::Lash | Sfx::Kiss | Sfx::Allure | Sfx::Hellgate | Sfx::Chain | Sfx::Gust => 2,
            Sfx::UiMove | Sfx::UiConfirm | Sfx::UiBack => 2,
            _ => 1,
        }
    }

    /// Minimum seconds between two starts, so a burst doesn't stack in phase.
    pub fn gap(self) -> f32 {
        match self {
            Sfx::Hit | Sfx::UiMove => 0.03,
            Sfx::HitArmored | Sfx::KillBeast | Sfx::KillHuman | Sfx::KillArmored | Sfx::Soul => {
                0.04
            }
            Sfx::Fire | Sfx::Lash | Sfx::Kiss | Sfx::Chain | Sfx::Gust => 0.05,
            _ => 0.0,
        }
    }

    /// Playback level before the volume settings, tuned to the tiers above.
    pub fn gain(self) -> f32 {
        match self {
            Sfx::Fire => 0.09,
            Sfx::Lash => 0.12,
            Sfx::Kiss => 0.075,
            Sfx::Allure => 0.1,
            Sfx::Hellgate => 0.125,
            Sfx::Chain => 0.14,
            Sfx::Gust => 0.09,
            Sfx::Hit => 0.22,
            Sfx::HitArmored => 0.2,
            Sfx::KillBeast => 0.35,
            Sfx::KillHuman => 0.32,
            Sfx::KillArmored => 0.4,
            Sfx::Soul => 0.22,
            Sfx::Hurt => 0.8,
            Sfx::Heartbeat => 0.2,
            Sfx::Death => 0.6,
            Sfx::LevelUp => 0.55,
            Sfx::UpgradePick => 0.35,
            Sfx::Horn => 0.5,
            Sfx::Pause => 0.1,
            Sfx::UiMove => 0.07,
            Sfx::UiConfirm | Sfx::UiBack => 0.1,
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
        // Dark fireball: a soft rising hiss with a low ember thump under it,
        // gentle enough to hear every half second.
        Sfx::Fire => Patch {
            instruments: vec![
                Instrument {
                    sweep: 24.0,
                    ..env(&[3, 5, 6, 5, 4, 3, 2, 1, 1, 0])
                },
                Instrument {
                    sweep: -40.0,
                    ..env(&[9, 6, 3, 1, 0])
                },
                env(&[3, 2, 1, 0]),
            ],
            notes: vec![
                note(0.0, Noise, 82.0, 0, 1.0),
                note(0.0, Triangle, 50.0, 1, 1.0),
                note(0.02, Wave, 86.0, 2, 1.0),
            ],
            echo: dry,
        },
        // Tail Lash: air rushing up as the tail swings, then the crack on a high D
        // falling fast, so it snaps without the weight of a hit.
        Sfx::Lash => Patch {
            instruments: vec![
                Instrument {
                    sweep: 70.0,
                    ..env(&[1, 2, 4, 6, 4, 0])
                },
                with_duty(env(&[15, 9, 4, 1, 0]), 1),
                Instrument {
                    sweep: -90.0,
                    ..with_duty(env(&[9, 5, 2, 0]), 0)
                },
            ],
            notes: vec![
                note(0.0, Noise, 70.0, 0, 1.0),
                note(0.075, Noise, 104.0, 1, 1.0),
                note(0.075, Pulse1, 86.0, 2, 0.8),
            ],
            echo: dry,
        },
        // Kiss of Ruin: a breath, then a soft sine that leans up a minor 2nd
        // (D to Eb, her hook) and lingers in a short echo.
        Sfx::Kiss => Patch {
            instruments: vec![
                env(&[2, 3, 2, 1, 0]),
                Instrument {
                    pitch: Macro::new(&[-6, -3, 0, 0, 4, 8, 12, 16]),
                    ..env(&[4, 8, 10, 9, 8, 6, 5, 4, 3, 2, 1, 1, 0])
                },
                env(&[3, 2, 1, 1, 0]),
            ],
            notes: vec![
                note(0.0, Noise, 92.0, 0, 1.0),
                note(0.02, Wave, 86.0, 1, 1.0),
                note(0.02, Triangle, 74.0, 2, 0.6),
            ],
            echo: Echo {
                delay: 0.08,
                feedback: 0.25,
                mix: 0.25,
            },
        },
        // Allure: her voice scooping up into a D with a slow, coaxing vibrato, the
        // leitmotif's tritone (G#) whispered above on a thin pulse, over a breath.
        Sfx::Allure => Patch {
            instruments: vec![
                env(&[2, 4, 3, 2, 1, 0]),
                Instrument {
                    pitch: Macro::new(&[-24, -16, -10, -6, -3, -1, 0]),
                    vibrato: Vibrato {
                        depth: 0.3,
                        rate_hz: 5.0,
                        delay: 0.12,
                    },
                    ..env(&[3, 6, 9, 10, 10, 9, 8, 7, 6, 5, 4, 3, 3, 2, 2, 1, 1, 0])
                },
                with_duty(env(&[2, 4, 5, 4, 3, 2, 2, 1, 1, 0]), 0),
                env(&fade(6, 24)),
            ],
            notes: vec![
                note(0.0, Noise, 88.0, 0, 1.0),
                note(0.0, Wave, 74.0, 1, 1.0),
                note(0.0, Triangle, 50.0, 3, 0.7),
                note(0.12, Pulse2, 80.0, 2, 0.5),
            ],
            echo: Echo {
                delay: 0.11,
                feedback: 0.3,
                mix: 0.3,
            },
        },
        // Hellgate: the ground tearing open with a low whoomph, then the fire roaring
        // up out of it and settling to a crackle.
        Sfx::Hellgate => Patch {
            instruments: vec![
                Instrument {
                    sweep: -30.0,
                    ..env(&[15, 14, 11, 8, 5, 3, 1, 0])
                },
                Instrument {
                    sweep: 30.0,
                    ..env(&[2, 5, 8, 9, 8, 7, 6, 5, 4, 3, 3, 2, 2, 1, 1, 0])
                },
                with_duty(env(&[4, 0, 3, 0, 0, 2, 0, 0, 1, 0]), 1),
            ],
            notes: vec![
                note(0.0, Triangle, 40.0, 0, 1.0),
                note(0.0, Noise, 60.0, 1, 1.0),
                note(0.12, Noise, 100.0, 2, 1.0),
            ],
            echo: dry,
        },
        // Chain of Torment: links snapping taut, a bright zap that jumps a tritone
        // and falls, rattling twice more as it leaps between victims.
        Sfx::Chain => Patch {
            instruments: vec![
                with_duty(env(&[12, 6, 2, 0]), 1),
                Instrument {
                    arp: Macro::new(&[0, 6, 12, 6, 0]),
                    sweep: -48.0,
                    ..with_duty(env(&[11, 9, 7, 5, 3, 2, 1, 0]), 0)
                },
                with_duty(env(&[7, 3, 0]), 1),
            ],
            notes: vec![
                note(0.0, Noise, 104.0, 0, 1.0),
                note(0.0, Pulse1, 86.0, 1, 1.0),
                note(0.06, Noise, 108.0, 2, 1.0),
                note(0.12, Noise, 102.0, 2, 0.8),
            ],
            echo: dry,
        },
        // Wing Gust: one heavy beat of her wings, a soft thump and the air rushing
        // past and falling away.
        Sfx::Gust => Patch {
            instruments: vec![
                Instrument {
                    sweep: -40.0,
                    ..env(&[9, 6, 3, 1, 0])
                },
                Instrument {
                    sweep: -36.0,
                    ..env(&[3, 6, 8, 9, 8, 6, 5, 4, 3, 2, 2, 1, 1, 0])
                },
            ],
            notes: vec![
                note(0.0, Triangle, 43.0, 0, 1.0),
                note(0.02, Noise, 84.0, 1, 1.0),
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
        // Steel turning the fireball: two sine partials a tritone apart and a
        // metallic tick, so armour sounds different from flesh.
        Sfx::HitArmored => Patch {
            instruments: vec![
                env(&[7, 5, 4, 3, 2, 1, 1, 0]),
                with_duty(env(&[5, 2, 0]), 1),
            ],
            notes: vec![
                note(0.0, Wave, 93.0, 0, 1.0),
                note(0.0, Noise, 100.0, 1, 1.0),
                note(0.01, Pulse2, 99.0, 0, 0.35),
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
        // Veyra takes a hit, the loudest short sound in the mix so it never gets
        // lost in a crowd: a dry crack, a low body blow, and her cry, a yelp that
        // jumps up and falls away with a fast shudder, doubled a minor 2nd
        // above (the leitmotif's interval, here as pain) so it grates.
        Sfx::Hurt => {
            let cry = Instrument {
                pitch: Macro::new(&[16, 32, 24, 8, -8, -32, -56, -80, -104, -128]),
                vibrato: Vibrato {
                    depth: 0.3,
                    rate_hz: 14.0,
                    delay: 0.05,
                },
                ..env(&[12, 15, 14, 12, 10, 8, 7, 5, 4, 3, 2, 1, 0])
            };
            Patch {
                instruments: vec![
                    env(&[15, 12, 8, 5, 3, 2, 1, 0]),
                    Instrument {
                        sweep: -60.0,
                        ..env(&[15, 15, 12, 8, 4, 0])
                    },
                    cry.clone(),
                    with_duty(cry, 0),
                ],
                notes: vec![
                    note(0.0, Noise, 90.0, 0, 1.0),
                    note(0.0, Triangle, 45.0, 1, 1.0),
                    note(0.01, Pulse1, 72.0, 2, 1.0),
                    note(0.01, Pulse2, 73.0, 3, 0.6),
                ],
                echo: Echo {
                    delay: 0.08,
                    feedback: 0.25,
                    mix: 0.25,
                },
            }
        }
        // Low HP: a lub-dub on the triangle with a soft thud of noise.
        Sfx::Heartbeat => Patch {
            instruments: vec![
                Instrument {
                    sweep: -40.0,
                    ..env(&[15, 13, 9, 5, 2, 0])
                },
                env(&[4, 2, 0]),
            ],
            notes: vec![
                note(0.0, Triangle, 38.0, 0, 1.0),
                note(0.0, Noise, 30.0, 1, 1.0),
                note(0.15, Triangle, 36.0, 0, 0.65),
                note(0.15, Noise, 30.0, 1, 0.6),
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
            // The bass enters on the final note, under the resolution.
            notes.push(note(0.38, Triangle, 50.0, 2, 1.0));
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
        // Taking an upgrade: the level-up lead rushing up a D major triad, with
        // a high sine shimmer, like the power settling in.
        Sfx::UpgradePick => {
            let mut ring = vec![13, 12, 11, 10, 9, 8, 7, 6];
            ring.extend(fade(6, 16));
            let lead = with_duty(env(&ring), 1);
            let mut notes: Vec<Note> = [74.0, 78.0, 81.0, 86.0]
                .iter()
                .enumerate()
                .map(|(i, &n)| note(i as f32 * 0.04, Pulse1, n, 0, 1.0))
                .collect();
            notes.push(note(0.12, Wave, 98.0, 1, 1.0));
            Patch {
                instruments: vec![
                    lead,
                    Instrument {
                        vibrato: Vibrato {
                            depth: 0.15,
                            rate_hz: 6.0,
                            delay: 0.1,
                        },
                        ..env(&fade(8, 30))
                    },
                ],
                notes,
                echo: Echo {
                    delay: 0.12,
                    feedback: 0.35,
                    mix: 0.35,
                },
            }
        }
        // The village war horn when the knights ride in: a brassy 25% pulse that
        // scoops up from below, a fourth then the root held, parallel fifths
        // under it and a long distant echo.
        Sfx::Horn => {
            let mut swell = vec![5, 7, 9, 11, 12, 13];
            swell.extend([13; 40]);
            swell.extend(fade(13, 36));
            let horn = Instrument {
                pitch: Macro::new(&[-32, -20, -10, -4, 0]),
                vibrato: Vibrato {
                    depth: 0.15,
                    rate_hz: 5.0,
                    delay: 0.35,
                },
                ..with_duty(env(&swell), 1)
            };
            Patch {
                instruments: vec![horn.clone(), with_duty(horn, 2), env(&fade(12, 90))],
                notes: vec![
                    note(0.0, Pulse1, 45.0, 0, 1.0),
                    note(0.0, Pulse2, 38.0, 1, 0.55),
                    note(0.0, Triangle, 26.0, 2, 1.0),
                    note(0.42, Pulse1, 50.0, 0, 1.0),
                    note(0.42, Pulse2, 43.0, 1, 0.55),
                ],
                echo: Echo {
                    delay: 0.22,
                    feedback: 0.4,
                    mix: 0.35,
                },
            }
        }
        // Pausing quotes the leitmotif's b2 sigh (E flat falling to D) on a soft
        // sine with a little echo.
        Sfx::Pause => Patch {
            instruments: vec![env(&fade(9, 24))],
            notes: vec![
                note(0.0, Wave, 75.0, 0, 0.8),
                note(0.11, Wave, 74.0, 0, 1.0),
            ],
            echo: Echo {
                delay: 0.14,
                feedback: 0.3,
                mix: 0.3,
            },
        },
        // UI: round sine voices instead of thin pulses. A move is a tiny pluck
        // on A; confirm rises D to A, back falls A to D, each with a quiet
        // octave below on the triangle for warmth.
        Sfx::UiMove => Patch {
            instruments: vec![Instrument {
                pitch: Macro::new(&[4, 0]),
                ..env(&[6, 3, 1, 0])
            }],
            notes: vec![note(0.0, Wave, 81.0, 0, 1.0)],
            echo: dry,
        },
        Sfx::UiConfirm | Sfx::UiBack => {
            let (a, b) = if sfx == Sfx::UiConfirm {
                (74.0, 81.0)
            } else {
                (81.0, 74.0)
            };
            let ring = env(&[9, 8, 6, 5, 4, 3, 2, 1, 1, 0]);
            Patch {
                instruments: vec![ring, env(&[5, 3, 2, 1, 0])],
                notes: vec![
                    note(0.0, Wave, a, 0, 0.8),
                    note(0.0, Triangle, a - 12.0, 1, 1.0),
                    note(0.055, Wave, b, 0, 1.0),
                    note(0.055, Triangle, b - 12.0, 1, 1.0),
                ],
                echo: Echo {
                    delay: 0.09,
                    feedback: 0.2,
                    mix: 0.2,
                },
            }
        }
    }
}

/// Renders `sfx` shifted by `pitch` semitones, normalised to `PEAK`.
pub fn render(sfx: Sfx, pitch: f32) -> Vec<f32> {
    let mut p = patch(sfx);
    p.notes.sort_by(|a, b| a.at.total_cmp(&b.at));
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

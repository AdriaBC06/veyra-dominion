//! Renders synth test tones and every sound effect to WAV so they can be
//! listened to: `cargo run --example render_audio [out_dir]` (default
//! `target/audio`). Effects go to `sfx/`, all their pitch variants in a row.

#[allow(dead_code)]
#[path = "../src/audio/sfx.rs"]
mod sfx;
#[allow(dead_code)]
#[path = "../src/audio/synth.rs"]
mod synth;
#[allow(dead_code)]
#[path = "../src/audio/wav.rs"]
mod wav;

use std::path::PathBuf;
use synth::{Channel, Echo, Instrument, Macro, Synth, Vibrato, SAMPLE_RATE};
use Channel::*;

/// Silence between the variants of an effect in its export.
const SFX_GAP: f32 = 0.15;

const PLAIN: usize = 0;
const PLUCK: usize = 1;
const KICK: usize = 2;
const SNARE: usize = 3;
const HAT: usize = 4;
const MINOR_ARP: usize = 5;
const LEAD: usize = 6;
const ORGAN: usize = 7;
const DUTY_SWEEP: usize = 8;

fn instruments() -> Vec<Instrument> {
    let decay = |steps: &[i8]| Macro::new(steps).with_release(steps.len() - 1);
    vec![
        Instrument::default(),
        Instrument {
            volume: Macro::new(&[15, 13, 11, 10, 9, 8, 8, 7, 0]).with_release(8),
            duty: Macro::new(&[1]),
            ..Instrument::default()
        },
        Instrument {
            volume: decay(&[15, 15, 13, 10, 7, 4, 2, 0]),
            sweep: -90.0,
            ..Instrument::default()
        },
        Instrument {
            volume: decay(&[15, 12, 9, 7, 5, 3, 2, 1, 0]),
            pitch: Macro::new(&[16, 0]),
            ..Instrument::default()
        },
        Instrument {
            volume: decay(&[10, 6, 3, 0]),
            duty: Macro::new(&[1]),
            ..Instrument::default()
        },
        Instrument {
            volume: Macro::new(&[12, 0]).with_release(1),
            arp: Macro::looped(&[0, 0, 3, 3, 7, 7], 0),
            duty: Macro::new(&[0]),
            ..Instrument::default()
        },
        Instrument {
            // Swell, hold 12, then fade on release.
            volume: Macro::looped(&[13, 14, 15, 14, 13, 12, 9, 6, 3, 0], 5).with_release(6),
            pitch: Macro::new(&[-16, -8, -4, 0]),
            vibrato: Vibrato {
                depth: 0.3,
                rate_hz: 5.5,
                delay: 0.25,
            },
            duty: Macro::new(&[2, 2, 1]),
            ..Instrument::default()
        },
        Instrument {
            volume: Macro::new(&[11, 0]).with_release(1),
            wave: [
                15, 15, 14, 13, 11, 9, 8, 8, 9, 11, 12, 12, 11, 9, 7, 6, 6, 7, 8, 8, 7, 5, 3, 2, 2,
                3, 4, 4, 3, 2, 1, 0,
            ],
            ..Instrument::default()
        },
        Instrument {
            volume: Macro::new(&[12, 0]).with_release(1),
            duty: Macro::looped(&[0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 2, 2, 2, 1, 1, 1], 0),
            ..Instrument::default()
        },
    ]
}

/// (start s, channel, MIDI note, instrument, length s)
type Note = (f32, Channel, f32, usize, f32);

fn sequence(echo: Echo, notes: &[Note], length: f32) -> Vec<f32> {
    let mut s = Synth::new(SAMPLE_RATE, instruments());
    s.set_echo(echo);
    let mut events: Vec<(f32, Option<&Note>, Channel)> = Vec::new();
    for n in notes {
        events.push((n.0, Some(n), n.1));
        events.push((n.0 + n.4, None, n.1));
    }
    // Note-offs sort before note-ons at the same time, so legato notes restart.
    events.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.is_some().cmp(&b.1.is_some())));
    let mut out = Vec::new();
    let mut t = 0.0;
    for (at, on, ch) in events {
        out.extend(s.render_seconds(at - t));
        t = at;
        match on {
            Some(n) => s.note_on(ch, n.2, n.3, 1.0),
            None => s.note_off(ch),
        }
    }
    out.extend(s.render_seconds(length - t));
    out
}

/// Notes `notes` one after another, each `step` long, played for `gate` of it.
fn line(ch: Channel, inst: usize, start: f32, step: f32, gate: f32, notes: &[f32]) -> Vec<Note> {
    notes
        .iter()
        .enumerate()
        .map(|(i, &n)| (start + i as f32 * step, ch, n, inst, step * gate))
        .collect()
}

fn main() {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "target/audio".into()),
    );
    std::fs::create_dir_all(&dir).expect("create output dir");
    let echo = Echo {
        delay: 0.3,
        feedback: 0.35,
        mix: 0.3,
    };
    // D Phrygian dominant, the title theme's mode.
    let scale = [62.0, 63.0, 66.0, 67.0, 69.0, 70.0, 72.0, 74.0];

    let mut pulse_duties = Vec::new();
    for duty in 0..4 {
        let inst = Instrument {
            duty: Macro::new(&[duty]),
            volume: Macro::new(&[12, 0]).with_release(1),
            ..Instrument::default()
        };
        let mut s = Synth::new(SAMPLE_RATE, vec![inst]);
        s.note_on(Pulse1, 69.0, 0, 1.0);
        pulse_duties.extend(s.render_seconds(0.6));
        s.note_off(Pulse1);
        pulse_duties.extend(s.render_seconds(0.15));
    }

    let mut drums = Vec::new();
    for bar in 0..2 {
        let b = bar as f32 * 1.6;
        for i in 0..8 {
            drums.push((b + i as f32 * 0.2, Noise, 90.0, HAT, 0.1));
        }
        drums.extend([
            (b, Triangle, 48.0, KICK, 0.15),
            (b + 0.8, Triangle, 48.0, KICK, 0.15),
            (b + 0.4, Noise, 72.0, SNARE, 0.2),
            (b + 1.2, Noise, 72.0, SNARE, 0.2),
        ]);
    }

    let mut demo = line(
        Pulse1,
        LEAD,
        0.0,
        0.4,
        0.9,
        &[74.0, 75.0, 80.0, 79.0, 75.0, 74.0],
    );
    demo.extend(line(Triangle, PLAIN, 0.0, 0.8, 0.95, &[38.0, 38.0, 39.0]));
    demo.extend(line(Pulse2, MINOR_ARP, 0.0, 1.2, 0.95, &[62.0, 63.0]));

    let files: [(&str, Vec<f32>); 8] = [
        ("01_pulse_duties", pulse_duties),
        (
            "02_pulse_scale",
            sequence(Echo::OFF, &line(Pulse1, PLUCK, 0.0, 0.25, 0.8, &scale), 2.4),
        ),
        (
            "03_triangle_bass",
            sequence(
                Echo::OFF,
                &line(Triangle, PLAIN, 0.0, 0.25, 0.9, &scale.map(|n| n - 24.0)),
                2.2,
            ),
        ),
        ("04_noise_drums", sequence(Echo::OFF, &drums, 3.4)),
        (
            "05_wave_organ",
            sequence(
                Echo::OFF,
                &line(
                    Wave,
                    ORGAN,
                    0.0,
                    0.5,
                    0.9,
                    &[62.0, 66.0, 69.0, 70.0, 69.0, 66.0, 63.0, 62.0],
                ),
                4.2,
            ),
        ),
        (
            "06_macros",
            sequence(
                Echo::OFF,
                &[
                    (0.0, Pulse1, 62.0, MINOR_ARP, 1.5),
                    (1.6, Pulse1, 69.0, DUTY_SWEEP, 1.5),
                    (3.2, Pulse1, 74.0, LEAD, 1.6),
                ],
                5.2,
            ),
        ),
        (
            "07_echo",
            sequence(
                echo,
                &line(Pulse1, PLUCK, 0.0, 0.6, 0.3, &[74.0, 75.0, 80.0, 74.0]),
                4.0,
            ),
        ),
        ("08_all_voices", sequence(echo, &demo, 3.4)),
    ];
    let sfx_dir = dir.join("sfx");
    std::fs::create_dir_all(&sfx_dir).expect("create sfx dir");
    let gap = vec![0.0; (SFX_GAP * SAMPLE_RATE as f32) as usize];
    let effects = sfx::Sfx::ALL.map(|e| {
        let mut out = Vec::new();
        for &pitch in e.variants() {
            out.extend(sfx::render(e, pitch).iter().map(|s| s * e.gain()));
            out.extend(&gap);
        }
        (format!("sfx/{}", e.name()), out)
    });

    let files = files.into_iter().map(|(n, s)| (n.to_owned(), s));
    for (name, samples) in files.chain(effects) {
        let path = dir.join(format!("{name}.wav"));
        std::fs::write(&path, wav::encode(&samples, SAMPLE_RATE)).expect("write wav");
        println!(
            "{} ({:.1} s)",
            path.display(),
            samples.len() as f32 / SAMPLE_RATE as f32
        );
    }
}

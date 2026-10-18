//! Chip synthesizer: NES-style voices (2 pulse, triangle, noise) plus a 4-bit
//! wavetable voice, FamiTracker-style instrument macros and a light SNES-style echo.
//! Renders mono f32 PCM deterministically. It only depends on `std`, so the
//! `render_audio` example includes this file directly.

pub const SAMPLE_RATE: u32 = 44_100;
/// Instrument macros advance once per tick, like a 60 Hz NES sound driver.
pub const TICK_RATE: f32 = 60.0;
pub const DUTIES: [f32; 4] = [0.125, 0.25, 0.5, 0.75];
/// Duty macro index used when an instrument has no duty macro (50%).
const DEFAULT_DUTY: i8 = 2;
/// The noise LFSR is clocked at the note frequency times this.
const NOISE_CLOCK_MUL: f32 = 16.0;
/// At most this many LFSR clocks per sample (very high noise notes).
const MAX_NOISE_STEPS: f32 = 32.0;
/// Mix weight per channel, in `Channel` order; full volume on all five stays below 1.
const CHANNEL_GAIN: [f32; 5] = [0.2, 0.2, 0.26, 0.16, 0.18];
/// One-pole DC blocker (~35 Hz) and low-pass cutoff, like the NES output stage.
const DC_BLOCK_R: f32 = 0.995;
const LOWPASS_HZ: f32 = 12_000.0;
/// Echo feedback is darkened by this one-pole low-pass, as on the SNES.
const ECHO_DAMP_HZ: f32 = 3_000.0;

/// A rounded 4-bit sine, the default wavetable.
pub const SINE_WAVE: [u8; 32] = [
    8, 9, 10, 12, 13, 14, 14, 15, 15, 15, 14, 14, 13, 12, 10, 9, 7, 6, 5, 3, 2, 1, 1, 0, 0, 0, 1,
    1, 2, 3, 5, 6,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Pulse1,
    Pulse2,
    Triangle,
    Noise,
    Wave,
}

impl Channel {
    pub const ALL: [Channel; 5] = [
        Channel::Pulse1,
        Channel::Pulse2,
        Channel::Triangle,
        Channel::Noise,
        Channel::Wave,
    ];
}

/// A per-tick value sequence. Without a release point the whole sequence is the
/// sustain part; with one, `note_off` jumps to it and plays the rest. When a part
/// ends it jumps back to `loop_from` if that lies inside the part, else it holds
/// the last value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Macro {
    pub steps: Vec<i8>,
    pub loop_from: Option<usize>,
    pub release: Option<usize>,
}

impl Macro {
    pub fn new(steps: &[i8]) -> Self {
        Self {
            steps: steps.to_vec(),
            ..Self::default()
        }
    }

    pub fn looped(steps: &[i8], from: usize) -> Self {
        Self {
            loop_from: Some(from),
            ..Self::new(steps)
        }
    }

    pub fn with_release(mut self, at: usize) -> Self {
        self.release = Some(at);
        self
    }

    fn value_or(&self, pos: usize, default: i8) -> i8 {
        match self.steps.len() {
            0 => default,
            n => self.steps[pos.min(n - 1)],
        }
    }

    fn next(&self, pos: usize, released: bool) -> usize {
        let len = self.steps.len();
        let (start, end) = match (released, self.release) {
            (true, Some(r)) => (r, len),
            (_, r) => (0, r.unwrap_or(len)),
        };
        if pos + 1 < end {
            return pos + 1;
        }
        match self.loop_from {
            Some(l) if l >= start && l < end => l,
            _ => pos.min(end.saturating_sub(1)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vibrato {
    /// Peak offset in semitones.
    pub depth: f32,
    pub rate_hz: f32,
    /// Seconds after note-on before the vibrato starts.
    pub delay: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instrument {
    /// 0..=15; an empty macro means 15.
    pub volume: Macro,
    /// Semitone offsets (absolute, not accumulated).
    pub arp: Macro,
    /// Offsets in 1/16 semitone.
    pub pitch: Macro,
    /// `DUTIES` index on the pulse voices; 0 long / 1 short LFSR on the noise voice.
    pub duty: Macro,
    pub vibrato: Vibrato,
    /// Pitch slide in semitones per second from note-on.
    pub sweep: f32,
    /// 4-bit samples (0..=15) for the wave voice.
    pub wave: [u8; 32],
}

impl Default for Instrument {
    fn default() -> Self {
        Self {
            volume: Macro::default(),
            arp: Macro::default(),
            pitch: Macro::default(),
            duty: Macro::default(),
            vibrato: Vibrato::default(),
            sweep: 0.0,
            wave: SINE_WAVE,
        }
    }
}

/// Echo in the output stage; `mix = 0` turns it off.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Echo {
    pub delay: f32,
    pub feedback: f32,
    pub mix: f32,
}

impl Echo {
    pub const OFF: Echo = Echo {
        delay: 0.0,
        feedback: 0.0,
        mix: 0.0,
    };
}

pub fn midi_to_hz(note: f32) -> f32 {
    440.0 * 2f32.powf((note - 69.0) / 12.0)
}

#[derive(Clone, Copy, Debug, Default)]
struct MacroPos {
    volume: usize,
    arp: usize,
    pitch: usize,
    duty: usize,
}

#[derive(Clone, Debug)]
struct Voice {
    channel: Channel,
    active: bool,
    released: bool,
    inst: usize,
    note: f32,
    velocity: f32,
    pos: MacroPos,
    /// Seconds since note-on, advanced per tick.
    age: f32,
    /// Samples left until the next macro tick.
    until_tick: f32,
    // Derived per tick.
    level: f32,
    step: f32,
    duty: f32,
    short_noise: bool,
    // Oscillator state.
    phase: f32,
    lfsr: u16,
}

impl Voice {
    fn new(channel: Channel) -> Self {
        Self {
            channel,
            active: false,
            released: false,
            inst: 0,
            note: 0.0,
            velocity: 0.0,
            pos: MacroPos::default(),
            age: 0.0,
            until_tick: 0.0,
            level: 0.0,
            step: 0.0,
            duty: 0.5,
            short_noise: false,
            phase: 0.0,
            lfsr: 1,
        }
    }

    /// Recomputes level, frequency and duty from the current macro positions.
    fn refresh(&mut self, inst: &Instrument, sample_rate: f32) {
        let p = self.pos;
        self.level =
            f32::from(inst.volume.value_or(p.volume, 15).clamp(0, 15)) / 15.0 * self.velocity;
        let v = inst.vibrato;
        let vibrato = if v.depth > 0.0 && self.age >= v.delay {
            v.depth * (std::f32::consts::TAU * v.rate_hz * (self.age - v.delay)).sin()
        } else {
            0.0
        };
        let semis = self.note
            + f32::from(inst.arp.value_or(p.arp, 0))
            + f32::from(inst.pitch.value_or(p.pitch, 0)) / 16.0
            + inst.sweep * self.age
            + vibrato;
        let hz = midi_to_hz(semis);
        let duty = inst.duty.value_or(p.duty, DEFAULT_DUTY);
        self.duty = DUTIES[duty.clamp(0, 3) as usize];
        self.short_noise = duty == 1;
        self.step = if self.channel == Channel::Noise {
            (hz * NOISE_CLOCK_MUL / sample_rate).min(MAX_NOISE_STEPS)
        } else {
            hz / sample_rate
        };
    }

    fn advance(&mut self, inst: &Instrument) {
        let r = self.released;
        self.pos.volume = inst.volume.next(self.pos.volume, r);
        self.pos.arp = inst.arp.next(self.pos.arp, r);
        self.pos.pitch = inst.pitch.next(self.pos.pitch, r);
        self.pos.duty = inst.duty.next(self.pos.duty, r);
        self.age += 1.0 / TICK_RATE;
    }

    fn clock_lfsr(&mut self) {
        let tap = if self.short_noise { 6 } else { 1 };
        let bit = (self.lfsr ^ (self.lfsr >> tap)) & 1;
        self.lfsr = (self.lfsr >> 1) | (bit << 14);
    }

    /// Raw oscillator output in -1..=1, before the volume level.
    fn oscillator(&mut self, inst: &Instrument) -> f32 {
        let out = match self.channel {
            Channel::Pulse1 | Channel::Pulse2 => {
                if self.phase < self.duty {
                    1.0
                } else {
                    -1.0
                }
            }
            Channel::Triangle => {
                let i = (self.phase * 32.0) as u8 & 31;
                let level = if i < 16 { 15 - i } else { i - 16 };
                f32::from(level) / 7.5 - 1.0
            }
            Channel::Noise => {
                if self.lfsr & 1 == 0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Channel::Wave => {
                let i = (self.phase * 32.0) as usize & 31;
                f32::from(inst.wave[i].min(15)) / 7.5 - 1.0
            }
        };
        self.phase += self.step;
        if self.channel == Channel::Noise {
            while self.phase >= 1.0 {
                self.phase -= 1.0;
                self.clock_lfsr();
            }
        } else {
            self.phase = self.phase.fract();
        }
        out
    }
}

pub struct Synth {
    sample_rate: f32,
    instruments: Vec<Instrument>,
    voices: [Voice; 5],
    echo: Echo,
    echo_buf: Vec<f32>,
    echo_pos: usize,
    echo_damp: f32,
    echo_damp_a: f32,
    dc_x: f32,
    dc_y: f32,
    lp_y: f32,
    lp_a: f32,
}

impl Synth {
    pub fn new(sample_rate: u32, instruments: Vec<Instrument>) -> Self {
        let sr = sample_rate as f32;
        let one_pole = |hz: f32| 1.0 - (-std::f32::consts::TAU * hz / sr).exp();
        Self {
            sample_rate: sr,
            instruments,
            voices: Channel::ALL.map(Voice::new),
            echo: Echo::OFF,
            echo_buf: Vec::new(),
            echo_pos: 0,
            echo_damp: 0.0,
            echo_damp_a: one_pole(ECHO_DAMP_HZ),
            dc_x: 0.0,
            dc_y: 0.0,
            lp_y: 0.0,
            lp_a: one_pole(LOWPASS_HZ),
        }
    }

    /// Replaces the echo settings and clears the echo buffer.
    pub fn set_echo(&mut self, echo: Echo) {
        let len = ((echo.delay * self.sample_rate) as usize).max(1);
        self.echo = echo;
        self.echo_buf = vec![0.0; len];
        self.echo_pos = 0;
        self.echo_damp = 0.0;
    }

    /// Starts `note` (MIDI number, fractional allowed) with instrument `inst`;
    /// `velocity` scales the volume macro (0..=1).
    pub fn note_on(&mut self, ch: Channel, note: f32, inst: usize, velocity: f32) {
        assert!(inst < self.instruments.len(), "unknown instrument {inst}");
        let v = &mut self.voices[ch as usize];
        v.active = true;
        v.released = false;
        v.inst = inst;
        v.note = note;
        v.velocity = velocity.clamp(0.0, 1.0);
        v.pos = MacroPos::default();
        v.age = 0.0;
        v.until_tick = self.sample_rate / TICK_RATE;
        v.refresh(&self.instruments[inst], self.sample_rate);
    }

    /// Jumps every macro with a release point to it; if the volume macro has
    /// none, the note is cut instead.
    pub fn note_off(&mut self, ch: Channel) {
        let v = &mut self.voices[ch as usize];
        if !v.active {
            return;
        }
        let inst = &self.instruments[v.inst];
        if inst.volume.release.is_none() {
            v.active = false;
            return;
        }
        v.released = true;
        let jump = |m: &Macro, pos: &mut usize| {
            if let Some(r) = m.release {
                *pos = r;
            }
        };
        jump(&inst.volume, &mut v.pos.volume);
        jump(&inst.arp, &mut v.pos.arp);
        jump(&inst.pitch, &mut v.pos.pitch);
        jump(&inst.duty, &mut v.pos.duty);
        v.refresh(inst, self.sample_rate);
    }

    pub fn note_cut(&mut self, ch: Channel) {
        self.voices[ch as usize].active = false;
    }

    pub fn render(&mut self, out: &mut [f32]) {
        let tick_len = self.sample_rate / TICK_RATE;
        for s in out.iter_mut() {
            let mut mix = 0.0;
            for (v, gain) in self.voices.iter_mut().zip(CHANNEL_GAIN) {
                if !v.active {
                    continue;
                }
                let inst = &self.instruments[v.inst];
                v.until_tick -= 1.0;
                if v.until_tick <= 0.0 {
                    v.until_tick += tick_len;
                    v.advance(inst);
                    v.refresh(inst, self.sample_rate);
                }
                mix += v.oscillator(inst) * v.level * gain;
            }
            *s = self.output_stage(mix);
        }
    }

    pub fn render_seconds(&mut self, seconds: f32) -> Vec<f32> {
        let mut out = vec![0.0; (seconds * self.sample_rate) as usize];
        self.render(&mut out);
        out
    }

    fn output_stage(&mut self, x: f32) -> f32 {
        let mut y = x;
        if self.echo.mix > 0.0 && !self.echo_buf.is_empty() {
            let delayed = self.echo_buf[self.echo_pos];
            self.echo_damp += (delayed - self.echo_damp) * self.echo_damp_a;
            self.echo_buf[self.echo_pos] = x + self.echo_damp * self.echo.feedback;
            self.echo_pos = (self.echo_pos + 1) % self.echo_buf.len();
            y += delayed * self.echo.mix;
        }
        self.dc_y = y - self.dc_x + DC_BLOCK_R * self.dc_y;
        self.dc_x = y;
        self.lp_y += (self.dc_y - self.lp_y) * self.lp_a;
        self.lp_y.clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 44_100;

    fn rms(s: &[f32]) -> f32 {
        (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
    }

    /// Rising zero crossings per second.
    fn freq(s: &[f32]) -> f32 {
        let n = s.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count();
        n as f32 * SR as f32 / s.len() as f32
    }

    fn one(ch: Channel, note: f32, inst: Instrument, secs: f32) -> Vec<f32> {
        let mut s = Synth::new(SR, vec![inst]);
        s.note_on(ch, note, 0, 1.0);
        s.render_seconds(secs)
    }

    #[test]
    fn macro_hold_loop_and_release() {
        let hold = Macro::new(&[1, 2, 3]);
        assert_eq!(hold.next(0, false), 1);
        assert_eq!(hold.next(2, false), 2);
        let looped = Macro::looped(&[1, 2, 3], 1);
        assert_eq!(looped.next(2, false), 1);
        // Sustain is 0..2, release part 2..4 holds its end.
        let rel = Macro::looped(&[9, 8, 4, 0], 0).with_release(2);
        assert_eq!(rel.next(1, false), 0);
        assert_eq!(rel.next(2, true), 3);
        assert_eq!(rel.next(3, true), 3);
        assert_eq!(Macro::default().value_or(5, 7), 7);
        assert_eq!(Macro::default().next(0, false), 0);
    }

    #[test]
    fn silent_without_notes_and_deterministic() {
        let mut s = Synth::new(SR, vec![Instrument::default()]);
        assert!(s.render_seconds(0.1).iter().all(|&x| x == 0.0));
        let a = one(Channel::Noise, 60.0, Instrument::default(), 0.2);
        let b = one(Channel::Noise, 60.0, Instrument::default(), 0.2);
        assert_eq!(a, b);
    }

    #[test]
    fn pitch_follows_note_on_every_tonal_voice() {
        for ch in [
            Channel::Pulse1,
            Channel::Pulse2,
            Channel::Triangle,
            Channel::Wave,
        ] {
            let out = one(ch, 69.0, Instrument::default(), 1.0);
            let f = freq(&out[4410..]);
            assert!((f - 440.0).abs() < 5.0, "{ch:?}: {f}");
        }
    }

    #[test]
    fn pulse_duty_sets_high_fraction() {
        for (i, &d) in DUTIES.iter().enumerate() {
            let mut v = Voice::new(Channel::Pulse1);
            v.velocity = 1.0;
            let inst = Instrument {
                duty: Macro::new(&[i as i8]),
                ..Instrument::default()
            };
            v.refresh(&inst, SR as f32);
            v.step = 1.0 / 400.0;
            let high = (0..4000).filter(|_| v.oscillator(&inst) > 0.0).count();
            assert!((high as f32 / 4000.0 - d).abs() < 0.01, "duty {d}");
        }
    }

    #[test]
    fn triangle_is_four_bit() {
        let mut v = Voice::new(Channel::Triangle);
        v.step = 1.0 / 320.0;
        let inst = Instrument::default();
        let mut levels: Vec<i32> = (0..320)
            .map(|_| (v.oscillator(&inst) * 7.5).round() as i32)
            .collect();
        levels.sort();
        levels.dedup();
        assert_eq!(levels.len(), 16);
    }

    #[test]
    fn wave_voice_plays_its_table() {
        let mut table = [0; 32];
        table[16..].fill(15);
        let inst = Instrument {
            wave: table,
            ..Instrument::default()
        };
        let mut v = Voice::new(Channel::Wave);
        v.step = 1.0 / 64.0;
        let out: Vec<f32> = (0..64).map(|_| v.oscillator(&inst)).collect();
        assert!(out[..32].iter().all(|&x| x == -1.0));
        assert!(out[32..].iter().all(|&x| x == 1.0));
    }

    #[test]
    fn noise_lfsr_periods() {
        for (short, period) in [(false, 32_767), (true, 93)] {
            let mut v = Voice::new(Channel::Noise);
            v.short_noise = short;
            // Settle onto the cycle first: short mode from seed 1 can start off it.
            for _ in 0..40_000 {
                v.clock_lfsr();
            }
            let start = v.lfsr;
            let mut n = 0;
            loop {
                v.clock_lfsr();
                n += 1;
                if v.lfsr == start {
                    break;
                }
            }
            assert_eq!(n, period, "short {short}");
        }
    }

    #[test]
    fn volume_macro_and_release() {
        let inst = Instrument {
            volume: Macro::new(&[15, 15, 8, 8, 0]).with_release(3),
            ..Instrument::default()
        };
        let mut s = Synth::new(SR, vec![inst]);
        s.note_on(Channel::Pulse1, 60.0, 0, 1.0);
        let early = s.render_seconds(0.03);
        let sustain = s.render_seconds(0.2);
        assert!(rms(&early) > rms(&sustain[4410..]) * 1.5);
        assert!(rms(&sustain[4410..]) > 0.01, "holds at 8 before release");
        s.note_off(Channel::Pulse1);
        let after = s.render_seconds(0.2);
        assert!(rms(&after[4410..]) < 1e-3, "release goes to 0");
    }

    #[test]
    fn note_off_without_release_cuts() {
        let mut s = Synth::new(SR, vec![Instrument::default()]);
        s.note_on(Channel::Triangle, 45.0, 0, 1.0);
        s.render_seconds(0.1);
        s.note_off(Channel::Triangle);
        let after = s.render_seconds(0.3);
        assert!(rms(&after[4410..]) < 1e-3);
    }

    #[test]
    fn arp_and_sweep_move_pitch() {
        let arp = Instrument {
            arp: Macro::new(&[12]),
            ..Instrument::default()
        };
        let f = freq(&one(Channel::Pulse1, 57.0, arp, 1.0)[4410..]);
        assert!((f - 440.0).abs() < 5.0, "arp +12: {f}");
        let sweep = Instrument {
            sweep: -12.0,
            ..Instrument::default()
        };
        let out = one(Channel::Pulse1, 81.0, sweep, 1.0);
        let (a, b) = (freq(&out[..11_025]), freq(&out[33_075..]));
        assert!(a > b * 1.5, "sweep down: {a} -> {b}");
    }

    #[test]
    fn vibrato_wobbles_after_delay() {
        let inst = Instrument {
            vibrato: Vibrato {
                depth: 1.0,
                rate_hz: 2.0,
                delay: 0.5,
            },
            ..Instrument::default()
        };
        let mut v = Voice::new(Channel::Pulse1);
        v.note = 69.0;
        v.refresh(&inst, SR as f32);
        let base = v.step;
        v.age = 0.625; // quarter cycle after the delay: peak
        v.refresh(&inst, SR as f32);
        assert!((v.step / base - 2f32.powf(1.0 / 12.0)).abs() < 1e-3);
    }

    #[test]
    fn echo_repeats_after_the_note() {
        let render = |echo: Echo| {
            let mut s = Synth::new(SR, vec![Instrument::default()]);
            s.set_echo(echo);
            s.note_on(Channel::Pulse1, 69.0, 0, 1.0);
            s.render_seconds(0.1);
            s.note_cut(Channel::Pulse1);
            s.render_seconds(0.3)
        };
        let dry = render(Echo::OFF);
        let wet = render(Echo {
            delay: 0.25,
            feedback: 0.3,
            mix: 0.4,
        });
        // 0.15..0.2 s after the cut is 0.25..0.3 s after note-on: the echo.
        let window = 6615..8820;
        assert!(rms(&dry[window.clone()]) < 1e-3);
        assert!(rms(&wet[window]) > 0.02);
    }

    #[test]
    fn full_mix_stays_in_range() {
        let mut s = Synth::new(SR, vec![Instrument::default()]);
        s.set_echo(Echo {
            delay: 0.1,
            feedback: 0.9,
            mix: 1.0,
        });
        for ch in Channel::ALL {
            s.note_on(ch, 40.0, 0, 1.0);
        }
        let out = s.render_seconds(1.0);
        assert!(out.iter().all(|x| (-1.0..=1.0).contains(x)));
        assert!(rms(&out) > 0.1);
    }
}

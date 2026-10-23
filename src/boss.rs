//! Bosses (docs/design.md 7.8, V19): a phase state machine over one body in the horde.
//! The body is an `Enemies` row of a `Behavior::Boss` kind, so every Art hits it as usual;
//! the phases, attacks and timers live here. Pure logic like the rest of the sim.

use crate::config::*;
use crate::enemies::{Action, Enemies, MODE_DASH, MODE_GUARD, MODE_WALK, MODE_WINDUP};
use crate::game::Cue;
use macroquad::math::Vec2;
use macroquad::rand::gen_range;
use std::f32::consts::TAU;

/// What the boss is doing; `Boss::timer` counts down the time left in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossMode {
    Walk,
    /// Planted before a rush, tracking her until `charge_lock` is left (the telegraph).
    Windup,
    Charge,
    /// Shield raised to bash her when it runs out.
    Bash,
    /// Arm raised, about to lob the torches.
    Volley,
    Guard,
    /// Blowing the horn; the troops arrive when it runs out.
    Call,
    Rest,
}

pub struct Boss {
    pub def: &'static BossDef,
    /// Stable id of its body in `Enemies`.
    pub uid: u32,
    pub phase: usize,
    pub mode: BossMode,
    pub timer: f32,
    /// Rush direction (or bash direction), unit length.
    pub dir: Vec2,
    /// Which way he faces (+1 right, -1 left); turns only past `BOSS_TURN_DEADZONE`, so he
    /// doesn't flicker with her straight above or below him.
    pub facing: f32,
    /// Mirrors of the body, for the HUD and the renderer.
    pub pos: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    /// Index of the next attack in the phase's rotation.
    next: usize,
    /// Rushes left after the current one.
    chain: u8,
    bash_ready: f32,
    /// Who the horn is calling.
    calling: &'static [Muster],
    volley: u8,
}

impl Boss {
    /// Puts its body in the horde at `pos` with `hp_scale` on its health.
    pub fn spawn(def: &'static BossDef, enemies: &mut Enemies, pos: Vec2, hp_scale: f32) -> Self {
        enemies.spawn(def.kind, pos, hp_scale);
        let i = enemies.len() - 1;
        Self {
            def,
            uid: enemies.uid[i],
            phase: 0,
            mode: BossMode::Walk,
            timer: def.phases[0].gap,
            dir: Vec2::NEG_X,
            facing: -1.0,
            pos,
            hp: enemies.hp[i],
            max_hp: enemies.max_hp[i],
            next: 0,
            chain: 0,
            bash_ready: 0.0,
            calling: &[],
            volley: 0,
        }
    }

    fn phase(&self) -> &'static Phase {
        &self.def.phases[self.phase]
    }

    /// One step toward `target` (Veyra): phase changes, the attack in progress, the next
    /// one. Its blows, torches and calls go to `enemies.actions`; returns the cue of an
    /// attack that started, if any, or `None` with the body gone (it died).
    pub fn update(
        &mut self,
        enemies: &mut Enemies,
        target: Vec2,
        dt: f32,
        speed_scale: f32,
    ) -> Option<Option<Cue>> {
        let i = enemies.find(self.uid)?;
        let def = self.def;
        let body = &ENEMY_DEFS[def.kind];
        self.pos = enemies.pos(i);
        (self.hp, self.max_hp) = (enemies.hp[i], enemies.max_hp[i]);
        self.timer -= dt;
        self.bash_ready -= dt;
        let mut cue = None;
        // Each threshold crossed interrupts whatever he was doing with a rally.
        while let Some(next) = def.phases.get(self.phase + 1) {
            if self.hp > self.max_hp * next.below {
                break;
            }
            self.phase += 1;
            self.next = 0;
            self.chain = 0;
            cue = self.call(next.rally);
        }
        let off = target - self.pos;
        let (dist, aim) = (off.length(), off.normalize_or(self.dir));
        let speed = body.speed * self.phase().speed * speed_scale;
        let mut step = Vec2::ZERO;
        match self.mode {
            BossMode::Walk => {
                step = aim * speed;
                if self.bash_ready <= 0.0 && dist < def.bash_range {
                    self.mode = BossMode::Bash;
                    self.timer = def.bash_windup;
                    self.dir = aim;
                } else if self.timer <= 0.0 {
                    let rotation = self.phase().rotation;
                    let attack = rotation[self.next % rotation.len()];
                    self.next = (self.next + 1) % rotation.len();
                    cue = self.begin(attack, aim).or(cue);
                }
            }
            BossMode::Windup => {
                if self.timer > def.charge_lock {
                    self.dir = aim;
                }
                if self.timer <= 0.0 {
                    self.mode = BossMode::Charge;
                    self.timer = def.charge_time;
                    cue = Some(Cue::Charge);
                }
            }
            BossMode::Charge => {
                step = self.dir * def.charge_speed;
                enemies.actions.push(Action::Strike {
                    pos: self.pos,
                    radius: body.radius(),
                    damage: def.charge_damage,
                });
                if self.timer <= 0.0 {
                    if self.chain > 0 {
                        self.chain -= 1;
                        self.mode = BossMode::Windup;
                        self.timer = self.phase().windup * 0.5 + def.charge_rest * 0.5;
                    } else {
                        self.rest(def.charge_rest);
                    }
                }
            }
            BossMode::Bash => {
                if self.timer <= 0.0 {
                    enemies.actions.push(Action::Strike {
                        pos: self.pos + self.dir * body.radius(),
                        radius: def.bash_radius,
                        damage: def.bash_damage,
                    });
                    self.bash_ready = def.bash_cooldown;
                    cue = Some(Cue::Bash);
                    self.rest(def.bash_windup);
                }
            }
            BossMode::Volley => {
                if self.timer <= 0.0 {
                    for k in 0..self.volley {
                        let scatter = if k == 0 {
                            Vec2::ZERO
                        } else {
                            Vec2::from_angle(gen_range(0.0, TAU))
                                * gen_range(def.volley_spread * 0.3, def.volley_spread)
                        };
                        enemies.actions.push(Action::Shot {
                            kind: ShotKind::Torch,
                            from: self.pos,
                            to: target + scatter,
                            damage: def.volley_damage,
                        });
                    }
                    self.walk();
                }
            }
            BossMode::Guard => {
                step = aim * speed * def.guard_speed;
                if self.timer <= 0.0 {
                    self.walk();
                }
            }
            BossMode::Call => {
                if self.timer <= 0.0 {
                    for &m in self.calling {
                        enemies.actions.push(Action::Call(m));
                    }
                    self.calling = &[];
                    self.walk();
                }
            }
            BossMode::Rest => {
                if self.timer <= 0.0 {
                    self.walk();
                }
            }
        }
        self.pos += step * dt;
        enemies.set_pos(i, self.pos);
        let toward = match self.mode {
            BossMode::Windup | BossMode::Charge | BossMode::Bash => self.dir.x * dist,
            _ => off.x,
        };
        if toward.abs() > BOSS_TURN_DEADZONE {
            self.facing = toward.signum();
        }
        enemies.mode[i] = match self.mode {
            BossMode::Guard => MODE_GUARD,
            BossMode::Charge => MODE_DASH,
            BossMode::Windup | BossMode::Bash => MODE_WINDUP,
            _ => MODE_WALK,
        };
        Some(cue)
    }

    /// Starts `attack`, facing `aim`.
    fn begin(&mut self, attack: Attack, aim: Vec2) -> Option<Cue> {
        self.dir = aim;
        match attack {
            Attack::Charge { chain } => {
                self.mode = BossMode::Windup;
                self.timer = self.phase().windup;
                self.chain = chain.saturating_sub(1);
                None
            }
            Attack::Volley { n } => {
                self.mode = BossMode::Volley;
                self.timer = self.def.volley_windup;
                self.volley = n;
                None
            }
            Attack::Guard => {
                self.mode = BossMode::Guard;
                self.timer = self.def.guard_time;
                Some(Cue::Guard)
            }
            Attack::Call(troops) => self.call(troops),
        }
    }

    /// Blows the horn for `troops` (nothing when there are none).
    fn call(&mut self, troops: &'static [Muster]) -> Option<Cue> {
        if troops.is_empty() {
            return None;
        }
        self.mode = BossMode::Call;
        self.timer = self.def.call_time;
        self.calling = troops;
        Some(Cue::Horn)
    }

    fn rest(&mut self, secs: f32) {
        self.mode = BossMode::Rest;
        self.timer = secs;
    }

    fn walk(&mut self) {
        self.mode = BossMode::Walk;
        self.timer = self.phase().gap;
    }

    /// Health left, 0..1, for the HUD bar.
    pub fn health(&self) -> f32 {
        (self.hp / self.max_hp).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, InputState, SimEvent};
    use macroquad::math::vec2;

    /// No spawns, no Arts, an unkillable Veyra at the origin and the Captain `at`.
    fn duel(at: Vec2) -> Game {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.arts.hold();
        g.player.hp = f32::INFINITY;
        g.spawn_boss(&CAPTAIN_DEF);
        g.enemies.set_pos(0, at);
        g
    }

    fn run(g: &mut Game, ticks: usize) {
        for _ in 0..ticks {
            g.tick(&InputState::default());
        }
    }

    fn boss(g: &Game) -> &Boss {
        g.boss.as_ref().expect("the captain lives")
    }

    /// Ticks until the boss is in `mode` (at most `secs`).
    fn until(g: &mut Game, mode: BossMode, secs: f32) {
        for _ in 0..(secs / TICK_DT) as usize {
            run(g, 1);
            if boss(g).mode == mode {
                return;
            }
        }
        panic!("never got to {mode:?}, is {:?}", boss(g).mode);
    }

    #[test]
    fn the_captain_closes_the_timeline() {
        let last = EVENTS.last().expect("events");
        assert_eq!(last.time, MAP_TIME);
        assert!(matches!(last.kind, EventKind::Boss(d) if d.kind == CAPTAIN));
        let thresholds: Vec<f32> = CAPTAIN_DEF.phases.iter().map(|p| p.below).collect();
        assert_eq!(thresholds[0], 1.0);
        assert!(thresholds.windows(2).all(|w| w[1] < w[0]), "phases go down");
    }

    #[test]
    fn he_plants_himself_then_charges_down_a_locked_lane() {
        let mut g = duel(vec2(200.0, 0.0));
        until(&mut g, BossMode::Windup, 4.0);
        let start = boss(&g).pos;
        let windup = CAPTAIN_DEF.phases[0].windup;
        // She steps aside before the lane locks; it follows her, then holds.
        g.player.pos = vec2(0.0, 60.0);
        run(
            &mut g,
            ((windup - CAPTAIN_DEF.charge_lock) / TICK_DT) as usize + 2,
        );
        let locked = boss(&g).dir;
        assert!(locked.y > 0.2, "the lane turned toward her");
        assert!(
            boss(&g).pos.distance(start) < 1.0,
            "planted while winding up"
        );
        g.player.pos = vec2(0.0, -60.0);
        until(&mut g, BossMode::Charge, 1.0);
        assert_eq!(boss(&g).dir, locked, "too late to re-aim");
        let before = boss(&g).pos;
        run(&mut g, 1);
        let step = boss(&g).pos.distance(before);
        assert!(step > CAPTAIN_DEF.charge_speed * TICK_DT * 0.9, "rushes");
    }

    #[test]
    fn the_charge_hurts_her_in_its_lane() {
        let mut g = duel(vec2(200.0, 0.0));
        g.player.hp = 1000.0;
        g.player.max_hp = 1000.0;
        until(&mut g, BossMode::Charge, 4.0);
        run(&mut g, (CAPTAIN_DEF.charge_time / TICK_DT) as usize);
        assert!(g.player.hp <= 1000.0 - CAPTAIN_DEF.charge_damage + PLAYER_IFRAMES);
    }

    #[test]
    fn he_only_turns_once_she_is_clearly_on_the_other_side() {
        let mut g = duel(vec2(100.0, 0.0));
        run(&mut g, 1);
        assert_eq!(boss(&g).facing, -1.0, "faces her, to his left");
        // Straight above him, wobbling across his centre line: he holds his facing.
        for k in 0..20 {
            let x = boss(&g).pos.x + if k % 2 == 0 { 3.0 } else { -3.0 };
            g.player.pos = vec2(x, -150.0);
            run(&mut g, 1);
            assert_eq!(boss(&g).facing, -1.0);
        }
        g.player.pos = vec2(boss(&g).pos.x + 60.0, -150.0);
        run(&mut g, 1);
        assert_eq!(boss(&g).facing, 1.0, "turns once she is well past");
    }

    #[test]
    fn too_close_and_he_bashes_with_the_shield() {
        let mut g = duel(vec2(30.0, 0.0));
        g.player.hp = 1000.0;
        g.player.max_hp = 1000.0;
        run(&mut g, 1);
        assert_eq!(boss(&g).mode, BossMode::Bash, "winds up the bash at once");
        assert!(g.player.hp == 1000.0, "the wind-up is a warning");
        run(&mut g, (CAPTAIN_DEF.bash_windup / TICK_DT) as usize + 2);
        assert!(g.player.hp < 1000.0, "then it lands");
        assert_ne!(boss(&g).mode, BossMode::Bash);
    }

    #[test]
    fn he_calls_the_militia() {
        let mut g = duel(vec2(250.0, 0.0));
        until(&mut g, BossMode::Call, 12.0);
        assert!(g.cues.contains(&Cue::Horn));
        let before = g.enemies.len();
        run(&mut g, (CAPTAIN_DEF.call_time / TICK_DT) as usize + 2);
        let Attack::Call(troops) = CAPTAIN_DEF.phases[0].rotation[1] else {
            panic!("phase one calls second");
        };
        let called: usize = troops.iter().map(|m| m.n as usize).sum();
        assert_eq!(g.enemies.len(), before + called);
    }

    #[test]
    fn wounds_open_new_phases_with_a_rally() {
        let mut g = duel(vec2(250.0, 0.0));
        let max = g.enemies.max_hp[0];
        g.enemies.hp[0] = max * 0.6;
        run(&mut g, 1);
        assert_eq!(boss(&g).phase, 1);
        assert_eq!(boss(&g).mode, BossMode::Call, "rallies at once");
        let before = g.enemies.len();
        run(&mut g, (CAPTAIN_DEF.call_time / TICK_DT) as usize + 2);
        let rally: usize = CAPTAIN_DEF.phases[1]
            .rally
            .iter()
            .map(|m| m.n as usize)
            .sum();
        assert_eq!(g.enemies.len(), before + rally);
        // Phase two's rotation brings the torch volley after the charge.
        until(&mut g, BossMode::Volley, 12.0);
        run(&mut g, (CAPTAIN_DEF.volley_windup / TICK_DT) as usize + 2);
        let torches = g.shots.iter().filter(|s| s.kind == ShotKind::Torch).count();
        assert_eq!(torches, 6);
        // Past the last threshold, straight to the third phase.
        g.enemies.hp[0] = max * 0.1;
        run(&mut g, 1);
        assert_eq!(boss(&g).phase, 2);
    }

    #[test]
    fn nothing_freezes_charms_or_heals_him() {
        let mut g = duel(vec2(60.0, 0.0));
        assert!(g.enemies.unswayed(0) && g.enemies.boss(0));
        g.enemies.spawn(CURATE, vec2(80.0, 0.0), 1.0);
        let hurt = g.enemies.max_hp[0] * 0.9;
        g.enemies.hp[0] = hurt;
        run(&mut g, 120);
        assert_eq!(g.enemies.hp[0], hurt, "the curate can't mend him");
    }

    #[test]
    fn killing_him_wins_and_drops_an_offering() {
        let mut g = duel(vec2(100.0, 0.0));
        run(&mut g, 10);
        assert!(g.boss.is_some());
        g.enemies.hp[0] = 0.0;
        assert_eq!(g.tick(&InputState::default()), SimEvent::Victory);
        assert!(g.boss.is_none());
        assert_eq!(g.offerings.len(), 1);
        for _ in 0..60 {
            assert_ne!(g.tick(&InputState::default()), SimEvent::Victory);
        }
    }
}

//! Run upgrades: Art levels, Sin levels, evolutions, the level-up choices and the
//! player stats derived from them. Pure logic.

use crate::config::*;
use crate::game::Player;
use macroquad::rand::gen_range;

/// One card on the level-up screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    /// Index into `ARTS`: a new Art at level 1, or the next level of an owned one.
    Art(usize),
    /// Index into `SINS`: a new Sin, or the next level of an owned one.
    Sin(usize),
}

pub struct Upgrades {
    /// Level of each Sin in `SINS`, 0 while not owned.
    pub sins: [u8; SINS.len()],
    /// Level of each Art in `ARTS`, 0 while not owned.
    pub arts: [u8; ARTS.len()],
    /// Owned Arts in the order they were taken (the HUD slots).
    pub slots: [Option<usize>; MAX_ARTS],
    /// Arts turned into their evolution by an Abyssal Offering.
    pub evolved: [bool; ARTS.len()],
}

impl Default for Upgrades {
    /// Every run starts with Hellflare.
    fn default() -> Self {
        let mut u = Self {
            sins: [0; SINS.len()],
            arts: [0; ARTS.len()],
            slots: [None; MAX_ARTS],
            evolved: [false; ARTS.len()],
        };
        u.take_art(HELLFLARE);
        u
    }
}

/// Effective player numbers after Sins. `might`, `cooldown`, `amount`, `duration` and
/// `area` apply to every Art: damage, cooldown, duration and area multipliers and
/// extra amount.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerStats {
    pub might: f32,
    pub cooldown: f32,
    pub amount: u32,
    pub duration: f32,
    pub area: f32,
    pub speed: f32,
    pub max_hp: f32,
    pub pickup_radius: f32,
    /// Taken off every blow she receives.
    pub armor: f32,
}

impl Upgrades {
    pub fn owned_arts(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    pub fn owned_sins(&self) -> usize {
        self.sins.iter().filter(|&&lv| lv > 0).count()
    }

    /// Every card that can be offered now: owned Arts and Sins below their top level,
    /// and new ones while a slot of their kind is free.
    fn pool(&self) -> ([Choice; ARTS.len() + SINS.len()], usize) {
        let mut pool = [Choice::Sin(0); ARTS.len() + SINS.len()];
        let mut n = 0;
        let free_art = self.owned_arts() < MAX_ARTS;
        for (i, &lv) in self.arts.iter().enumerate() {
            if (lv > 0 && lv < ART_MAX_LEVEL) || (lv == 0 && free_art) {
                pool[n] = Choice::Art(i);
                n += 1;
            }
        }
        let free_sin = self.owned_sins() < MAX_SINS;
        for (i, &lv) in self.sins.iter().enumerate() {
            if (lv > 0 && lv < SINS[i].max_level) || (lv == 0 && free_sin) {
                pool[n] = Choice::Sin(i);
                n += 1;
            }
        }
        (pool, n)
    }

    /// Fills `out` with up to `out.len()` distinct choices, `None` for the rest.
    pub fn roll_choices(&self, out: &mut [Option<Choice>]) {
        let (mut pool, n) = self.pool();
        for (k, slot) in out.iter_mut().enumerate() {
            *slot = if k < n {
                // Partial Fisher-Yates: pick from the untouched tail.
                let j = gen_range(k, n);
                pool.swap(k, j);
                Some(pool[k])
            } else {
                None
            };
        }
    }

    pub fn apply(&mut self, choice: Choice, player: &mut Player) {
        match choice {
            Choice::Art(i) => self.take_art(i),
            Choice::Sin(i) => self.take_sin(i, player),
        }
    }

    /// Adds Art `i` in the first free slot, or levels it up if owned.
    fn take_art(&mut self, i: usize) {
        let lv = &mut self.arts[i];
        if *lv >= ART_MAX_LEVEL {
            return;
        }
        if *lv == 0 {
            let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) else {
                return;
            };
            *slot = Some(i);
        }
        *lv += 1;
    }

    fn take_sin(&mut self, idx: usize, player: &mut Player) {
        let lv = self.sins[idx];
        if lv >= SINS[idx].max_level || (lv == 0 && self.owned_sins() >= MAX_SINS) {
            return;
        }
        self.sins[idx] += 1;
        let stats = self.stats();
        player.max_hp = stats.max_hp;
        if SINS[idx].kind == SinKind::BloodPact {
            player.hp = (player.hp + BLOOD_PACT_HEAL).min(player.max_hp);
        }
    }

    /// The first Art in slot order an Offering can evolve: at the top level, its
    /// paired Sin owned and not evolved yet.
    pub fn evolvable(&self) -> Option<usize> {
        self.slots.iter().flatten().copied().find(|&i| {
            self.arts[i] == ART_MAX_LEVEL
                && self.sins[ARTS[i].evolution.sin] > 0
                && !self.evolved[i]
        })
    }

    pub fn evolve(&mut self, art: usize) {
        self.evolved[art] = true;
    }

    /// Base values plus `step * level` (additive, multiplicative bonuses are percents).
    pub fn stats(&self) -> PlayerStats {
        let bonus = |i: usize| SINS[i].step * self.sins[i] as f32;
        PlayerStats {
            might: 1.0 + bonus(WRATH),
            cooldown: 1.0 - bonus(BLOODLUST),
            amount: bonus(LEGION) as u32,
            duration: 1.0 + bonus(DARK_SIGIL),
            area: 1.0 + bonus(ALLUREMENT),
            speed: PLAYER_SPEED * (1.0 + bonus(NIGHT_WINGS)),
            max_hp: PLAYER_MAX_HP + bonus(BLOOD_PACT),
            pickup_radius: PICKUP_RADIUS * (1.0 + bonus(SIRENS_CALL)),
            armor: bonus(INFERNAL_HIDE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn each_sin_matches_table() {
        let base = Upgrades::default().stats();
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        // Only six fit at once: take the first six, then check the other three alone.
        for i in 0..MAX_SINS {
            u.apply(Choice::Sin(i), &mut p);
        }
        let s = u.stats();
        assert!((s.might - base.might * 1.1).abs() < 1e-6);
        assert!((s.cooldown - base.cooldown * 0.92).abs() < 1e-6);
        assert_eq!(s.amount, base.amount + 1);
        assert!((s.speed - base.speed * 1.1).abs() < 1e-3);
        assert_eq!(s.max_hp, base.max_hp + 20.0);
        assert!((s.pickup_radius - base.pickup_radius * 1.4).abs() < 1e-3);
        assert_eq!(p.max_hp, s.max_hp);
        let mut u = Upgrades::default();
        for i in [DARK_SIGIL, ALLUREMENT, INFERNAL_HIDE] {
            u.apply(Choice::Sin(i), &mut p);
        }
        let s = u.stats();
        assert!((s.duration - 1.1).abs() < 1e-6);
        assert!((s.area - 1.1).abs() < 1e-6);
        assert_eq!(s.armor, 1.0);
    }

    #[test]
    fn caps_stop_offering_and_legion_stops_at_two() {
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        for _ in 0..10 {
            u.apply(Choice::Sin(LEGION), &mut p);
        }
        assert_eq!(u.stats().amount, 2);
        let mut out = [None; 3];
        for _ in 0..50 {
            u.roll_choices(&mut out);
            assert!(out.iter().all(|c| *c != Some(Choice::Sin(LEGION))));
        }
    }

    #[test]
    fn only_six_sins_and_then_no_new_ones() {
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        for i in 0..SINS.len() {
            u.apply(Choice::Sin(i), &mut p);
        }
        assert_eq!(u.owned_sins(), MAX_SINS);
        assert!(u.sins[MAX_SINS..].iter().all(|&lv| lv == 0));
        let mut out = [None; 3];
        for _ in 0..50 {
            u.roll_choices(&mut out);
            let new_sin = |c: &Option<Choice>| matches!(c, Some(Choice::Sin(i)) if *i >= MAX_SINS);
            assert!(!out.iter().any(new_sin));
        }
    }

    #[test]
    fn choices_are_distinct_and_shrink_when_few_remain() {
        let mut out = [None; 3];
        let mut u = Upgrades::default();
        for _ in 0..50 {
            u.roll_choices(&mut out);
            let [a, b, c] = out.map(Option::unwrap);
            assert!(a != b && b != c && a != c);
        }
        // Every Art capped; six Sins owned, all capped but two.
        u.arts = [ART_MAX_LEVEL; ARTS.len()];
        for (i, (lv, def)) in u.sins.iter_mut().zip(&SINS).take(MAX_SINS).enumerate() {
            *lv = def.max_level - (i < 2) as u8;
        }
        u.roll_choices(&mut out);
        assert!(out[0].is_some() && out[1].is_some() && out[2].is_none());
        u.sins[0] = SINS[0].max_level;
        u.sins[1] = SINS[1].max_level;
        u.roll_choices(&mut out);
        assert_eq!(out, [None; 3]);
    }

    #[test]
    fn blood_pact_heal_is_clamped() {
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        p.hp = PLAYER_MAX_HP - 5.0;
        u.apply(Choice::Sin(BLOOD_PACT), &mut p);
        assert_eq!(p.max_hp, PLAYER_MAX_HP + 20.0);
        assert_eq!(p.hp, PLAYER_MAX_HP + 20.0);
        p.hp = 10.0;
        u.apply(Choice::Sin(BLOOD_PACT), &mut p);
        assert_eq!(p.hp, 10.0 + BLOOD_PACT_HEAL);
    }

    #[test]
    fn runs_start_with_hellflare_and_new_arts_take_free_slots() {
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        assert_eq!(u.arts[HELLFLARE], 1);
        assert_eq!(u.slots[0], Some(HELLFLARE));
        u.apply(Choice::Art(KISS_OF_RUIN), &mut p);
        u.apply(Choice::Art(KISS_OF_RUIN), &mut p);
        assert_eq!(u.arts[KISS_OF_RUIN], 2);
        assert_eq!(u.slots[..3], [Some(HELLFLARE), Some(KISS_OF_RUIN), None]);
        for _ in 0..20 {
            u.apply(Choice::Art(KISS_OF_RUIN), &mut p);
        }
        assert_eq!(u.arts[KISS_OF_RUIN], ART_MAX_LEVEL);
        let mut out = [None; 3];
        for _ in 0..50 {
            u.roll_choices(&mut out);
            assert!(out.iter().all(|c| *c != Some(Choice::Art(KISS_OF_RUIN))));
        }
    }

    #[test]
    fn full_slots_offer_no_new_arts() {
        let mut u = Upgrades::default();
        // Pretend the other slots hold more Arts.
        for s in u.slots.iter_mut().skip(1) {
            *s = Some(HELLFLARE);
        }
        let mut out = [None; 3];
        for _ in 0..50 {
            u.roll_choices(&mut out);
            let new_art = |c: &Option<Choice>| matches!(c, Some(Choice::Art(i)) if *i != HELLFLARE);
            assert!(!out.iter().any(new_art));
        }
    }

    #[test]
    fn an_art_evolves_only_at_the_top_level_with_its_sin() {
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        assert_eq!(u.evolvable(), None);
        for _ in 1..ART_MAX_LEVEL {
            u.apply(Choice::Art(HELLFLARE), &mut p);
        }
        assert_eq!(u.evolvable(), None, "Legion missing");
        u.apply(Choice::Sin(WRATH), &mut p);
        assert_eq!(u.evolvable(), None, "the wrong Sin");
        u.apply(Choice::Sin(ARTS[HELLFLARE].evolution.sin), &mut p);
        assert_eq!(u.evolvable(), Some(HELLFLARE));
        u.evolve(HELLFLARE);
        assert_eq!(u.evolvable(), None, "only once");
    }

    #[test]
    fn every_evolution_pairs_with_its_own_sin() {
        let sins: Vec<usize> = ARTS.iter().map(|a| a.evolution.sin).collect();
        for (k, sin) in sins.iter().enumerate() {
            assert!(*sin < SINS.len());
            assert!(!sins[..k].contains(sin), "{sin} paired twice");
        }
        assert!(!sins.contains(&INFERNAL_HIDE));
    }
}

//! Run upgrades: Art levels, passive upgrade levels, the level-up choices and the
//! player stats derived from them. Pure logic.

use crate::config::*;
use crate::game::Player;
use macroquad::rand::gen_range;

/// One card on the level-up screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    /// Index into `ARTS`: a new Art at level 1, or the next level of an owned one.
    Art(usize),
    /// Index into `UPGRADES`.
    Upgrade(usize),
}

pub struct Upgrades {
    pub levels: [u8; UPGRADES.len()],
    /// Level of each Art in `ARTS`, 0 while not owned.
    pub arts: [u8; ARTS.len()],
    /// Owned Arts in the order they were taken (the HUD slots).
    pub slots: [Option<usize>; MAX_ARTS],
}

impl Default for Upgrades {
    /// Every run starts with Hellflare.
    fn default() -> Self {
        let mut u = Self {
            levels: [0; UPGRADES.len()],
            arts: [0; ARTS.len()],
            slots: [None; MAX_ARTS],
        };
        u.take_art(HELLFLARE);
        u
    }
}

/// Effective player numbers after upgrades. `might`, `cooldown` and `amount` apply
/// to every Art: damage multiplier, cooldown multiplier and extra amount.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerStats {
    pub might: f32,
    pub cooldown: f32,
    pub amount: u32,
    pub speed: f32,
    pub max_hp: f32,
    pub pickup_radius: f32,
}

impl Upgrades {
    /// Indices into `UPGRADES` that are not capped yet.
    pub fn available(&self) -> impl Iterator<Item = usize> + '_ {
        (0..UPGRADES.len()).filter(|&i| self.levels[i] < UPGRADES[i].max_level)
    }

    pub fn owned_arts(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    /// Every card that can be offered now: owned Arts below the top level, new Arts
    /// while a slot is free, and upgrades not capped.
    fn pool(&self) -> ([Choice; ARTS.len() + UPGRADES.len()], usize) {
        let mut pool = [Choice::Upgrade(0); ARTS.len() + UPGRADES.len()];
        let mut n = 0;
        let free_slot = self.owned_arts() < MAX_ARTS;
        for (i, &lv) in self.arts.iter().enumerate() {
            if (lv > 0 && lv < ART_MAX_LEVEL) || (lv == 0 && free_slot) {
                pool[n] = Choice::Art(i);
                n += 1;
            }
        }
        for i in self.available() {
            pool[n] = Choice::Upgrade(i);
            n += 1;
        }
        (pool, n)
    }

    /// Fills `out` with up to 3 distinct choices, `None` for the rest.
    pub fn roll_choices(&self, out: &mut [Option<Choice>; 3]) {
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
            Choice::Upgrade(i) => self.apply_upgrade(i, player),
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

    fn apply_upgrade(&mut self, idx: usize, player: &mut Player) {
        if self.levels[idx] >= UPGRADES[idx].max_level {
            return;
        }
        self.levels[idx] += 1;
        let stats = self.stats();
        player.max_hp = stats.max_hp;
        if UPGRADES[idx].kind == UpgradeKind::BloodPact {
            player.hp = (player.hp + BLOOD_PACT_HEAL).min(player.max_hp);
        }
    }

    /// Base values plus `step * level` (additive, multiplicative bonuses are percents).
    pub fn stats(&self) -> PlayerStats {
        let bonus = |kind: UpgradeKind| {
            let i = UPGRADES.iter().position(|u| u.kind == kind).unwrap();
            UPGRADES[i].step * self.levels[i] as f32
        };
        PlayerStats {
            might: 1.0 + bonus(UpgradeKind::Fury),
            cooldown: 1.0 / (1.0 + bonus(UpgradeKind::Frenzy)),
            amount: bonus(UpgradeKind::EmberLegion) as u32,
            speed: PLAYER_SPEED * (1.0 + bonus(UpgradeKind::LightWings)),
            max_hp: PLAYER_MAX_HP + bonus(UpgradeKind::BloodPact),
            pickup_radius: PICKUP_RADIUS * (1.0 + bonus(UpgradeKind::SoulPull)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    fn idx(kind: UpgradeKind) -> usize {
        UPGRADES.iter().position(|u| u.kind == kind).unwrap()
    }

    #[test]
    fn each_effect_matches_table() {
        let base = Upgrades::default().stats();
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        for i in 0..UPGRADES.len() {
            u.apply(Choice::Upgrade(i), &mut p);
        }
        let s = u.stats();
        assert_eq!(s.might, base.might * 1.25);
        assert!((s.cooldown - base.cooldown / 1.15).abs() < 1e-6);
        assert_eq!(s.amount, base.amount + 1);
        assert!((s.speed - base.speed * 1.1).abs() < 1e-3);
        assert_eq!(s.max_hp, base.max_hp + 20.0);
        assert!((s.pickup_radius - base.pickup_radius * 1.4).abs() < 1e-3);
        assert_eq!(p.max_hp, s.max_hp);
    }

    #[test]
    fn caps_stop_offering_and_ember_legion_stops_at_four() {
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        let ember = idx(UpgradeKind::EmberLegion);
        for _ in 0..10 {
            u.apply(Choice::Upgrade(ember), &mut p);
        }
        assert_eq!(u.stats().amount, 4);
        assert!(u.available().all(|i| i != ember));
        let mut out = [None; 3];
        for _ in 0..50 {
            u.roll_choices(&mut out);
            assert!(out.iter().all(|c| *c != Some(Choice::Upgrade(ember))));
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
        u.arts = [ART_MAX_LEVEL; ARTS.len()];
        for (lv, def) in u.levels.iter_mut().zip(&UPGRADES).skip(2) {
            *lv = def.max_level;
        }
        u.roll_choices(&mut out);
        assert!(out[0].is_some() && out[1].is_some() && out[2].is_none());
        u.levels = UPGRADES.map(|d| d.max_level);
        u.roll_choices(&mut out);
        assert_eq!(out, [None; 3]);
    }

    #[test]
    fn blood_pact_heal_is_clamped() {
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        p.hp = PLAYER_MAX_HP - 5.0;
        u.apply(Choice::Upgrade(idx(UpgradeKind::BloodPact)), &mut p);
        assert_eq!(p.max_hp, PLAYER_MAX_HP + 20.0);
        assert_eq!(p.hp, PLAYER_MAX_HP + 20.0);
        p.hp = 10.0;
        u.apply(Choice::Upgrade(idx(UpgradeKind::BloodPact)), &mut p);
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
        // Pretend the other slots hold Arts that don't exist yet.
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
}

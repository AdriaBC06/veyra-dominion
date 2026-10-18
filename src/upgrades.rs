//! Upgrade levels and the player stats derived from them. Pure logic.

use crate::config::*;
use crate::game::Player;
use macroquad::rand::gen_range;

#[derive(Default)]
pub struct Upgrades {
    pub levels: [u8; UPGRADES.len()],
}

/// Effective player numbers after upgrades.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerStats {
    pub damage: f32,
    pub fire_interval: f32,
    pub projectiles: u32,
    pub speed: f32,
    pub max_hp: f32,
    pub pickup_radius: f32,
}

impl Upgrades {
    /// Indices into `UPGRADES` that are not capped yet.
    pub fn available(&self) -> impl Iterator<Item = usize> + '_ {
        (0..UPGRADES.len()).filter(|&i| self.levels[i] < UPGRADES[i].max_level)
    }

    /// Fills `out` with up to 3 distinct available upgrades, `None` for the rest.
    pub fn roll_choices(&self, out: &mut [Option<usize>; 3]) {
        let mut pool = [0usize; UPGRADES.len()];
        let mut n = 0;
        for i in self.available() {
            pool[n] = i;
            n += 1;
        }
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

    pub fn apply(&mut self, idx: usize, player: &mut Player) {
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
            damage: FIRE_DAMAGE * (1.0 + bonus(UpgradeKind::Fury)),
            fire_interval: FIRE_INTERVAL / (1.0 + bonus(UpgradeKind::Frenzy)),
            projectiles: 1 + bonus(UpgradeKind::EmberLegion) as u32,
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
            u.apply(i, &mut p);
        }
        let s = u.stats();
        assert_eq!(s.damage, base.damage * 1.25);
        assert!((s.fire_interval - base.fire_interval / 1.15).abs() < 1e-6);
        assert_eq!(s.projectiles, 2);
        assert!((s.speed - base.speed * 1.1).abs() < 1e-3);
        assert_eq!(s.max_hp, base.max_hp + 20.0);
        assert!((s.pickup_radius - base.pickup_radius * 1.4).abs() < 1e-3);
        assert_eq!(p.max_hp, s.max_hp);
    }

    #[test]
    fn caps_stop_offering_and_ember_legion_stops_at_five() {
        let mut p = Game::new().player;
        let mut u = Upgrades::default();
        let ember = idx(UpgradeKind::EmberLegion);
        for _ in 0..10 {
            u.apply(ember, &mut p);
        }
        assert_eq!(u.stats().projectiles, 5);
        assert!(u.available().all(|i| i != ember));
        let mut out = [None; 3];
        for _ in 0..50 {
            u.roll_choices(&mut out);
            assert!(out.iter().all(|c| *c != Some(ember)));
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
        u.apply(idx(UpgradeKind::BloodPact), &mut p);
        assert_eq!(p.max_hp, PLAYER_MAX_HP + 20.0);
        assert_eq!(p.hp, PLAYER_MAX_HP + 20.0);
        p.hp = 10.0;
        u.apply(idx(UpgradeKind::BloodPact), &mut p);
        assert_eq!(p.hp, 10.0 + BLOOD_PACT_HEAL);
    }
}

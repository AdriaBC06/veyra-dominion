# Veyra: Dominion — design history (archive)

Superseded design notes moved out of `docs/design.md` on 2026-10-22. They describe how things were built before later redesigns; the current state is in `docs/design.md`.

## 7.7 Arts and Sins before the V17b redesign

The V17 state (replaced by "Redesign direction" in V17b: Kiss of Ruin became Hellflare's evolution, Wing Gust became Scent of Sin, Blood Pact → Gluttony, Siren's Call and Allurement removed).

| Art | Archetype (VS analogue) | Behaviour | Evolution (+ Sin) |
|---|---|---|---|
| Hellflare | Magic Wand | Homing fireballs at the nearest enemy | **Hellstorm** (+ Legion): rain of fireballs |
| Tail Lash | Whip | Horizontal sweep on the facing side; alternates sides | **Barbed Scourge** (+ Wrath): bleed damage over time |
| Kiss of Ruin | Knife | Fast piercing heart bolt in the move direction | **Kiss of Death** (+ Blood Pact): lifesteal on hit |
| Brimstone Halo | King Bible | Orbiting embers | **Infernal Crown** (+ Dark Sigil): permanent ring |
| Allure | Garlic (area) plus a succubus twist | Aura that **charms** enemies: they fight for Veyra for a few seconds | **Siren's Dominion** (+ Siren's Call): charmed enemies explode on expiry |
| Hellgate | Santa Water | Burning pools at random spots near enemies | **Abyssal Rift** (+ Allurement): pools pull enemies in |
| Chain of Torment | Lightning Ring | Chain lightning between N enemies | **Torment Eternal** (+ Bloodlust): unlimited bounces in a radius |
| Wing Gust | Clock Lancet / control | Cone knockback with a short slow | **Tempest of Wings** (+ Night Wings): pushes and damages |

| Sin | Effect / level | Max |
|---|---|---|
| Wrath | +10% damage | 5 |
| Bloodlust | −8% cooldown | 5 |
| Legion | +1 projectile / instance | 2 |
| Night Wings | +10% speed | 5 |
| Blood Pact | +20 max HP, heal 30 | 5 |
| Siren's Call | +40% pickup radius | 5 |
| Dark Sigil | +10% duration | 5 |
| Allurement | +10% area | 5 |
| Greed | +20% Tribute (joins with Tribute in V20) | 5 |
| Infernal Hide | +1 armour | 5 |

**Built in V15 (S12):**
- `src/arts.rs` (not `weapons.rs`): `Arts` owns the fixed pools (fireballs, lashes, hearts) and the Halo state, the cooldowns and the effective stats; `Game::tick` calls `arts.update(World { player, enemies, hash, time }, alive)` where Hellflare used to fire. Levels live in `Upgrades` (`arts[i]`, 0 = not owned, plus `slots`, 6 max); every run starts with Hellflare at level 1. Call `arts.refresh(&upgrades)` after any change.
- Data in `config/arts.rs` (`config.rs` became `config/mod.rs` once it passed 600 lines): `ArtDef { base: ArtStats, gains: [ArtGain; 7] }`, `ArtDef::stats(level)`. `ArtStats` fields mean different things per Art (see its doc: e.g. the Halo's `cooldown` is its rest time and `speed` its radians per second). Cooldown, area and speed gains are fractions of the base.
- Level-up choices are mixed already (`upgrades::Choice::{Art, Upgrade}`): owned Arts below level 8, new Arts while a slot is free, and the 6 old upgrades. Cards show "New" + what the Art does, or what its next level adds (built from the gain, `hud::gain_text`). V17 only has to rename the upgrades into the Sins table and add the new Sins.
- Fury, Frenzy and Ember Legion now boost **every** Art (damage ×, cooldown ÷, +amount); their texts say so.
- Hellflare: 10 dmg, 0.6 s, levels add fireballs (2, 4, 7), −10% cooldown (3, 6), +5 damage (5, 8). Tail Lash: 14 dmg every 1.35 s on a 52×24 box on the side she faces; level 2 adds a second lash behind her 0.12 s later (extra amount keeps alternating sides); then +5 damage per level and −10% cooldown at 4 and 6. Kiss of Ruin: 8 dmg hearts at 420 px/s along the last move direction (`Player::aim`), pierce 1, 0.9 s life; +1 heart at 2, 3, 4, 6, 7, +4 damage at 3 and 7, +1 pierce at 5 and 8. A heart never hits the same enemy twice (enemies have a stable `uid` now). Brimstone Halo: 1 ember at radius 36 for 3 s, then 3 s off; 10 dmg, each enemy burned at most every 0.6 s (`Enemies::halo_until`) and pushed 6 px out; +1 ember at 2, 5, 8, +25% radius and +30% spin at 3 and 6, +5 damage and +0.5 s at 4 and 7.
- No Art scales its sprite (area only grows the Halo's radius), since non-integer scaling breaks the pixel art. Decide in V17 what Allurement (+area) does for the Lash.
- Art: `assets/arts.px.js` → `assets/sprites/arts/anims/` (tail_lash 56×20 ×3, heart_bolt 9×9 ×2, ember 7×8 ×2). The lash is drawn with its tips 2 px above her ground point, flipped by side; embers behind her when on the far side of the orbit.
- Sounds: `Sfx::Lash` (rising air, then a crack on a high D) and `Sfx::Kiss` (a breath and a sine leaning D→Eb, her minor 2nd), both measured at about −28 dB like Hellflare. Halo hits use the normal hit sound.
- Debug: **F7** in a run levels up every Art (new ones join at level 1).
- Stress test: every Art at level 8 on a full 2,500 horde (×1000 HP so it stays full): 0.53 ms mean tick, was 0.50 ms with Hellflare alone.

**Built in V17 (S14):**
- `config/mod.rs`: `SINS` table (`SinDef`, `SinKind`, index consts `WRATH`..`INFERNAL_HIDE`), values as in the table above (Wrath is +10% now, not Fury's +25%; Legion caps at 2). `Choice::Sin`; `Upgrades.sins` levels; at most `MAX_SINS` (6) owned, new Sins only offered while one is free. Greed is left out until Tribute exists (V20), so no Sin is inert.
- `PlayerStats` gained `duration`, `area`, `armor`. Dark Sigil scales every Art's `duration` except the projectile lifetimes (Hellflare, Kiss: that is their range). Infernal Hide takes 1 per level off contact damage, never below `MIN_CONTACT_DAMAGE` (1).
- **Allurement on the Lash (decision):** area deepens the lash's band (`LASH_DEPTH × area`), not its reach; the sprite never scales, so two faint copies (`LASH_ECHO_ALPHA`) mark the band's edges. Hellgate's pools still ignore area; Abyssal Rift's pull radius uses it, hence the pairing.
- Evolutions: `ArtDef.evolution { sin, name, desc }`, `Upgrades::evolvable()` (first slot at level 8 with its Sin owned) / `evolve`; `Arts` branches on `evolved[i]`. Hellstorm: +2 fireballs, each at its own random enemy (reservoir pick). Barbed Scourge: lashed enemies bleed for the Lash's `duration` (3 s base), `BLEED_SHARE` of its damage every `BLEED_TICK`, and drip blood (`Enemies.bleed_until`). Kiss of Death: 1 HP per heart hit, at most `KISS_HEAL_CAP`/s. Infernal Crown: never rests, +1 ember. Siren's Dominion: when a charm ends the enemy bursts (dies, `SIREN_BURST_DAMAGE` × damage around it, a ring of hearts); `charm_until` is reset to 0 on expiry. Abyssal Rift: pools drag enemies within `RIFT_RADIUS` × area to their centre. Torment Eternal: up to `MAX_CHAIN_LINKS` (now 24) bites, hopping anywhere within `TORMENT_RADIUS` of the first victim. Tempest of Wings: one gust all round, ×3 damage, feathers in 6 directions.
- **Abyssal Offerings:** elites drop one where they die (`Game.offerings`); touching it opens it (`SimEvent::Offering`, a screen in `run.rs` that waits `OFFERING_HOLD` before accepting a click/Enter). It evolves the first evolvable Art, else grants a random level (`Reward::Gift`), else heals `OFFERING_HEAL`. Sprite `offering` (13×14 ×4, horned reliquary with a pulsing sigil) in `assets/arts.px.js`. Debug **F8** drops one ahead of Veyra (use with F5 elites / F7 levels).
- Sounds: `Sfx::SirenBurst` (wet pop, sine falling G#→D) −28.1 dB, `Sfx::Offering` (low bell, sine climbing D minor to the Eb→D sigh, thin octave glint) about −15.5 dB like level-up.
- Stress test now runs with every Art evolved and every Sin maxed except Siren's Dominion (it kills what it charms and would thin the horde), ×20000 HP: 0.37 ms mean tick (0.57 before on the same machine with the old setup).

**Built in V16 (S13):**
- New `Enemies` columns: `charm_until`, `slow_until`, `knock` (velocity). `steer` now takes the game time: charmed enemies head for the nearest free enemy within `CHARM_SEEK` (or stand still), slowed ones move at `GUST_SLOW`, knockback is added on top and decays at `KNOCK_DECAY`. Charmed enemies never deal contact damage to Veyra. Hellflare and the other Arts still hit charmed enemies (kept simple).
- Allure (4): a pulse every 3 s, only when an enemy is within `ALLURE_RADIUS` (56 × area). The first `amount` free enemies in reach are **charmed for `duration`, not hurt** (at first the pulse burned them too and killed the weak ones as soon as they were charmed); the rest take the damage. Every `CHARM_BITE` (0.5 s) each charmed enemy bites the free enemies within its radius + `CHARM_REACH` for the Art's damage. Base 5 dmg, 2 charmed, 2.5 s; levels add charmed (2, 5, 8 ×2), area (3, 6), damage + duration (4, 7). Drawn: a ring of `charm_heart`s spreading over `ALLURE_WAVE_TIME`, charmed enemies tinted `CHARM_TINT` with a beating heart over their heads.
- Hellgate (5): every 4 s `amount` pools on random enemies within `HELLGATE_RANGE` (reservoir pick, `random_enemy`). A pool is an ellipse (`HELLGATE_RADIUS` wide, half as deep, plus the enemy radius) that burns everything inside every `HELLGATE_TICK` (0.35 s) for `duration` (2.5 s). Base 7 dmg, 1 pool. No area gains (sprite can't scale). Drawn on the ground right after the blood, fading in and out.
- Chain of Torment (6): every 1.6 s it bites a random enemy within `CHAIN_RANGE`, then jumps to the nearest unbitten enemy within `CHAIN_JUMP` × area, `amount` enemies in all (base 3, 12 dmg). It counts as a cast (cast animation). Drawn as a jagged line from her hand (a kink every `CHAIN_KINK` px, reshaped every 0.05 s): 5 px dark glow, 3 px crimson, 1 px white-hot core (`CHAIN_COLORS`), plus a `chain_spark` on each victim. The first try (1 px line, 4 kinks per link) was invisible on the grass.
- Wing Gust (7): every 2.2 s, if an enemy is within `GUST_REACH` (72 × area), `amount` cones (±`GUST_HALF_ANGLE`, the first toward that enemy, the rest spread around the circle) throw enemies out at `GUST_PUSH` (less for big ones), slow them for `duration` and deal 5 dmg. Each gust throws `GUST_FEATHERS` of her feathers (pink, they read as brown dots once faded, so they fade only in the last 30%).
- Ember Legion's +amount reaches all four (one more charmed / pool / bitten enemy / gust). With 8 Arts and 6 slots, F7 now only fills the free slots.
- Art: `charm_heart` 7×7 ×2, `hellgate` 36×22 ×4 (pool centre at (18, 16)), `chain_spark` 9×9 ×2, `gust_feather` 11×6 ×2, in `assets/arts.px.js`.
- Sounds (peak 50 ms RMS): `Sfx::Allure` -27.8 (a sine scooping up into D with a slow vibrato, the tritone G# whispered on a thin pulse), `Hellgate` -25.5 (low whoomph, fire roar, crackle), `Chain` -28.3 (snap and a zap leaping a tritone, rattles), `Gust` -27.2 (wing thump and a falling whoosh).
- Stress test with all 8 Arts at level 8: 0.56 ms mean tick (was 0.53 with 4).

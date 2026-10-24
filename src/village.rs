//! The living village (design.md 7.14, V19h): the Arts leave marks where they hit, and the
//! marks break barrels and carts, crack houses down to rubble, set things alight and cut
//! down the villagers fleeing from Veyra. Wreckage drops coins, food, coffers and the relic.
//! Simulation only: the renderer reads `props`, `civilians` and `drops`, the game drains
//! `happenings`.

use crate::config::*;
use crate::map::{props, Map};
use macroquad::math::{vec2, Rect, Vec2};
use macroquad::rand::gen_range;
use std::f32::consts::TAU;

/// Sprite sizes (the sim needs the hit areas, not the pixels).
const VILLAGE_IDX: &str = include_str!("../assets/sprites/village/village.idx");
const NONE: u16 = u16::MAX;

fn sprite_size(name: &str) -> Option<Vec2> {
    VILLAGE_IDX.lines().find_map(|l| {
        let mut w = l.split(' ');
        (w.next() == Some(name)).then(|| {
            let mut n = || w.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            vec2(n(), n())
        })
    })
}

/// Where an Art struck: whatever of the village it overlaps takes `damage`, and catches
/// fire if `fire`.
#[derive(Clone, Copy, Debug)]
pub struct Mark {
    pub pos: Vec2,
    pub radius: f32,
    pub damage: f32,
    pub fire: bool,
}

/// Queues a mark of `art` (dropped when the queue is full).
pub fn mark(marks: &mut Vec<Mark>, art: usize, pos: Vec2, radius: f32, damage: f32) {
    if marks.len() < MAX_MARKS {
        marks.push(Mark {
            pos,
            radius,
            damage,
            fire: ARTS[art].fire,
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropKind {
    /// Index into `BUILDINGS`.
    Building(u8),
    /// Index into `BREAKABLES`.
    Breakable(u8),
    /// Chars into `<name>_burnt`.
    Charring,
    /// Burns down to stubble.
    Low,
}

/// A prop the Arts can touch.
pub struct Prop {
    pub kind: PropKind,
    /// Its sprite in world pixels: the hit area.
    pub rect: Rect,
    pub hp: f32,
    pub max_hp: f32,
    heat: f32,
    /// Seconds left burning, 0 when not.
    pub burning: f32,
    pub burnt: bool,
    /// A breakable smashed, a building fallen to rubble.
    pub broken: bool,
    pub hit_until: f32,
    /// On the `hot` list (warm or burning).
    hot: bool,
}

impl Prop {
    pub fn cracked(&self) -> bool {
        self.hp < self.max_hp * CRACK_AT
    }

    fn burns(&self) -> bool {
        match self.kind {
            PropKind::Breakable(b) => BREAKABLES[b as usize].burns && !self.broken,
            _ => !self.burnt,
        }
    }

    fn ignite_at(&self) -> f32 {
        match self.kind {
            PropKind::Building(_) => BUILDING_IGNITE,
            _ => PLANT_IGNITE,
        }
    }

    fn burn_time(&self) -> f32 {
        match self.kind {
            PropKind::Building(_) => BUILDING_BURN_TIME,
            PropKind::Breakable(_) => BREAKABLE_BURN_TIME,
            PropKind::Charring => TREE_BURN_TIME,
            PropKind::Low => LOW_BURN_TIME,
        }
    }

    /// Where its folk come out.
    pub fn door(&self) -> Vec2 {
        vec2(self.rect.center().x, self.rect.bottom() + 4.0)
    }
}

/// A building's people: still inside, and the wanderers it may still let out.
struct Home {
    prop: u16,
    folk: u8,
    wanderers: u8,
}

/// Index into `CIVILIAN_SPRITES`: the first animal.
pub const CHICKEN: u8 = 3;

pub struct Civilian {
    pub pos: Vec2,
    pub kind: u8,
    /// -1.0 facing left, 1.0 right.
    pub facing: f32,
    /// Distance walked, for the run cycle.
    pub stride: f32,
    /// Seconds left running from Veyra.
    pub flee: f32,
    /// Moved this tick (else it stands, for the drawing).
    pub walking: bool,
    home: u16,
    /// Came out to wander (goes back to its home's wanderers), else fled from inside.
    wanderer: bool,
    target: Vec2,
    /// Seconds left standing still before the next stroll.
    pause: f32,
}

impl Civilian {
    pub fn folk(&self) -> bool {
        self.kind < CHICKEN
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DropKind {
    Coin,
    Bread,
    Ham,
    Coffer,
    Relic,
}

/// In `BREAKABLE_DROPS` order.
const DROP_ROLL: [DropKind; 5] = [
    DropKind::Coin,
    DropKind::Bread,
    DropKind::Ham,
    DropKind::Coffer,
    DropKind::Relic,
];

pub struct Drop {
    pub kind: DropKind,
    pub pos: Vec2,
    /// Where it was flung from; it hops to `pos` over `DROP_SETTLE`.
    pub from: Vec2,
    pub t: f32,
    /// Seconds since a coffer was opened.
    pub opened: Option<f32>,
}

/// What happened in the village this tick, for the game to answer (souls, blood, smoke,
/// rewards).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Happening {
    Slain { pos: Vec2, folk: bool },
    Broke { pos: Vec2 },
    /// A blow landed on a standing building (or breakable, `building` false).
    Struck { pos: Vec2, building: bool },
    Collapsed { rect: Rect },
    Picked { kind: DropKind, pos: Vec2 },
}

pub struct Village {
    pub props: Vec<Prop>,
    /// Per layout prop (`map::props` order), its index in `props` or `NONE`.
    by_layout: Vec<u16>,
    /// Props per chunk: `chunk_items[chunk_start[c]..chunk_start[c + 1]]`.
    chunk_start: Vec<u32>,
    chunk_items: Vec<u16>,
    chunk_cols: usize,
    chunk_rows: usize,
    origin: Vec2,
    homes: Vec<Home>,
    /// Props warm or burning (each once, see `Prop::hot`).
    hot: Vec<u16>,
    burning: usize,
    spread_clock: f32,
    pub civilians: Vec<Civilian>,
    pub drops: Vec<Drop>,
    /// Arts' hits this tick (filled during `Arts::update`, spent by `update`).
    pub marks: Vec<Mark>,
    pub happenings: Vec<Happening>,
}

impl Village {
    /// No village (open ground).
    pub fn empty() -> Self {
        Self::build(&Map::open(), "")
    }

    /// The interactive props of layout `src` (the one `map` was parsed from).
    pub fn new(map: &Map, src: &str) -> Self {
        Self::build(map, src)
    }

    fn build(map: &Map, src: &str) -> Self {
        let mut list = Vec::new();
        let mut by_layout = Vec::new();
        let mut homes = Vec::new();
        for (name, bottom_left) in props(src) {
            let kind = if let Some(b) = BUILDINGS.iter().position(|b| b.name == name) {
                Some(PropKind::Building(b as u8))
            } else if let Some(b) = BREAKABLES.iter().position(|b| b.name == name) {
                Some(PropKind::Breakable(b as u8))
            } else if CHARRING_PLANTS.contains(&name) {
                Some(PropKind::Charring)
            } else if LOW_PLANTS.contains(&name) {
                Some(PropKind::Low)
            } else {
                None
            };
            let (Some(kind), Some(size)) = (kind, sprite_size(name)) else {
                by_layout.push(NONE);
                continue;
            };
            let p = map.origin + bottom_left - vec2(0.0, size.y);
            let hp = match kind {
                PropKind::Building(b) => {
                    let def = &BUILDINGS[b as usize];
                    homes.push(Home {
                        prop: list.len() as u16,
                        folk: def.folk,
                        wanderers: HOME_WANDERERS,
                    });
                    def.hp
                }
                PropKind::Breakable(b) => BREAKABLES[b as usize].hp,
                _ => 1.0,
            };
            by_layout.push(list.len() as u16);
            list.push(Prop {
                kind,
                rect: Rect::new(p.x, p.y, size.x, size.y),
                hp,
                max_hp: hp,
                heat: 0.0,
                burning: 0.0,
                burnt: false,
                broken: false,
                hit_until: 0.0,
                hot: false,
            });
        }
        let bounds = map.bounds().unwrap_or_default();
        let chunk_cols = (bounds.w / PROP_CHUNK).ceil() as usize + 1;
        let chunk_rows = (bounds.h / PROP_CHUNK).ceil() as usize + 1;
        let mut buckets = vec![Vec::new(); chunk_cols * chunk_rows];
        let origin = bounds.point();
        for (k, p) in list.iter().enumerate() {
            let (c0, r0, c1, r1) = chunk_span(origin, chunk_cols, chunk_rows, p.rect);
            for r in r0..=r1 {
                for c in c0..=c1 {
                    buckets[r * chunk_cols + c].push(k as u16);
                }
            }
        }
        let mut chunk_start = Vec::with_capacity(buckets.len() + 1);
        let mut chunk_items = Vec::new();
        for b in &buckets {
            chunk_start.push(chunk_items.len() as u32);
            chunk_items.extend_from_slice(b);
        }
        chunk_start.push(chunk_items.len() as u32);
        let n = list.len();
        Self {
            props: list,
            by_layout,
            chunk_start,
            chunk_items,
            chunk_cols,
            chunk_rows,
            origin,
            homes,
            hot: Vec::with_capacity(n),
            burning: 0,
            spread_clock: 0.0,
            civilians: Vec::with_capacity(MAX_CIVILIANS),
            drops: Vec::with_capacity(MAX_DROPS),
            marks: Vec::with_capacity(MAX_MARKS),
            happenings: Vec::with_capacity(MAX_MARKS),
        }
    }

    /// The prop that layout prop `k` became, if the Arts can touch it.
    pub fn of_layout(&self, k: usize) -> Option<usize> {
        self.by_layout
            .get(k)
            .filter(|&&i| i != NONE)
            .map(|&i| i as usize)
    }

    /// How many things are burning.
    #[cfg(test)]
    pub fn burning(&self) -> usize {
        self.burning
    }

    /// One tick: spends the marks, then fire, villagers and drops (picked up only while
    /// Veyra is `alive`).
    pub fn update(&mut self, map: &mut Map, veyra: Vec2, time: f32, alive: bool) {
        for k in 0..self.marks.len() {
            let m = self.marks[k];
            self.apply_mark(map, m, time);
        }
        self.marks.clear();
        self.update_fire(map);
        self.update_homes(veyra, alive);
        self.update_civilians(map, veyra);
        if alive {
            self.update_drops(veyra);
        }
    }

    fn happen(&mut self, h: Happening) {
        if self.happenings.len() < self.happenings.capacity() {
            self.happenings.push(h);
        }
    }

    fn apply_mark(&mut self, map: &mut Map, m: Mark, time: f32) {
        let area = Rect::new(
            m.pos.x - m.radius,
            m.pos.y - m.radius,
            m.radius * 2.0,
            m.radius * 2.0,
        );
        let (c0, r0, c1, r1) = chunk_span(self.origin, self.chunk_cols, self.chunk_rows, area);
        if !self.props.is_empty() {
            for r in r0..=r1 {
                for c in c0..=c1 {
                    let ch = r * self.chunk_cols + c;
                    for n in self.chunk_start[ch]..self.chunk_start[ch + 1] {
                        let i = self.chunk_items[n as usize] as usize;
                        self.mark_prop(map, i, m, time);
                    }
                }
            }
        }
        let mut k = self.civilians.len();
        while k > 0 {
            k -= 1;
            let c = &self.civilians[k];
            let reach = m.radius + CIVILIAN_RADIUS;
            if c.pos.distance_squared(m.pos) <= reach * reach {
                let (pos, folk) = (c.pos, c.folk());
                self.civilians.swap_remove(k);
                self.happen(Happening::Slain { pos, folk });
            }
        }
    }

    fn mark_prop(&mut self, map: &mut Map, i: usize, m: Mark, time: f32) {
        let p = &mut self.props[i];
        if p.hit_until > time || !circle_hits_rect(m.pos, m.radius, p.rect) {
            return;
        }
        let structure = matches!(p.kind, PropKind::Building(_) | PropKind::Breakable(_));
        if !structure && !m.fire {
            return; // plants only burn
        }
        if structure && p.broken && !m.fire {
            return;
        }
        p.hit_until = time + PROP_HIT_COOLDOWN;
        if structure && !p.broken {
            p.hp -= m.damage;
            let r = p.rect;
            let at = vec2(
                m.pos.x.clamp(r.x, r.right()),
                m.pos.y.clamp(r.y, r.bottom()),
            );
            let building = matches!(p.kind, PropKind::Building(_));
            self.happen(Happening::Struck { pos: at, building });
        }
        let p = &mut self.props[i];
        if m.fire && p.burns() && p.burning == 0.0 {
            p.heat += FIRE_HEAT;
            self.warm(i);
        }
        if structure && !self.props[i].broken && self.props[i].hp <= 0.0 {
            self.wreck(map, i);
        }
    }

    /// Puts prop `i` on the hot list, lighting it if it's warm enough and there's room.
    fn warm(&mut self, i: usize) {
        let room = self.burning < MAX_BURNING;
        let p = &mut self.props[i];
        if !p.hot {
            p.hot = true;
            self.hot.push(i as u16);
        }
        if p.burning == 0.0 && p.heat >= p.ignite_at() && p.burns() && room {
            p.burning = p.burn_time();
            p.heat = 0.0;
            self.burning += 1;
        }
    }

    /// A breakable smashes, a building falls to rubble (it no longer stops thrown things),
    /// its folk running out; both scatter what they held.
    fn wreck(&mut self, map: &mut Map, i: usize) {
        let p = &mut self.props[i];
        p.broken = true;
        p.hp = 0.0;
        let (rect, kind) = (p.rect, p.kind);
        match kind {
            PropKind::Breakable(_) => {
                let at = vec2(rect.center().x, rect.bottom() - 2.0);
                if p.burning > 0.0 {
                    p.burning = 0.0;
                    self.burning -= 1;
                }
                self.happen(Happening::Broke { pos: at });
                let r = gen_range(0.0, 1.0);
                let mut acc = 0.0;
                for (k, &chance) in BREAKABLE_DROPS.iter().enumerate() {
                    acc += chance;
                    if r < acc {
                        self.drop_item(DROP_ROLL[k], at);
                        break;
                    }
                }
            }
            PropKind::Building(b) => {
                map.open_shot(rect);
                self.happen(Happening::Collapsed { rect });
                let at = vec2(rect.center().x, rect.bottom() - 6.0);
                for _ in 0..gen_range(HOUSE_COINS.0, HOUSE_COINS.1 + 1) {
                    self.drop_item(DropKind::Coin, at);
                }
                if BUILDINGS[b as usize].name == "chapel" {
                    if gen_range(0.0, 1.0) < CHAPEL_RELIC {
                        self.drop_item(DropKind::Relic, at);
                    }
                } else if gen_range(0.0, 1.0) < HOUSE_COFFER {
                    self.drop_item(DropKind::Coffer, at);
                }
                if let Some(h) = self.homes.iter().position(|h| h.prop as usize == i) {
                    let folk = std::mem::take(&mut self.homes[h].folk);
                    self.homes[h].wanderers = 0;
                    for _ in 0..folk {
                        self.let_out(h, false);
                    }
                }
            }
            _ => {}
        }
    }

    fn drop_item(&mut self, kind: DropKind, from: Vec2) {
        if self.drops.len() >= MAX_DROPS {
            return;
        }
        let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(DROP_FLING * 0.4, DROP_FLING);
        self.drops.push(Drop {
            kind,
            pos: from + vec2(off.x, off.y * 0.6 + 4.0),
            from,
            t: 0.0,
            opened: None,
        });
    }

    fn update_fire(&mut self, map: &mut Map) {
        self.spread_clock -= TICK_DT;
        let spread = self.spread_clock <= 0.0;
        if spread {
            self.spread_clock += SPREAD_TICK;
        }
        let mut k = self.hot.len();
        while k > 0 {
            k -= 1;
            let i = self.hot[k] as usize;
            let p = &mut self.props[i];
            if p.burning == 0.0 {
                p.heat = (p.heat - HEAT_COOL * TICK_DT).max(0.0);
                if p.heat == 0.0 {
                    p.hot = false;
                    self.hot.swap_remove(k);
                }
                continue;
            }
            p.burning = (p.burning - TICK_DT).max(0.0);
            let building = matches!(p.kind, PropKind::Building(_));
            if building && !p.broken {
                p.hp -= p.max_hp * BURN_DECAY * TICK_DT;
            }
            let (rect, ended, breakable) = (
                p.rect,
                p.burning == 0.0,
                matches!(p.kind, PropKind::Breakable(_)),
            );
            if ended {
                self.burning -= 1;
                if breakable {
                    self.wreck(map, i);
                } else {
                    self.props[i].burnt = true;
                }
                self.props[i].hot = false;
                self.hot.swap_remove(k);
                continue;
            }
            if building && !self.props[i].broken && self.props[i].hp <= 0.0 {
                self.wreck(map, i);
            }
            if building && spread {
                self.spread_from(rect, i);
            }
        }
    }

    /// A burning building warms the burnables around it.
    fn spread_from(&mut self, rect: Rect, from: usize) {
        let near = Rect::new(
            rect.x - SPREAD_RANGE,
            rect.y - SPREAD_RANGE,
            rect.w + SPREAD_RANGE * 2.0,
            rect.h + SPREAD_RANGE * 2.0,
        );
        let (c0, r0, c1, r1) = chunk_span(self.origin, self.chunk_cols, self.chunk_rows, near);
        for r in r0..=r1 {
            for c in c0..=c1 {
                let ch = r * self.chunk_cols + c;
                for n in self.chunk_start[ch]..self.chunk_start[ch + 1] {
                    let j = self.chunk_items[n as usize] as usize;
                    let q = &mut self.props[j];
                    if j == from || q.burning > 0.0 || !q.burns() || !q.rect.overlaps(&near) {
                        continue;
                    }
                    // A prop sits in every chunk it touches: warm it from the first one
                    // both share only.
                    let (qc, qr, _, _) =
                        chunk_span(self.origin, self.chunk_cols, self.chunk_rows, q.rect);
                    if (c, r) != (qc.max(c0), qr.max(r0)) {
                        continue;
                    }
                    q.heat += SPREAD_HEAT;
                    self.warm(j);
                }
            }
        }
    }

    /// Homes near Veyra let out wanderers; when she comes to the door the folk inside run.
    fn update_homes(&mut self, veyra: Vec2, alive: bool) {
        for h in 0..self.homes.len() {
            let door = self.props[self.homes[h].prop as usize].door();
            let d2 = door.distance_squared(veyra);
            if !alive || d2 > WANDER_RANGE * WANDER_RANGE {
                continue;
            }
            if self.homes[h].wanderers > 0 && self.civilians.len() < MAX_CIVILIANS {
                self.homes[h].wanderers -= 1;
                self.let_out(h, true);
            }
            if d2 <= PANIC_RANGE * PANIC_RANGE
                && self.homes[h].folk > 0
                && self.civilians.len() < MAX_CIVILIANS
            {
                self.homes[h].folk -= 1;
                self.let_out(h, false);
            }
        }
    }

    /// Fills the villager pool from the homes nearest `p`, whatever their stock (stress test).
    #[cfg(test)]
    pub fn crowd(&mut self, p: Vec2) {
        let mut near: Vec<usize> = (0..self.homes.len()).collect();
        let door = |v: &Self, h: usize| v.props[v.homes[h].prop as usize].door();
        near.sort_by(|&a, &b| {
            let (da, db) = (door(self, a).distance(p), door(self, b).distance(p));
            da.total_cmp(&db)
        });
        for k in 0..MAX_CIVILIANS {
            if let Some(&h) = near.get(k % near.len().max(1)) {
                self.let_out(h, true);
            }
        }
    }

    /// One villager steps out of home `h`'s door: a wanderer of any kind, or folk running.
    fn let_out(&mut self, h: usize, wanderer: bool) {
        if self.civilians.len() >= MAX_CIVILIANS {
            return;
        }
        let door = self.props[self.homes[h].prop as usize].door();
        let kind = if wanderer {
            pick(&WANDER_WEIGHTS)
        } else {
            pick(&WANDER_WEIGHTS[..CHICKEN as usize])
        };
        let pos = door + vec2(gen_range(-6.0, 6.0), gen_range(0.0, 4.0));
        self.civilians.push(Civilian {
            pos,
            kind: kind as u8,
            facing: if gen_range(0, 2) == 0 { -1.0 } else { 1.0 },
            stride: gen_range(0.0, 12.0),
            flee: if wanderer { 0.0 } else { FLEE_KEEP },
            walking: false,
            home: h as u16,
            wanderer,
            target: pos,
            pause: gen_range(0.0, 1.5),
        });
    }

    fn update_civilians(&mut self, map: &Map, veyra: Vec2) {
        let mut k = self.civilians.len();
        while k > 0 {
            k -= 1;
            let c = &mut self.civilians[k];
            let off = c.pos - veyra;
            let d2 = off.length_squared();
            if d2 > CIVILIAN_DESPAWN * CIVILIAN_DESPAWN {
                let (home, wanderer, folk) = (c.home as usize, c.wanderer, c.folk());
                self.civilians.swap_remove(k);
                let h = &mut self.homes[home];
                if !self.props[h.prop as usize].broken {
                    if wanderer {
                        h.wanderers = h.wanderers.saturating_add(1);
                    } else if folk {
                        h.folk = h.folk.saturating_add(1);
                    }
                }
                continue;
            }
            if d2 < FLEE_RANGE * FLEE_RANGE {
                c.flee = FLEE_KEEP;
            }
            let kind = c.kind as usize;
            let vel = if c.flee > 0.0 {
                c.flee -= TICK_DT;
                let away = off.normalize_or(Vec2::X);
                // A little sideways, so a crowd fans out instead of running in file.
                let side = if k.is_multiple_of(2) { 0.35 } else { -0.35 };
                (away + away.perp() * side).normalize() * CIVILIAN_RUN[kind]
            } else if c.pause > 0.0 {
                c.pause -= TICK_DT;
                Vec2::ZERO
            } else {
                let to = c.target - c.pos;
                if to.length_squared() < 4.0 {
                    let home = self.props[self.homes[c.home as usize].prop as usize].door();
                    let r = gen_range(10.0, WANDER_LEASH);
                    c.target = home + Vec2::from_angle(gen_range(0.0, TAU)) * vec2(r, r * 0.6);
                    c.pause = gen_range(0.8, 3.0);
                    Vec2::ZERO
                } else {
                    to.normalize() * CIVILIAN_WALK[kind]
                }
            };
            c.walking = vel != Vec2::ZERO;
            if c.walking {
                let next = map.push_out(c.pos + vel * TICK_DT, CIVILIAN_RADIUS);
                let moved = next.distance(c.pos);
                if moved < vel.length() * TICK_DT * 0.3 && c.flee <= 0.0 {
                    c.target = c.pos; // walked into something: pick another stroll
                }
                c.pos = next;
                c.stride += moved;
                if vel.x.abs() > 1.0 {
                    c.facing = vel.x.signum();
                }
            }
        }
    }

    fn update_drops(&mut self, veyra: Vec2) {
        let mut k = self.drops.len();
        while k > 0 {
            k -= 1;
            let d = &mut self.drops[k];
            d.t += TICK_DT;
            if let Some(t) = &mut d.opened {
                *t += TICK_DT;
                if *t >= COFFER_LINGER {
                    self.drops.swap_remove(k);
                }
                continue;
            }
            if d.t < DROP_SETTLE || d.pos.distance_squared(veyra) > DROP_PICKUP * DROP_PICKUP {
                continue;
            }
            let (kind, pos) = (d.kind, d.pos);
            if kind == DropKind::Coffer {
                d.opened = Some(0.0);
            } else {
                self.drops.swap_remove(k);
            }
            self.happen(Happening::Picked { kind, pos });
        }
    }
}

/// Index picked with probability proportional to its weight.
fn pick(weights: &[f32]) -> usize {
    let mut r = gen_range(0.0, weights.iter().sum::<f32>());
    for (k, &w) in weights.iter().enumerate() {
        if r < w {
            return k;
        }
        r -= w;
    }
    weights.len() - 1
}

/// The chunks `r` covers (clamped to the grid): (col0, row0, col1, row1).
fn chunk_span(origin: Vec2, cols: usize, rows: usize, r: Rect) -> (usize, usize, usize, usize) {
    let cell = |v: f32, n: usize| ((v / PROP_CHUNK).floor().max(0.0) as usize).min(n - 1);
    (
        cell(r.x - origin.x, cols),
        cell(r.y - origin.y, rows),
        cell(r.right() - origin.x, cols),
        cell(r.bottom() - origin.y, rows),
    )
}

fn circle_hits_rect(c: Vec2, radius: f32, r: Rect) -> bool {
    let near = vec2(c.x.clamp(r.x, r.right()), c.y.clamp(r.y, r.bottom()));
    near.distance_squared(c) <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cottage over two rows of building cells, a barrel and a tree, on open grass.
    const TEST_MAP: &str = "size 12 10\nstart 1 1\ngrid\n\
        ............\n\
        ............\n\
        ............\n\
        ............\n\
        ....HHHH....\n\
        ....HHHH....\n\
        ............\n\
        ............\n\
        ............\n\
        ............\n\
        props\nhouse_cottage 64 96\nbarrel 48 110\ntree_oak 144 150\nlamp 0 40\n";

    fn village() -> (Map, Village) {
        let map = Map::parse(TEST_MAP);
        let v = Village::new(&map, TEST_MAP);
        (map, v)
    }

    fn hit(v: &mut Village, map: &mut Map, pos: Vec2, damage: f32, fire: bool, t: &mut f32) {
        v.marks.push(Mark {
            pos,
            radius: 6.0,
            damage,
            fire,
        });
        *t += PROP_HIT_COOLDOWN + TICK_DT;
        v.update(map, vec2(1000.0, 1000.0), *t, true);
    }

    #[test]
    fn layout_props_become_interactive_ones() {
        let (_, v) = village();
        assert_eq!(v.props.len(), 3, "the lamp stays scenery");
        assert_eq!(v.of_layout(3), None);
        let house = &v.props[v.of_layout(0).unwrap()];
        assert_eq!(house.kind, PropKind::Building(0));
        assert_eq!(house.rect.size(), vec2(54.0, 48.0));
        for b in BUILDINGS
            .iter()
            .map(|b| b.name)
            .chain(BREAKABLES.iter().map(|b| b.name))
        {
            assert!(sprite_size(b).is_some(), "{b}");
            assert!(
                sprite_size(&format!("{b}_cracked")).is_some(),
                "{b}_cracked"
            );
        }
        for b in BUILDINGS.iter().map(|b| b.name) {
            for s in ["_burnt", "_burnt_cracked", "_rubble", "_rubble_burnt"] {
                assert!(sprite_size(&format!("{b}{s}")).is_some(), "{b}{s}");
            }
        }
        for p in CHARRING_PLANTS {
            assert!(sprite_size(&format!("{p}_burnt")).is_some(), "{p}_burnt");
        }
        for s in CIVILIAN_SPRITES {
            assert!(sprite_size(s).is_some(), "{s}");
        }
    }

    #[test]
    fn houses_crack_then_fall_to_rubble_that_lets_shots_through() {
        let (mut map, mut v) = village();
        let i = v.of_layout(0).unwrap();
        let mid = v.props[i].rect.center();
        let cell = map.origin + vec2(5.5, 5.5) * MAP_CELL;
        assert!(map.blocks_shot(cell) && map.blocks_walk(cell));
        let mut t = 0.0;
        let hp = v.props[i].max_hp;
        hit(&mut v, &mut map, mid, hp * 0.6, false, &mut t);
        assert!(v.props[i].cracked() && !v.props[i].broken);
        // Marks inside the cooldown don't stack.
        v.marks.push(Mark {
            pos: mid,
            radius: 4.0,
            damage: hp,
            fire: false,
        });
        v.update(&mut map, vec2(1000.0, 1000.0), t, true);
        assert!(!v.props[i].broken);
        hit(&mut v, &mut map, mid, hp, false, &mut t);
        assert!(v.props[i].broken);
        assert!(!map.blocks_shot(cell) && map.blocks_walk(cell));
        assert!(v.happenings.contains(&Happening::Collapsed {
            rect: v.props[i].rect
        }));
        assert!(v.drops.iter().any(|d| d.kind == DropKind::Coin));
    }

    #[test]
    fn fire_catches_burns_out_and_spreads_from_buildings_only() {
        let (mut map, mut v) = village();
        let (house, tree) = (v.of_layout(0).unwrap(), v.of_layout(2).unwrap());
        let mut t = 0.0;
        // A plain hit leaves the tree alone; it takes a few touches of fire.
        let at = v.props[tree].rect.center();
        hit(&mut v, &mut map, at, 5.0, false, &mut t);
        assert_eq!(v.props[tree].burning, 0.0);
        let mut touches = 0;
        while v.props[tree].burning == 0.0 && touches < 20 {
            hit(&mut v, &mut map, at, 5.0, true, &mut t);
            touches += 1;
        }
        assert!(
            touches >= PLANT_IGNITE as usize && v.burning() == 1,
            "{touches}"
        );
        // A house many more.
        let mid = v.props[house].rect.center();
        let mut touches = 0;
        while v.props[house].burning == 0.0 && touches < 40 {
            hit(&mut v, &mut map, mid, 0.0, true, &mut t);
            touches += 1;
        }
        assert!(touches >= BUILDING_IGNITE as usize, "{touches}");
        assert!(v.props[house].burning > 0.0);
        let hp = v.props[house].hp;
        for _ in 0..(BUILDING_BURN_TIME / TICK_DT) as usize + 2 {
            v.update(&mut map, vec2(1000.0, 1000.0), t, true);
        }
        let p = &v.props[house];
        assert!(p.burnt && p.burning == 0.0 && p.hp < hp);
        assert!(v.props[tree].burnt);
        assert_eq!(v.burning(), 0);
        // The barrel beside the house caught from it and burned away.
        let barrel = &v.props[v.of_layout(1).unwrap()];
        assert!(barrel.broken, "fire spread to the barrel");
    }

    #[test]
    fn villagers_wander_flee_die_and_go_home() {
        let (mut map, mut v) = village();
        let i = v.of_layout(0).unwrap();
        let door = v.props[i].door();
        let far = door + vec2(200.0, 0.0);
        for _ in 0..10 {
            v.update(&mut map, far, 0.0, true);
        }
        assert_eq!(v.civilians.len(), HOME_WANDERERS as usize, "wanderers only");
        assert!(v.civilians.iter().all(|c| c.flee == 0.0));
        // She comes to the door: the folk run out and away.
        let near = door + vec2(30.0, 0.0);
        for _ in 0..30 {
            v.update(&mut map, near, 0.0, true);
        }
        assert_eq!(
            v.civilians.len(),
            (HOME_WANDERERS + BUILDINGS[0].folk) as usize
        );
        let c = v.civilians.last().unwrap();
        assert!(c.flee > 0.0 && c.pos.distance(near) > door.distance(near) + 10.0);
        // A mark on one kills it.
        let at = v.civilians[0].pos;
        v.marks.push(Mark {
            pos: at,
            radius: 2.0,
            damage: 1.0,
            fire: false,
        });
        v.update(&mut map, near, 0.0, true);
        assert!(v
            .happenings
            .iter()
            .any(|h| matches!(h, Happening::Slain { .. })));
        // Far away, the rest go home.
        let n = v.civilians.len();
        v.update(&mut map, door + vec2(5000.0, 0.0), 0.0, false);
        assert!(v.civilians.is_empty());
        let h = &v.homes[0];
        assert_eq!((h.folk + h.wanderers) as usize, n);
    }

    #[test]
    fn drops_settle_then_veyra_picks_them_up() {
        let (mut map, mut v) = village();
        v.drop_item(DropKind::Coin, vec2(0.0, 0.0));
        v.drop_item(DropKind::Coffer, vec2(0.0, 0.0));
        let at = v.drops[0].pos;
        v.update(&mut map, at, 0.0, true);
        assert!(v.happenings.is_empty(), "still flying");
        for _ in 0..(DROP_SETTLE / TICK_DT) as usize + 1 {
            v.update(&mut map, at, 0.0, true);
        }
        assert!(v.happenings.contains(&Happening::Picked {
            kind: DropKind::Coin,
            pos: at
        }));
        // The coffer opens where it lies and lingers a moment.
        let coffer = v.drops[0].pos;
        v.update(&mut map, coffer, 0.0, true);
        assert!(v.drops[0].opened.is_some());
        for _ in 0..(COFFER_LINGER / TICK_DT) as usize + 1 {
            v.update(&mut map, coffer, 0.0, true);
        }
        assert!(v.drops.is_empty());
    }
}

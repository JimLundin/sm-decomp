//! Fresh retail enemy events, observed by the verifier without gameplay writes.
use super::{Game, ReplayCoverage};
use core::mem::{offset_of, size_of};
use sm::types::EnemyData;
// ram_map's SAMUS_HEALTH, SAMUS_X/Y_POS and EPROJ_ID.
// Its word constants are crate-private; the standalone verifier reads these
// documented offsets without adding an engine API for verification.
const SAMUS_HEALTH: usize = 0x9c2;
const SAMUS_X_POS: usize = 0xaf6;
const SAMUS_Y_POS: usize = 0xafa;
const EPROJ_ID: usize = 0x1997;
const PROJECTILE_TYPE: usize = 0xc18;
const POWER_BOMB_EXPLOSION_RADIUS: usize = 0xcea;

#[derive(Clone, Copy, Default)]
struct Actor {
    definition: u16,
    bank: u8,
    properties: u16,
    frozen: u16,
    ai_handlers: u16,
    health: u16,
    x: u16,
    y: u16,
    function: u16,
    bombed_off_timer: u16,
}

#[cfg(sm_verify_dll)]
pub(super) fn check_fresh_events() {
    let mut before = Snapshot {
        health: 99,
        player_x: 100,
        player_y: 200,
        ..Snapshot::default()
    };
    before.actors[0] = Actor {
        definition: 0xdd7f,
        bank: 0xa3,
        health: 500,
        x: 100,
        y: 192,
        function: 1,
        ..Actor::default()
    };
    let mut after = before;
    after.actors[0].function = 2;
    assert!(before.metroid_latch(after));
    assert!(!after.metroid_latch(after)); // Loaded latch is no event.
    let mut nearby = after;
    nearby.actors[0].x += 7;
    assert!(before.metroid_latch(nearby));
    nearby.actors[0].x += 1;
    assert!(!before.metroid_latch(nearby)); // Contact admits strictly under 8px.
    before = after;
    after.health -= 1;
    assert!(before.metroid_drain(after));
    after.actors[0].definition = 0;
    assert!(!before.metroid_drain(after)); // Unrelated health loss.
    after.actors[0].definition = before.actors[0].definition;
    for corrupt in [0, 1, 2] {
        let mut wrong = after;
        match corrupt {
            0 => wrong.actors[0].bank = 0xa9,
            1 => wrong.actors[0].properties = 0x200,
            _ => wrong.actors[0].frozen = 1,
        }
        assert!(!before.metroid_drain(wrong));
    }
    before = Snapshot::default();
    before.actors[0] = Actor {
        definition: 0xf293,
        bank: 0xb3,
        health: 3000,
        function: 0x9ee0,
        ..Actor::default()
    };
    after = before;
    after.actors[0].function = 0x9f7a;
    after.enemy_projectiles[..5].fill(0xec48);
    assert!(before.botwoon_spit(after));
    assert!(!after.botwoon_spit(after)); // Loaded volley is no event.
    before.enemy_projectiles[0] = 0xec48;
    assert!(!before.botwoon_spit(after)); // Require every fresh birth.
    before.enemy_projectiles[0] = 0;
    after.enemy_projectiles[4] = 0;
    assert!(!before.botwoon_spit(after)); // Incomplete volley.
    before.actors[0].function = 0x9f34;
    after.enemy_projectiles[3] = 0;
    assert!(before.botwoon_spit(after));
    check_freeze_shatter();
    check_bomb_release();
}

fn check_bomb_release() {
    let mut before = Snapshot {
        player_x: 100,
        player_y: 200,
        ..Snapshot::default()
    };
    before.actors[0] = Actor {
        definition: 0xdd7f,
        bank: 0xa3,
        health: 500,
        x: 100,
        y: 192,
        function: 2,
        ..Actor::default()
    };
    before.projectile_types[5] = 0x500;
    let mut off = before;
    off.actors[0].function = 3;
    off.actors[0].bombed_off_timer = 3;
    let mut chased = off;
    chased.actors[0].function = 0;
    chased.actors[0].bombed_off_timer = 0;
    let mut episode = MetroidBombRelease::default();
    assert!(!episode.observe(off, chased, true)); // Loaded bomb-off is insufficient.
    assert!(!episode.observe(before, off, true));
    assert!(episode.observe(off, chased, true));
    assert!(!episode.observe(off, chased, true)); // One completion per episode.
    episode.observe(before, off, true);
    assert!(!episode.observe(off, chased, false));
    assert!(!episode.observe(off, chased, true));
    before.projectile_types[5] = 0;
    episode.observe(before, off, true);
    assert!(!episode.observe(off, chased, true)); // No ordinary bomb was observed.
}

fn check_freeze_shatter() {
    let mut before = Snapshot::default();
    before.actors[0] = Actor {
        definition: 0xdd7f,
        bank: 0xa3,
        health: 500,
        ..Actor::default()
    };
    let mut frozen = before;
    frozen.actors[0].frozen = 399;
    frozen.actors[0].ai_handlers = 4;
    assert!(before.metroid_freeze(frozen));
    assert!(!frozen.metroid_freeze(frozen));
    let mut dead = frozen;
    dead.actors[0] = Actor::default();
    dead.enemy_projectiles[0] = 0xf345;
    frozen.projectile_types[0] = 0x100;
    let mut episode = MetroidShatter::default();
    assert!(!episode.observe(frozen, dead, true)); // Loaded freeze is insufficient.
    assert!(!episode.observe(before, frozen, true));
    for competing in [0x200, 0x500] {
        let mut wrong = frozen;
        wrong.projectile_types[1] = competing;
        assert!(!episode.observe(wrong, frozen, true));
        assert!(!episode.observe(wrong, dead, true));
        episode.observe(before, frozen, true);
    }
    assert!(episode.observe(frozen, dead, true));
    episode.observe(before, frozen, true);
    assert!(!episode.observe(frozen, dead, false)); // Transition resets the episode.
    assert!(!episode.observe(frozen, dead, true));
}

impl Actor {
    fn same_live_actor(self, next: Self, definition: u16, bank: u8) -> bool {
        self.definition == definition
            && next.definition == definition
            && self.bank == bank
            && next.bank == bank
            && (self.properties | next.properties) & 0x200 == 0
            && self.health != 0
            && next.health != 0
            && self.frozen == 0
            && next.frozen == 0
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Snapshot {
    actors: [Actor; 32],
    health: u16,
    player_x: u16,
    player_y: u16,
    enemy_projectiles: [u16; 18],
    projectile_types: [u16; 10],
    power_bomb_radius: u16,
}

impl Snapshot {
    pub(super) fn read(game: &Game) -> Self {
        let word = |address| ReplayCoverage::word(game, address);
        Self {
            actors: core::array::from_fn(|slot| {
                let base = 0xf78 + slot * size_of::<EnemyData>();
                let field = |offset| word(base + offset);
                Actor {
                    definition: field(offset_of!(EnemyData, enemy_ptr)),
                    bank: game.ram().read_byte(base + offset_of!(EnemyData, bank)),
                    properties: field(offset_of!(EnemyData, properties)),
                    frozen: field(offset_of!(EnemyData, frozen_timer)),
                    ai_handlers: field(offset_of!(EnemyData, ai_handler_bits)),
                    health: field(offset_of!(EnemyData, health)),
                    x: field(offset_of!(EnemyData, x_pos)),
                    y: field(offset_of!(EnemyData, y_pos)),
                    function: field(offset_of!(EnemyData, ai_preinstr)),
                    bombed_off_timer: field(offset_of!(EnemyData, ai_var_e)),
                }
            }),
            health: word(SAMUS_HEALTH),
            player_x: word(SAMUS_X_POS),
            player_y: word(SAMUS_Y_POS),
            enemy_projectiles: core::array::from_fn(|slot| word(EPROJ_ID + slot * 2)),
            projectile_types: core::array::from_fn(|slot| word(PROJECTILE_TYPE + slot * 2)),
            power_bomb_radius: word(POWER_BOMB_EXPLOSION_RADIUS),
        }
    }

    fn metroid_latched(self, actor: Actor) -> bool {
        actor.function == 2 && actor.x == self.player_x && actor.y == self.player_y.wrapping_sub(8)
    }

    pub(super) fn metroid_latch(self, next: Self) -> bool {
        self.actors
            .into_iter()
            .zip(next.actors)
            .any(|(before, after)| {
                before.same_live_actor(after, 0xdd7f, 0xa3)
                    && before.function < 2
                    && after.function == 2
                    && (after.x.wrapping_sub(next.player_x) as i16).unsigned_abs() < 8
                    && (after.y.wrapping_sub(next.player_y.wrapping_sub(8)) as i16).unsigned_abs()
                        < 8
            })
    }

    pub(super) fn metroid_drain(self, next: Self) -> bool {
        self.health.checked_sub(next.health) == Some(1)
            && self
                .actors
                .into_iter()
                .zip(next.actors)
                .any(|(before, after)| {
                    before.same_live_actor(after, 0xdd7f, 0xa3)
                        && self.metroid_latched(before)
                        && next.metroid_latched(after)
                })
    }

    pub(super) fn botwoon_spit(self, next: Self) -> bool {
        let born = self
            .enemy_projectiles
            .into_iter()
            .zip(next.enemy_projectiles)
            .filter(|&(before, after)| before == 0 && after == 0xec48)
            .count();
        self.actors
            .into_iter()
            .zip(next.actors)
            .any(|(before, after)| {
                before.same_live_actor(after, 0xf293, 0xb3)
                    && after.function == 0x9f7a
                    && match before.function {
                        0x9ee0 => born == 5,
                        0x9f34 => born == 3,
                        _ => false,
                    }
            })
    }

    fn metroid_freezes(before: Actor, after: Actor) -> bool {
        before.definition == 0xdd7f
            && after.definition == before.definition
            && before.bank == 0xa3
            && after.bank == before.bank
            && (before.properties | after.properties) & 0x200 == 0
            && before.health != 0
            && after.health == before.health
            && before.frozen == 0
            // Contact freezes for 400; frozen AI runs later in that same tick.
            && after.frozen == 399
            && after.ai_handlers & 4 != 0
    }

    pub(super) fn metroid_freeze(self, next: Self) -> bool {
        self.actors
            .into_iter()
            .zip(next.actors)
            .any(|(before, after)| Self::metroid_freezes(before, after))
    }
}

/// A loaded frozen enemy cannot establish the freeze-to-shatter episode.
#[derive(Default)]
pub(super) struct MetroidShatter {
    frozen_here: [bool; 32],
}

impl MetroidShatter {
    pub(super) fn observe(&mut self, before: Snapshot, after: Snapshot, same_room: bool) -> bool {
        if !same_room {
            self.frozen_here.fill(false);
            return false;
        }
        let ordinary_missile = before
            .projectile_types
            .into_iter()
            .any(|kind| kind & 0xf00 == 0x100);
        let competing_shot = before
            .projectile_types
            .into_iter()
            .any(|kind| matches!(kind & 0xf00, 0x200 | 0x500))
            || before.power_bomb_radius != 0
            || after.power_bomb_radius != 0;
        let explosion = before
            .enemy_projectiles
            .into_iter()
            .zip(after.enemy_projectiles)
            .any(|(old, new)| old == 0 && new == 0xf345);
        let mut shattered = false;
        for ((seen, old), new) in self
            .frozen_here
            .iter_mut()
            .zip(before.actors)
            .zip(after.actors)
        {
            if *seen
                && old.definition == 0xdd7f
                && old.bank == 0xa3
                && old.health != 0
                && old.frozen != 0
                && old.properties & 0x200 == 0
                && new.definition == 0
                && new.health == 0
                && ordinary_missile
                && !competing_shot
                && explosion
            {
                shattered = true;
            }
            if new.definition != 0xdd7f
                || new.bank != 0xa3
                || new.health == 0
                || new.frozen == 0
                || new.properties & 0x200 != 0
            {
                *seen = false;
            } else if Snapshot::metroid_freezes(old, new) {
                *seen = true;
            }
        }
        shattered
    }
}

/// A release completes only after an observed bomb hit and return to chasing.
#[derive(Default)]
pub(super) struct MetroidBombRelease {
    bombed_here: [bool; 32],
}

impl MetroidBombRelease {
    pub(super) fn observe(&mut self, before: Snapshot, after: Snapshot, same_room: bool) -> bool {
        if !same_room {
            self.bombed_here.fill(false);
            return false;
        }
        let bomb = before
            .projectile_types
            .into_iter()
            .any(|kind| kind & 0xf00 == 0x500);
        let mut released = false;
        for ((seen, old), new) in self
            .bombed_here
            .iter_mut()
            .zip(before.actors)
            .zip(after.actors)
        {
            if !old.same_live_actor(new, 0xdd7f, 0xa3) {
                *seen = false;
                continue;
            }
            if *seen && old.function == 3 && new.function == 0 && new.bombed_off_timer == 0 {
                released = true;
                *seen = false;
            } else if bomb
                && before.metroid_latched(old)
                && new.function == 3
                && new.bombed_off_timer == 3
            {
                // Shot sets four frames; the same tick's AI consumes the first.
                *seen = true;
            } else if new.function != 3 {
                *seen = false;
            }
        }
        released
    }
}

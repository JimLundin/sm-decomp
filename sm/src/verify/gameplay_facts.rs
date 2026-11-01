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

#[derive(Clone, Copy, Default)]
struct Actor {
    definition: u16,
    bank: u8,
    properties: u16,
    frozen: u16,
    health: u16,
    x: u16,
    y: u16,
    function: u16,
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
                    health: field(offset_of!(EnemyData, health)),
                    x: field(offset_of!(EnemyData, x_pos)),
                    y: field(offset_of!(EnemyData, y_pos)),
                    function: field(offset_of!(EnemyData, ai_preinstr)),
                }
            }),
            health: word(SAMUS_HEALTH),
            player_x: word(SAMUS_X_POS),
            player_y: word(SAMUS_Y_POS),
            enemy_projectiles: core::array::from_fn(|slot| word(EPROJ_ID + slot * 2)),
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
}

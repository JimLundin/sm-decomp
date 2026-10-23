//! Actual shared flag producers and consumers; no codec roundtrip between calls.
use super::game::{enemy_projectiles as enemy, gameplay_flag_host::BossWordOrder, progress_flags};
use super::{Composed, Event, animated, colors, room};

fn plm_visual() {
    let mut h = Composed::new();
    h.world.player.flag_compatibility.bitmask = 0xa534;
    h.word(0x84, 0x9100, 64);
    room::command(&mut h, 8685630, 0x9100, 0);
    assert_eq!(
        h.world.player.flag_compatibility.bitmask, 1,
        "flags.scratch.full_mask"
    );
    h.byte(0x87, 0x9200, 1);
    h.word(0x87, 0x9201, 0x9800);
    let branch = animated::command(&mut h, 8880416, 0, 0x9200);
    assert_eq!(
        (branch, h.world.player.gameplay_flags.bosses_by_area[0]),
        (0x9800, 1),
        "flags.plm.animation_event_alias"
    );
    h.world.player.flag_compatibility.bitmask = 0xa534;
    h.byte(0x87, 0x9300, 2);
    animated::command(&mut h, 8880435, 0, 0x9300);
    assert_eq!(
        h.world.player.flag_compatibility.bitmask, 0xa502,
        "flags.scratch.boss_low_only"
    );
    h.world.player.visual.palette_effects.slots[0].definition = 0x8123;
    colors::pre(&mut h, 9301701, 0);
    assert_eq!(
        (
            h.world.player.gameplay_flags.bosses_by_area[0],
            h.world.player.visual.palette_effects.slots[0].definition
        ),
        (3, 0),
        "flags.animation.palette_boss"
    );
    assert_eq!(
        h.world.player.flag_compatibility.bitmask, 0xa502,
        "flags.scratch.raw_unchanged"
    );
}
fn collections() {
    let mut h = Composed::new();
    h.world.player.mechanisms.slots[0].room_argument = 9;
    room::command(&mut h, 8685669, 0x9100, 0);
    h.world.player.mechanisms.slots[0].room_argument = 18;
    room::command(&mut h, 8685721, 0x9100, 0);
    h.word(0x84, 0x9100, 0x9800);
    h.world.player.mechanisms.slots[0].room_argument = 9;
    let object_hit = room::flag_command(&mut h, 8685640, 0x9100, 0);
    let item_miss = room::flag_command(&mut h, 8685692, 0x9100, 0);
    h.world.player.mechanisms.slots[0].room_argument = 18;
    let object_miss = room::flag_command(&mut h, 8685640, 0x9100, 0);
    let item_hit = room::flag_command(&mut h, 8685692, 0x9100, 0);
    assert_eq!(
        (object_hit, item_miss, object_miss, item_hit),
        (Some(0x9800), Some(0x9102), Some(0x9102), Some(0x9800)),
        "flags.plm.collection_identity"
    );
    assert_eq!(
        (
            h.world.player.gameplay_flags.used_room_objects[1],
            h.world.player.gameplay_flags.collected_locations[2],
            h.world.player.flag_compatibility.bitmask
        ),
        (2, 4, 4),
        "flags.plm.collection_bytes"
    );
}
fn change_scratch(h: &mut Composed) {
    h.world.player.flag_compatibility.bitmask = 0x56;
}
fn rng_scratch() {
    for identity in [8836577, 8836762] {
        let mut h = Composed::new();
        h.world.player.flag_compatibility.bitmask = 0x34;
        h.world.player.enemy_projectiles.slots[0].velocity.y = 0x1234;
        h.enemy_random.push_back(0xabcd);
        h.on_enemy_random = Some(change_scratch);
        let next = enemy::flag_command(&mut h, identity, 0, 0x9000);
        assert_eq!(
            (
                h.world.player.enemy_projectiles.slots[0].velocity.x,
                h.world.player.flag_compatibility.bitmask,
                next
            ),
            (0x56ab, 0x56, 0x31000),
            "flags.eproj.rng_live_scratch"
        );
        assert_eq!(
            h.world.player.enemy_projectiles.slots[0].velocity.y,
            if identity == 8836577 { 0xc0 } else { 0x1234 },
            "flags.eproj.rng_y_contract"
        );
        assert_eq!(h.events, [Event::EnemyRandom]);
    }
}
fn eye_door() {
    for opened in [false, true] {
        let mut h = Composed::new();
        h.world.player.room.width = 16;
        h.world.player.flag_compatibility.bitmask = 0x20;
        h.world.player.gameplay_flags.opened_doors[0] = if opened { 0x80 } else { 0 };
        let slot = &mut h.world.player.enemy_projectiles.slots[0];
        slot.x.whole = 64;
        slot.y.whole = 64;
        slot.radius = 0x0101;
        slot.program = 0x9000;
        slot.delay = 7;
        slot.behavior_b = 7;
        enemy::eye_door(&mut h, 0);
        let slot = &h.world.player.enemy_projectiles.slots[0];
        assert_eq!(
            (
                slot.program,
                slot.delay,
                h.world.player.flag_compatibility.bitmask
            ),
            (
                if opened { 0xb5f3 } else { 0x9000 },
                if opened { 1 } else { 7 },
                0x80
            ),
            "flags.eproj.eye_door_current_mask"
        );
    }
}
fn door511() {
    let mut h = Composed::new();
    h.world.player.gameplay_flags.opened_doors[63] = 1;
    h.write_flag_byte(0xd8f0, 0x5a);
    h.world.player.mechanisms.slots[0].room_argument = 511;
    h.byte(0x84, 0x9100, 1);
    h.word(0x84, 0x9101, 0x9800);
    let next = room::flag_command(&mut h, 8686225, 0x9100, 0);
    assert_eq!(
        (
            h.world.player.gameplay_flags.opened_doors[63],
            h.flag_byte_at(0xd8f0),
            h.world.player.mechanisms.slots[0].room_argument,
            h.world.player.mechanisms.slots[0].pre_action,
            h.world.player.flag_compatibility.bitmask,
            next
        ),
        (0x81, 0x5a, 0xda81, 0x8aa6, 0x80, Some(0x9800)),
        "flags.plm.door511_adjacent"
    );
}
fn aliases() {
    let mut h = Composed::new();
    h.write_flag_byte(0xf81f, 1);
    h.word(0x84, 0x9100, 65535);
    room::command(&mut h, 8685630, 0x9100, 0);
    assert_eq!(
        (
            h.flag_byte_at(0xf81f),
            h.world.player.flag_compatibility.bitmask
        ),
        (0x81, 0x80),
        "flags.alias.event_wide_index"
    );
    h.write_flag_byte(0x1d827, 0x7e);
    assert_eq!(
        progress_flags::boss_byte_at(&h, 65535),
        0x7e,
        "flags.alias.boss_wide_index"
    );
    h.world.player.gameplay_flags.bosses_by_area[7] = 0x12;
    h.world.player.gameplay_flags.used_room_objects[0] = 0x34;
    let word = progress_flags::boss_word_at(&h, 7, BossWordOrder::Native);
    assert_eq!(
        (word, h.world.player.flag_compatibility.bitmask),
        (u16::from_ne_bytes([0x12, 0x34]), 0x80),
        "flags.alias.boss_word_cross_owner"
    );
    h.world.player.room.area = 7;
    h.world.player.visual.palette_effects.slots[0].definition = 0x8123;
    colors::pre(&mut h, 9301701, 0);
    assert_eq!(
        (
            h.world.player.visual.palette_effects.slots[0].definition,
            h.world.player.flag_compatibility.bitmask
        ),
        (0, 0x80),
        "flags.alias.raw_palette_consumer"
    );
}
pub(super) fn run_selected(case: &str) {
    match case {
        "plm_visual" => plm_visual(),
        "collections" => collections(),
        "rng_scratch" => rng_scratch(),
        "eye_door" => eye_door(),
        "door511" => door511(),
        "aliases" => aliases(),
        _ => panic!("unknown flags composition case: {case}"),
    }
}
pub(super) fn run() {
    for case in [
        "plm_visual",
        "collections",
        "rng_scratch",
        "eye_door",
        "door511",
        "aliases",
    ] {
        run_selected(case);
    }
}

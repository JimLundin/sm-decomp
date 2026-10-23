//! Actual enemy/room/player algorithms over one Session and synthetic content.
use super::game::{enemy_projectiles as enemy, player_projectiles as player};
use super::{Composed, Event, composition_checks::missile_content, player_checks::prepared, room};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn definition(h: &Composed, address: u16, initialize: u32, properties: u16) {
    // Packed definition words: init, pre, program, radius, properties, hit, shot.
    for (offset, value) in [
        (0, initialize as u16),
        (2, 0x84fb),
        (4, 0x9500),
        (6, 0x0202),
        (8, properties),
        (10, 0x9520),
        (12, 0x9540),
    ] {
        h.word(0x86, address.wrapping_add(offset), value);
    }
}
fn scheduler_fixture(h: &mut Composed, command: u32, operand: Option<u16>) {
    h.world.player.mechanisms.flags = 0x8000;
    let slot = &mut h.world.player.mechanisms.slots[1];
    slot.header = 1;
    slot.instruction_delay = 1;
    slot.pre_action = 8684649u32 as u16;
    slot.program = 0x9800;
    slot.block = 202;
    slot.room_argument = 1234;
    h.world.player.mechanisms.slots[3].program = 0x8888;
    h.word(0x84, 0x9800, command as u16);
    let stop = if let Some(value) = operand {
        h.word(0x84, 0x9802, value);
        0x9804
    } else {
        0x9802
    };
    h.word(0x84, stop, 8685236u32 as u16);
    h.word(0x84, 0x9900, 8685249u32 as u16);
    h.word(0x84, 0x9902, 0x1234);
    h.word(0x84, 0x9904, 8685236u32 as u16);
}
fn redirect_room(h: &mut Composed) {
    h.world.player.mechanisms.selected = 6;
    h.world.player.mechanisms.slots[1].header = 0;
    let observer = &mut h.world.player.mechanisms.slots[2];
    observer.header = 1;
    observer.instruction_delay = 1;
    observer.pre_action = 8684649u32 as u16;
    observer.program = 0x9900;
}
fn observe_eye_sound(h: &mut Composed) {
    let projectile = &h.world.player.enemy_projectiles.slots[17];
    assert_eq!(
        (
            projectile.x.whole,
            projectile.y.whole,
            projectile.behavior_b
        ),
        (72, 64, u16::from_ne_bytes(1234u16.to_le_bytes())),
        "1050 setup before sound"
    );
    assert_eq!(h.world.player.enemy_projectiles.spawn.primary, 0);
    assert_eq!(h.events.last(), Some(&Event::RoomSound(2, 6, 0x4c)));
    redirect_room(h);
}
fn room_enemy_room() {
    let mut h = prepared();
    definition(&h, 0xb743, 8828461, 0);
    scheduler_fixture(&mut h, 8705914, Some(0));
    h.on_room_sound = Some(observe_eye_sound);
    let mut pass = room::host::Pass::Instructions;
    assert!(matches!(
        room::advance(&mut h, &mut pass),
        room::host::RunResult::Complete
    ));
    assert_eq!(
        h.world.player.mechanisms.slots[1].program, 0x9800,
        "1050 captured old program"
    );
    assert_eq!(
        h.world.player.mechanisms.slots[3].program, 0x9804,
        "1050 live selected after enemy return"
    );
    assert_eq!(
        h.world.player.mechanisms.slots[2].pre_action, 0x1234,
        "1050 live outer subtraction"
    );
    assert_eq!(
        h.world.player.enemy_projectiles.slots[17].definition,
        0xb743
    );
}
fn smoke_rng(h: &mut Composed) {
    let projectile = &h.world.player.enemy_projectiles.slots[17];
    assert_eq!(
        (projectile.x.whole, projectile.y.whole),
        (91, 60),
        "1051 setup before RNG"
    );
    redirect_room(h);
    h.world.player.inventory.ammunition[0].current = 6;
}
fn nested_selection() {
    let mut h = prepared();
    definition(&h, 0xe517, 8840358, 0);
    // primary0B chooses table word11; high byte0 selects mask/offset row0.
    h.word(0x86, 0xe442, 0x9600);
    for (at, value) in [(0xe47e, 0), (0xe480, 0), (0xe482, 3), (0xe484, 4)] {
        h.word(0x86, at, value);
    }
    scheduler_fixture(&mut h, 8705974, None);
    h.enemy_random.push_back(0x4567);
    h.on_enemy_random = Some(smoke_rng);
    let mut pass = room::host::Pass::Instructions;
    assert!(matches!(
        room::advance(&mut h, &mut pass),
        room::host::RunResult::Complete
    ));
    assert_eq!(
        h.world.player.mechanisms.slots[3].program, 0x9802,
        "1051 live selected after nested RNG"
    );
    assert_eq!(h.world.player.mechanisms.slots[1].program, 0x9800);
    assert_eq!(
        h.world.player.mechanisms.slots[2].pre_action, 0x1234,
        "1051 live outer subtraction"
    );
    assert_eq!(h.world.player.inventory.ammunition[0].current, 6);
    assert_eq!(h.events, [Event::EnemyRandom]);
}
fn shot_contact() {
    for piercing in [false, true] {
        let mut h = prepared();
        for cell in h.world.player.room.terrain.cells_mut() {
            cell.level = 0;
        }
        if piercing {
            h.world.player.inventory.beams.active = 8;
            h.word(0x93, 0x83d1, 0x8a00);
            h.word(0x93, 0x8a00, 43);
            h.word(0x93, 0x8a06, 0x9000);
        }
        player::fire(&mut h);
        definition(&h, 0x9700, 0, 0x8000);
        let item = &mut h.world.player.enemy_projectiles.slots[17];
        item.definition = 0x9700;
        item.properties = 0x8000;
        item.x.whole = 41;
        item.y.whole = 33;
        enemy::receive_shots(&mut h);
        assert_eq!(
            h.world.player.projectiles.shots[0].direction,
            if piercing { 2 } else { 0x12 },
            "1052 shared shot mark"
        );
        let item = &h.world.player.enemy_projectiles.slots[17];
        assert_eq!(
            (
                item.behavior_c,
                item.program,
                item.delay,
                item.pre_action,
                item.properties
            ),
            (if piercing { 0x8008 } else { 0x8000 }, 0x9540, 1, 0x84fb, 0)
        );
        player::advance(&mut h);
        assert_eq!(
            h.world.player.projectiles.count,
            u16::from(piercing),
            "1052 actual consumer count"
        );
        if piercing {
            assert_ne!(h.world.player.projectiles.shots[0].damage, 0);
        } else {
            assert_eq!(
                h.world.player.projectiles.shots[0].damage, 0,
                "1052 actual consumer clears marked shot"
            );
        }
    }
}
fn award_sound(h: &mut Composed) {
    assert_eq!(h.events.last(), Some(&Event::EnemySound(2, 1, 3)));
    let item = &h.world.player.enemy_projectiles.slots[17];
    assert_eq!(
        (item.program, item.pre_action),
        (0x9550, 0x9560),
        "1053 reward sound before dismissal"
    );
    assert_eq!(
        h.world.player.inventory.ammunition[0].current,
        h.expected_drop_award.unwrap(),
        "1053 actual drop award before sound"
    );
    h.world.player.enemy_projectiles.selected = 0;
}
fn drop_then_missile() {
    for (before, awarded, spent) in [(0, 2, 1), (6, 7, 6)] {
        let mut h = prepared();
        missile_content(&h);
        h.world.player_inputs.player_x_radius = 8;
        h.world.player.player.contact.radius = 8;
        h.world.player.inventory.ammunition[0].current = before;
        h.world.player.inventory.ammunition[0].maximum = 7;
        let item = &mut h.world.player.enemy_projectiles.slots[17];
        item.x.whole = 30;
        item.y.whole = 32;
        item.radius = 0x0202;
        item.behavior_a = 8;
        item.behavior_b = 2;
        item.program = 0x9550;
        item.pre_action = 0x9560;
        h.expected_drop_award = Some(awarded);
        h.on_enemy_sound = Some(award_sound);
        enemy::collect(&mut h, 34);
        assert_eq!(
            h.world.player.inventory.ammunition[0].current, awarded,
            "1053 actual drop award"
        );
        let item = &h.world.player.enemy_projectiles.slots[17];
        assert_eq!(
            (item.program, item.delay, item.properties, item.pre_action),
            (0xeca3, 1, 0x3000, 0xefdf),
            "1053 captured dismissal slot"
        );
        assert_eq!(h.world.player.enemy_projectiles.slots[0].program, 0);
        h.world.player.projectiles.controls.selected_weapon = 1;
        player::fire_missile(&mut h);
        assert_eq!(
            h.world.player.inventory.ammunition[0].current, spent,
            "1053 actual missile consumes drop"
        );
        assert_eq!(
            (
                h.world.player.projectiles.shots[0].kind,
                h.world.player.projectiles.shots[0].damage
            ),
            (0x8100, 37)
        );
    }
}
fn contact_failure_prefix() {
    let mut h = prepared();
    h.world.player_inputs.player_x_radius = 8;
    h.world.player.player.contact.radius = 8;
    h.world.player.inventory.health.current = 50;
    h.world.player.inventory.health.maximum = 99;
    h.world.player.projectiles.controls.protection = 0x1234;
    definition(&h, 0x9700, 0, 0x400a);
    let item = &mut h.world.player.enemy_projectiles.slots[17];
    item.definition = 0x9700;
    item.properties = 0x400a;
    item.radius = 0x0202;
    item.x.whole = 30;
    item.y.whole = 32;
    let error = catch_unwind(AssertUnwindSafe(|| enemy::hit_player(&mut h)))
        .expect_err("1054 unprepared damage must fail");
    let reason = error
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| error.downcast_ref::<String>().map(String::as_str));
    assert_eq!(reason, Some("unprepared enemy operation: damage_player"));
    assert_eq!(
        (
            h.world.player.player.invincibility,
            h.world.player.player.knockback_timer,
            h.world.player.projectiles.controls.protection
        ),
        (96, 5, 0x1234),
        "1054 distinct contact prefix"
    );
    assert_eq!(h.world.player.inventory.health.current, 50);
    assert_eq!(h.world.player.enemy_projectiles.slots[17].program, 0x9520);
}
fn native_drop_then_missile() {
    let mut h = prepared();
    missile_content(&h);
    h.world.player_inputs.player_x_radius = 8;
    h.world.player.player.contact.radius = 8;
    h.world.player.inventory.health.current = 99;
    h.world.player.inventory.health.maximum = 99;
    h.world.player.inventory.ammunition[0].maximum = 7;
    h.world.player.enemy_projectiles.drop_bias = 1;
    h.world.player.enemy_projectiles.spawn.point.x = 67;
    h.world.player.enemy_projectiles.spawn.point.y = 81;
    definition(&h, 0xf337, 8843049, 0);
    h.word(0xa0, 0x983a, 0x9900);
    for (offset, value) in [0, 0, 255, 0, 0, 0].into_iter().enumerate() {
        h.byte(0xb4, 0x9900 + offset as u16, value);
    }
    h.word(0x86, 0xef0c, 0x9640);
    h.enemy_random.push_back(1);
    enemy::spawn_drop(&mut h, 0x9800, 0, 0);
    let item = &h.world.player.enemy_projectiles.slots[17];
    assert_eq!(
        (
            item.x.whole,
            item.y.whole,
            item.drop.enemy_definition,
            item.behavior_a,
            item.behavior_b,
            item.program
        ),
        (67, 81, 0x9800, 8, 400, 0x9640),
        "1058 native point feeds actual drop initializer"
    );
    assert_eq!(h.world.player.enemy_projectiles.drop_bias, 0);
    assert_eq!(h.events, [Event::EnemyRandom]);
    h.world.player.player.x = 67;
    h.world.player.player.y = 81;
    enemy::collect(&mut h, 34);
    assert_eq!(h.world.player.inventory.ammunition[0].current, 2);
    h.world.player.projectiles.controls.selected_weapon = 1;
    player::fire_missile(&mut h);
    assert_eq!(
        h.world.player.inventory.ammunition[0].current, 1,
        "1058 actual missile consumes initialized drop"
    );
    assert_eq!(
        (
            h.world.player.projectiles.shots[0].x,
            h.world.player.projectiles.shots[0].y
        ),
        (78, 82)
    );
}
pub(super) fn run() {
    room_enemy_room();
    nested_selection();
    shot_contact();
    drop_then_missile();
    contact_failure_prefix();
    native_drop_then_missile();
}

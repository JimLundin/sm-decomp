//! Composed actual firing, collision, animation and palette execution.
use super::{Composed, Event, World, colors, game::player_projectiles as player};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn prepared() -> Composed {
    let mut h = Composed::new();
    h.world.player.player.x = 32;
    h.world.player.player.y = 32;
    h.world.player.player.grapple = 0xc4f0;
    h.world.player_inputs.shoot_button = 1;
    h.world.player_inputs.held = 1;
    h.world.player_inputs.pressed = 1;
    h.world.player_inputs.room_width = 32;
    h.world.player_inputs.room_scroll_width = 2;
    h.world.player_inputs.room_scroll_height = 2;
    h.world.player_inputs.room_bytes = 2048;
    h.byte(0x91, 0xb62c, 2); // Prepared pose zero: fire right.
    h.byte(0x91, 0xb62d, 0);
    for (cursor, value) in [
        (0x83c1, 0x8800),
        (0x8800, 20),
        (0x8806, 0x9000),
        (0x9000, 100),
        (0x9002, 0x9234),
        (0x9004, 0x0202),
        (0x867b, 0x9100),
        (0x9100, 100),
        (0x9102, 0x9345),
        (0x9104, 0x0202),
    ] {
        h.word(0x93, cursor, value);
    }
    // Only plain solid terrain is prepared; shootable mechanisms are rejected.
    for row in 0..32 {
        h.terrain[row * 32 + 3] = 0x8000;
    }
    h
}

fn finish_flight(h: &mut Composed) {
    player::advance(h);
    assert_eq!(
        (
            h.world.player.shots[0].x,
            h.world.player.shots[0].x_fraction
        ),
        (47, 0x1000)
    );
    player::advance(h);
    let shot = &h.world.player.shots[0];
    assert_eq!((shot.x, shot.x_fraction, shot.y), (53, 0x3000, 33));
    assert_eq!(
        (shot.kind, shot.damage, shot.motion, shot.sprite),
        (0x8700, 8, 0xb169, 0x9345)
    );
    player::draw_explosions(h);
}

fn firing_and_replay() {
    let mut h = prepared();
    player::fire(&mut h);
    let shot = &h.world.player.shots[0];
    assert_eq!(
        (shot.x, shot.y, shot.damage, shot.x_speed),
        (43, 33, 20, 0x400)
    );
    assert_eq!(h.world.player.count, 1);
    assert_eq!(h.events, [Event::PlayerSound(1, 15, 0xb)]);
    let snapshot = h.world.clone();
    h.events.clear();
    finish_flight(&mut h);
    assert_eq!(
        h.events,
        [Event::PlayerSound(2, 6, 12), Event::Shot(0, 53, 33)]
    );
    let completed = h.world.clone();
    let outputs = h.events.clone();
    h.world = World::default();
    h.world = snapshot;
    h.events.clear();
    finish_flight(&mut h);
    assert_eq!(h.world, completed);
    assert_eq!(h.events, outputs);
}

fn reset_palette_composition() {
    let mut h = prepared();
    h.world.player.shots[0].x_fraction = 0xabcd;
    h.world.player.shots[0].auxiliary = 0x1234;
    h.world.player_inputs.hyper = 1;
    for (cursor, value) in [
        (0xe1f0, 9291397u32 as u16),
        (0xe1f2, 0x9000),
        (0x9000, 1),
        (0x9002, 0x3456),
        (0x9004, 9291157u32 as u16),
        (0x9006, 9291294u32 as u16),
        (0x9008, 0x9000),
    ] {
        h.word(0x8d, cursor, value);
    }
    h.world.palette.set_enabled(true);
    player::reset(&mut h);
    assert_eq!(h.events, [Event::PaletteSpawn(0xe1f0)]);
    assert_eq!(h.world.palette.effects.slots[7].id, 0xe1f0);
    assert_eq!(h.world.player.shots[0].x_fraction, 0xabcd);
    assert_eq!(h.world.player.shots[0].auxiliary, 0x1234);
    colors::advance(&mut h);
    assert_eq!(h.world.palette.colors[0], 0x3456);
    assert_eq!(
        h.events,
        [Event::PaletteSpawn(0xe1f0), Event::Color(0, 0x3456)]
    );
}

fn unsupported_does_not_succeed() {
    use super::game::player_projectiles::host::Host;
    let mut h = prepared();
    assert!(catch_unwind(AssertUnwindSafe(|| player::prepare(&mut h))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| h.spawn_mechanism(0x8000))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| h.enable_hdma())).is_err());
    assert!(h.events.is_empty());
}

pub(super) fn run() {
    firing_and_replay();
    reset_palette_composition();
    unsupported_does_not_succeed();
}

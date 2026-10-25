//! Composed actual firing, collision, animation and palette execution.
use super::{Composed, Event, World, colors, game::player_projectiles as player};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(super) fn prepared() -> Composed {
    let mut h = Composed::new();
    // The muzzle is player X + 11. Start at 41 so the first right edge
    // (45 + radius 2 - 1 = 46) remains before the wall at X = 48.
    h.world.player.player.x = 30;
    h.world.player.player.y = 32;
    h.world.player.player.grapple = 0xc4f0;
    h.world.player_inputs.shoot_button = 1;
    h.world.player_inputs.held = 1;
    h.world.player_inputs.pressed = 1;
    h.world.player.room.width = 32;
    h.world.player.room.scroll_width = 2;
    h.world.player.room.scroll_height = 2;
    h.world.player.room.size_bytes = 2048;
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
        h.world.player.room.terrain.cells_mut()[row * 32 + 3].level = 0x8000;
    }
    h
}

fn finish_flight(h: &mut Composed) {
    player::advance(h);
    // First velocity 0x0410 adds 4:1000 to the muzzle at 41:0000.
    assert_eq!(
        (
            h.world.player.projectiles.shots[0].x,
            h.world.player.projectiles.shots[0].x_fraction
        ),
        (45, 0x1000)
    );
    assert_eq!(
        (
            h.world.player.projectiles.shots[0].kind,
            h.world.player.projectiles.shots[0].damage
        ),
        (0x8000, 20)
    );
    player::advance(h);
    let shot = &h.world.player.projectiles.shots[0];
    // Next velocity 0x0420 reaches 49:3000; leading edge 50 hits column 3.
    // Beam retirement shifts the center by radius 2 before showing impact.
    assert_eq!((shot.x, shot.x_fraction, shot.y), (51, 0x3000, 33));
    assert_eq!(
        (shot.kind, shot.damage, shot.motion, shot.sprite),
        (0x8700, 8, 0xb169, 0x9345)
    );
    player::draw_explosions(h);
}

fn firing_and_replay() {
    let mut h = prepared();
    player::fire(&mut h);
    let shot = &h.world.player.projectiles.shots[0];
    assert_eq!(
        (shot.x, shot.y, shot.damage, shot.x_speed),
        (41, 33, 20, 0x400)
    );
    assert_eq!(h.world.player.projectiles.count, 1);
    assert_eq!(h.events, [Event::PlayerSound(1, 15, 0xb)]);
    let snapshot = h.world.clone();
    h.events.clear();
    finish_flight(&mut h);
    assert_eq!(
        h.events,
        [Event::PlayerSound(2, 6, 12), Event::Shot(0, 51, 33)]
    );
    let completed = h.world.clone();
    let outputs = h.events.clone();
    h.world = World::default();
    assert_eq!(h.world, World::default());
    h.world = snapshot;
    h.events.clear();
    finish_flight(&mut h);
    assert_eq!(h.world, completed);
    assert_eq!(h.events, outputs);
}

fn first_frame_boundary() {
    let mut h = prepared();
    h.world.player.player.x = 32;
    player::fire(&mut h);
    assert_eq!(h.world.player.projectiles.shots[0].x, 43);
    h.events.clear();
    player::advance(&mut h);
    // 43:0000 + 4:1000 gives center 47, but edge 47 + 2 - 1 = 48
    // is already in the wall. Retirement adds radius 2 immediately.
    let shot = &h.world.player.projectiles.shots[0];
    assert_eq!((shot.x, shot.x_fraction, shot.y), (49, 0x1000, 33));
    assert_eq!(
        (shot.kind, shot.damage, shot.motion, shot.sprite),
        (0x8700, 8, 0xb169, 0x9345)
    );
    assert_eq!(
        (shot.remaining, shot.x_speed, shot.trail_time),
        (100, 0x410, 3)
    );
    assert_eq!(h.events, [Event::PlayerSound(2, 6, 12)]);
    player::advance(&mut h);
    let shot = &h.world.player.projectiles.shots[0];
    assert_eq!((shot.x, shot.x_fraction, shot.y), (49, 0x1000, 33));
    // Null motion leaves velocity and trail time stale; playback alone ticks.
    assert_eq!(
        (shot.remaining, shot.x_speed, shot.trail_time),
        (99, 0x410, 3)
    );
    assert_eq!(h.events, [Event::PlayerSound(2, 6, 12)]);
}

fn reset_palette_composition() {
    let mut h = prepared();
    h.world.player.projectiles.shots[0].x_fraction = 0xabcd;
    h.world.player.projectiles.shots[0].auxiliary = 0x1234;
    h.world.player.inventory.hyper_beam = 1;
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
    colors::set_enabled(&mut h.world.player, true);
    player::reset(&mut h);
    assert_eq!(h.events, [Event::PaletteSpawn(0xe1f0)]);
    assert_eq!(
        h.world.player.visual.palette_effects.slots[7].definition,
        0xe1f0
    );
    assert_eq!(h.world.player.projectiles.shots[0].x_fraction, 0xabcd);
    assert_eq!(h.world.player.projectiles.shots[0].auxiliary, 0x1234);
    colors::advance(&mut h);
    assert_eq!(h.world.player.visual.colors[0], 0x3456);
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

// Captured records and synchronous callbacks in the existing composed Host.
fn trail_definitions(h: &Composed) {
    for (bank, cursor, value) in [
        (0x90, 0xb5bb, 0x8100),
        (0x90, 0xb609, 0x8400),
        (0x90, 0x8100, 2),
        (0x90, 0x8102, 0x2222),
        (0x90, 0x8400, 2),
        (0x90, 0x8402, 0x3333),
        (0x93, 0x9000, 9),
        (0x93, 0x9002, 0x1111),
        (0x93, 0x9004, 0x0705),
        (0x93, 0x9006, 2),
        (0x93, 0x8ffe, 1),
        (0x9b, 0xa4b3, 0xa800),
        (0x9b, 0xa804, 0xb000),
        (0x9b, 0xb008, 0x0806),
        (0x9b, 0xb00a, 0x0c0a),
        (0x9b, 0xb00c, 0x1210),
        (0x9b, 0xb00e, 0x1614),
    ] {
        h.word(bank, cursor, value);
    }
}
fn trail_host() -> Composed {
    let mut h = prepared();
    trail_definitions(&h);
    h.world.player.projectiles.selected = 0;
    let shot = &mut h.world.player.projectiles.shots[0];
    shot.kind = 0;
    shot.direction = 2;
    shot.cursor = 0x9000;
    shot.remaining = 1;
    shot.trail_time = 1;
    shot.x = 50;
    shot.y = 60;
    h
}
fn trail_event(h: &mut Composed, pair: usize, right: bool, frozen: bool) {
    let pair_value = &h.world.player.projectiles.trails[pair];
    let side = if right {
        &pair_value.right
    } else {
        &pair_value.left
    };
    h.events
        .push(Event::Trail(pair, right, frozen, side.x, side.y));
}
fn animation_capture_and_unknown() {
    let mut h = trail_host();
    h.on_player_record = Some(|h, bank, cursor| {
        assert_eq!((bank, cursor), (0x93, 0x9000));
        assert_eq!(h.world.player.projectiles.shots[0].remaining, 0);
        let old = h.image.clone();
        h.image = super::RomImage::zeroed(0x300000);
        h.world.player.projectiles.selected = 2;
        Some(super::RomRecord {
            image: old,
            base: 0x99000,
        })
    });
    player::animation::advance(&mut h);
    let shot = &h.world.player.projectiles.shots[0];
    assert_eq!(
        (
            shot.remaining,
            shot.sprite,
            shot.x_radius,
            shot.y_radius,
            shot.cursor
        ),
        (9, 0x1111, 5, 7, 0x9008)
    );
    assert_eq!(h.world.player.projectiles.shots[1].remaining, 0);
    let mut h = trail_host();
    h.word(0x93, 0x9000, 0x822f);
    h.world.player.projectiles.shots[1].kind = 0x3456;
    h.on_player_record = Some(|h, _, _| {
        h.world.player.projectiles.selected = 2;
        None
    });
    player::animation::advance(&mut h);
    assert_eq!(h.world.player.projectiles.shots[0].cursor, 0);
    assert_eq!(h.world.player.projectiles.shots[1].kind, 0x3456);
    for continuing in [false, true] {
        let mut h = trail_host();
        h.word(0x93, 0x9000, 0xffff);
        h.word(0x93, 0x9100, 4);
        h.word(0x93, 0x9102, 0x4444);
        h.on_player_unknown = Some(if continuing {
            |h| {
                h.world.player.projectiles.selected = 2;
                true
            }
        } else {
            |_| false
        });
        h.on_player_record = Some(|h, bank, cursor| {
            h.events.push(Event::PlayerRecord(bank, cursor));
            (cursor == 1).then(|| super::RomRecord {
                image: h.image.clone(),
                base: 0x99100,
            })
        });
        player::animation::advance(&mut h);
        assert_eq!(
            h.world.player.projectiles.shots[0].remaining,
            if continuing { 4 } else { 0 }
        );
        assert_eq!(
            h.world.player.projectiles.shots[0].cursor,
            if continuing { 9 } else { 0x9000 }
        );
        assert_eq!(h.events.len(), if continuing { 2 } else { 1 });
    }
}
fn emission_record_recapture() {
    let mut h = trail_host();
    h.on_player_record = Some(|h, bank, cursor| {
        let first = h.events.is_empty();
        h.events.push(Event::PlayerRecord(bank, cursor));
        if !first {
            return None;
        }
        let pair = &h.world.player.projectiles.trails[17];
        assert_eq!(
            (
                pair.left.remaining,
                pair.right.remaining,
                pair.left.cursor,
                pair.right.cursor
            ),
            (1, 1, 0x8100, 0x8400)
        );
        let old = h.image.clone();
        h.image = super::RomImage::zeroed(0x300000);
        trail_definitions(h);
        h.word(0x93, 0x9000, 0x8239);
        h.word(0x93, 0x9006, 3);
        h.world.player.projectiles.selected = 2;
        h.world.player.projectiles.shots[0].x = 150;
        h.world.player.projectiles.shots[0].y = 200;
        Some(super::RomRecord {
            image: old,
            base: 0x99000,
        })
    });
    player::presentation::emit(&mut h, 0);
    let pair = &h.world.player.projectiles.trails[17];
    assert_eq!(
        (pair.left.y, pair.left.x, pair.right.y, pair.right.x),
        (214, 162, 218, 166)
    );
    assert_eq!(
        h.events,
        [
            Event::PlayerRecord(0x93, 0x9000),
            Event::PlayerRecord(0x93, 0x9000),
            Event::PlayerRecord(0x9b, 0xa804),
            Event::PlayerRecord(0x9b, 0xb00c)
        ]
    );
}
fn emission_caller_reload() {
    for code in [0xaef3, 0xdc9c] {
        let mut h = trail_host();
        for cell in h.world.player.room.terrain.cells_mut() {
            cell.level = 0;
            cell.behavior = 0;
        }
        let next = &mut h.world.player.projectiles.shots[1];
        next.direction = 2;
        next.x = 100;
        next.y = 120;
        next.x_speed = 0x200;
        h.on_player_record = Some(|h, bank, cursor| {
            h.world.player.projectiles.selected = 2;
            h.events.push(Event::PlayerRecord(bank, cursor));
            None
        });
        player::call_motion(&mut h, code, 0);
        assert_eq!(h.world.player.projectiles.shots[0].x, 50);
        assert_eq!(h.world.player.projectiles.shots[0].y, 60);
        let next = &h.world.player.projectiles.shots[1];
        assert_eq!(
            (next.x, next.y),
            if code == 0xaef3 {
                (102, 120)
            } else {
                (100, 128)
            }
        );
        assert_eq!(h.world.player.projectiles.shots[0].trail_time, 4);
        let pair = &h.world.player.projectiles.trails[17];
        assert_eq!((pair.left.x, pair.left.y), (52, 64));
    }
}
fn trail_frame_recapture_and_unknown() {
    let mut h = trail_host();
    h.world.player.projectiles.trails[17].left.remaining = 1;
    h.world.player.projectiles.trails[17].left.cursor = 0x8100;
    h.on_player_record = Some(|h, bank, cursor| {
        let first = h.events.is_empty();
        h.events.push(Event::PlayerRecord(bank, cursor));
        if !first {
            assert_eq!(h.world.player.projectiles.trails[17].left.remaining, 2);
            return None;
        }
        let old = h.image.clone();
        h.image = super::RomImage::zeroed(0x300000);
        h.word(0x90, 0x8100, 8);
        h.word(0x90, 0x8102, 0x5555);
        Some(super::RomRecord {
            image: old,
            base: 0x80100,
        })
    });
    h.on_player_trail = Some(trail_event);
    player::presentation::update_trails(&mut h);
    let trail = &h.world.player.projectiles.trails[17].left;
    assert_eq!(
        (trail.remaining, trail.tile, trail.cursor),
        (2, 0x5555, 0x8104)
    );
    let mut h = trail_host();
    h.word(0x90, 0x8100, 0xffff);
    h.word(0x90, 0x8102, 2);
    h.word(0x90, 0x8104, 0x6666);
    h.world.player.projectiles.trails[17].left.remaining = 1;
    h.world.player.projectiles.trails[17].left.cursor = 0x8100;
    h.on_player_unknown = Some(|h| {
        h.world.player.projectiles.trails[17].left.y = 45;
        false
    });
    h.on_player_trail = Some(trail_event);
    player::presentation::update_trails(&mut h);
    let trail = &h.world.player.projectiles.trails[17].left;
    assert_eq!(
        (trail.remaining, trail.tile, trail.cursor, trail.y),
        (2, 0x6666, 0x8106, 45)
    );
}
fn trail_draw_reentry() {
    let mut h = trail_host();
    h.world.player_inputs.frozen = 1;
    h.world.player.projectiles.trails[17].left.remaining = 2;
    h.world.player.projectiles.trails[17].right.remaining = 3;
    h.on_player_trail = Some(|h, pair, right, frozen| {
        trail_event(h, pair, right, frozen);
        if h.events.len() == 1 {
            let mut replacement = h.world.player.clone();
            replacement.projectiles.trails[17].right.remaining = 0;
            replacement.projectiles.trails[16].left.remaining = 1;
            replacement.projectiles.trails[16].left.cursor = 0x8100;
            replacement.projectiles.trails[16].left.x = 77;
            h.world.player = replacement;
            h.world.player_inputs.frozen = 0;
        }
    });
    player::presentation::update_trails(&mut h);
    assert_eq!(
        h.events,
        [
            Event::Trail(17, false, true, 0, 0),
            Event::Trail(16, false, true, 77, 0)
        ]
    );
    assert_eq!(h.world.player.projectiles.trails[16].left.remaining, 1);
}

fn terrain_reaction_owner() {
    for header in [0x8000, 0xb974, 0x8001] {
        let mut h = prepared();
        for cell in h.world.player.room.terrain.cells_mut() {
            cell.level = 0;
            cell.behavior = 0;
        }
        h.world.player.room.width = 4;
        h.world.player.room.terrain.cells_mut()[6].level = 0xc000;
        h.world.player.room.terrain.cells_mut()[14].level = 0x8000;
        h.world.player.projectiles.selected = 0;
        h.world.player.projectiles.count = 2;
        let shot = &mut h.world.player.projectiles.shots[0];
        shot.x = 32;
        shot.x_fraction = 0xf000;
        shot.y = 32;
        shot.x_radius = 2;
        shot.y_radius = 9;
        shot.x_speed = 0x180;
        shot.kind = 0x8000;
        shot.direction = 2;
        shot.damage = 20;
        let other = &mut h.world.player.projectiles.shots[1];
        other.x = 16;
        other.y = 16;
        other.x_speed = 0x100;
        other.kind = 0x8100;
        other.damage = 30;
        h.word(0x94, 0x9ea6, header);
        h.on_player_mechanism = Some(|h, header| {
            assert_eq!(
                (
                    h.world.player.projectiles.shots[0].x,
                    h.world.player.projectiles.shots[0].x_fraction
                ),
                (34, 0x7000),
                "terrain.callback.movement_published"
            );
            assert_eq!(h.world.player.room.collision_cursor, 6);
            h.world.player.room.width = 8;
            h.world.player.projectiles.selected = 2;
            if header == 0x8001 {
                panic!("terrain.callback.intentional_failure");
            }
            if header == 0x8000 {
                player::call_motion(h, 0xafdd, 2);
                assert_eq!(
                    h.world.player.projectiles.shots[1].x, 17,
                    "terrain.callback.actual_reentry"
                );
                assert_eq!(h.world.player.room.collision_cursor, 9);
            }
        });
        let mut expected = h.world.clone();
        expected.player.room.width = 8;
        expected.player.room.collision_cursor = if header == 0x8000 { 14 } else { 6 };
        expected.player.projectiles.selected = 2;
        expected.player.projectiles.shots[0].x = 34;
        expected.player.projectiles.shots[0].x_fraction = 0x7000;
        if header == 0x8000 {
            expected.player.projectiles.shots[1].x = 17;
        }
        let result = catch_unwind(AssertUnwindSafe(|| player::call_motion(&mut h, 0xaf60, 0)));
        if header == 0x8001 {
            let error = result.expect_err("terrain callback must fail");
            let reason = error
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| error.downcast_ref::<String>().map(String::as_str));
            assert_eq!(reason, Some("terrain.callback.intentional_failure"));
            assert_eq!(h.world, expected, "terrain.callback.failure_prefix");
            assert_eq!(h.events, [Event::PlayerMechanism(header)]);
        } else {
            assert!(result.is_ok(), "terrain callback success must return");
            let shot = &mut expected.player.projectiles.shots[0];
            shot.x = 36;
            shot.kind = 0x8700;
            shot.cursor = 0x9100;
            shot.remaining = 1;
            shot.damage = 8;
            shot.motion = 0xb169;
            assert_eq!(
                h.world, expected,
                "terrain.callback.live_width_captured_slot.{header}"
            );
            assert_eq!(
                h.events,
                [Event::PlayerMechanism(header), Event::PlayerSound(2, 6, 12)]
            );
        }
    }
}

pub(super) fn run() {
    terrain_reaction_owner();
    firing_and_replay();
    first_frame_boundary();
    reset_palette_composition();
    unsupported_does_not_succeed();
    animation_capture_and_unknown();
    emission_record_recapture();
    emission_caller_reload();
    trail_frame_recapture_and_unknown();
    trail_draw_reentry();
}

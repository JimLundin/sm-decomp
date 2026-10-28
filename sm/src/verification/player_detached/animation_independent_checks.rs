//! Preserved independent animation assertions on the actual shared visual Host.
#![forbid(unsafe_code)]
use super::game::visual_state::AnimationSlot;
use super::{Composed, Event, RomImage, Upload, animated, colors};
use std::panic::{AssertUnwindSafe, catch_unwind};

// composition() is already preserved in checks.rs. Upload callback reuse,
// entry-only enable checks, acknowledgement and retained callback state are
// shared with visual_checks; keep one fixture for those assertions.
fn progression_and_operands() {
    let mut host = Composed::new();
    host.world.player.gameplay_flags.bosses_by_area[0] = 2;
    host.world.player.flag_compatibility.bitmask = 0xab55;
    host.byte(0x87, 0x8500, 2);
    host.word(0x87, 0x8501, 0x9123);
    assert_eq!(animated::command(&mut host, 8880416, 0, 0x8500), 0x9123);
    assert_eq!(host.world.player.flag_compatibility.bitmask, 0xab02);
    host.world.player.flag_compatibility.bitmask = 0xbeef;
    host.byte(0x87, 0x8510, 2);
    host.byte(0x87, 0x8511, 0);
    host.word(0x87, 0x8512, 0x9345);
    assert_eq!(animated::command(&mut host, 8880899, 0, 0x8510), 0x9345);
    assert_eq!(host.world.player.flag_compatibility.bitmask, 0xbeef); // explicit area has no scratch write
    host.world.player.gameplay_flags.events[1] = 2;
    host.word(0x87, 0x8520, 9);
    host.word(0x87, 0x8522, 0x9567);
    assert_eq!(animated::command(&mut host, 8880447, 0, 0x8520), 0x9567);
    assert_eq!(host.world.player.flag_compatibility.bitmask, 2); // full event mask replaces high byte
    host.word(0x87, 0x8530, 64);
    animated::command(&mut host, 8880464, 0, 0x8530);
    assert_eq!(host.world.player.gameplay_flags.bosses_by_area[0], 3); // event byte eight aliases area-zero flags
    host.word(0x87, 0x8600, 0xab09);
    host.word(0x8d, 0x8600, 0xab09);
    assert_eq!(animated::command(&mut host, 8880380, 0, 0x8600), 0x8601);
    assert_eq!(colors::command(&mut host, 9291370, 0, 0x8600), (0, 0x8601));
    assert_eq!(host.events, [Event::Sound(1, 9), Event::Sound(1, 0xab09)]);
    host.world
        .player
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[1]
        .counter = 0xaabb;
    host.byte(0x87, 0x8602, 0x34);
    animated::command(&mut host, 8880355, 3, 0x8602);
    assert_eq!(
        host.world.player.presentation.visual().animated_tiles.slots[1].counter,
        0x34bb
    );
    host.animation_invalid_result = Some(0x2345);
    assert_eq!(animated::command(&mut host, 0, 0, 0), 0x2345);
}
fn failures_and_uploads() {
    let mut host = Composed::new();
    host.image = RomImage::zeroed(0x39000);
    host.world
        .player
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[0]
        .counter = 2;
    assert!(
        catch_unwind(AssertUnwindSafe(|| animated::command(
            &mut host, 8880340, 0, 0x9000
        )))
        .is_err()
    );
    assert_eq!(
        host.world.player.presentation.visual().animated_tiles.slots[0].counter,
        1
    );
    assert_eq!(animated::command(&mut host, 8880340, 0, 0x9000), 0x9002);
    host.image = RomImage::zeroed(0x38004);
    host.word(0x87, 0x8000, 0x9000);
    host.word(0x87, 0x8002, 0x40);
    host.world
        .player
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[5] = AnimationSlot {
        definition: 0,
        counter: 123,
        program: 0xaaaa,
        source: 0x9999,
        size: 3,
        vram: 0xbeef,
        delay: 77,
    };
    assert!(catch_unwind(AssertUnwindSafe(|| animated::spawn(&mut host, 0x8000))).is_err());
    assert_eq!(
        host.world.player.presentation.visual().animated_tiles.slots[5],
        AnimationSlot {
            definition: 0x8000,
            counter: 0,
            program: 0x9000,
            source: 0,
            size: 0x40,
            vram: 0xbeef,
            delay: 77,
        }
    );
    animated::set_enabled(&mut host.world.player, true);
    host.world
        .player
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[5]
        .source = 0xa000;
    host.world
        .player
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[5]
        .size = 0;
    host.world
        .player
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[5]
        .vram = 0x6000;
    host.fail_upload = true;
    assert!(catch_unwind(AssertUnwindSafe(|| animated::upload_pending(&mut host))).is_err());
    assert_eq!(
        host.world.player.presentation.visual().animated_tiles.slots[5].source,
        0xa000
    );
    assert_eq!(
        host.events.pop(),
        Some(Event::Upload(Upload {
            source: 0xa000,
            size: 0,
            vram: 0x6000
        }))
    );
}

pub(super) fn run() {
    progression_and_operands();
    failures_and_uploads();
}

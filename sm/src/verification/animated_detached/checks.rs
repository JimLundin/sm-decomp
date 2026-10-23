//! Independent composed behavior, effect ordering, persistence and failure checks.
#![forbid(unsafe_code)]
use super::{Composed, Event, RomImage, Upload, World, animated, animation, colors};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn composition() {
    let mut host = Composed::new();
    for (cursor, value) in [
        (0x8000, 0x9000),
        (0x8002, 0x40),
        (0x8004, 0x6000),
        (0x9000, 8881010u32 as u16),
        (0x9002, 0x8000),
        (0x9004, 8880987u32 as u16),
        (0x9006, 0),
        (0x9008, 8881023u32 as u16),
        (0x900a, 0),
        (0x900c, 8880464u32 as u16),
        (0x900e, 9),
        (0x9010, 8880435u32 as u16),
        (0x9013, 8880368u32 as u16),
        (0x9016, 8880380u32 as u16),
        (0x9019, 2),
        (0x901b, 0xa000),
        (0x901d, 8880311u32 as u16),
        (0x901f, 0x9019),
    ] {
        host.word(0x87, cursor, value);
    }
    host.byte(0x87, 0x9012, 2);
    host.byte(0x87, 0x9015, 7);
    host.byte(0x87, 0x9018, 9);
    for (cursor, value) in [
        (0x8000, 9291397u32 as u16),
        (0x8002, 0x9000),
        (0x9000, 1),
        (0x9002, 0x1234),
        (0x9004, 9291157u32 as u16),
        (0x9006, 9291294u32 as u16),
        (0x9008, 0x9000),
    ] {
        host.word(0x8d, cursor, value);
    }
    host.world.palette.colors.fill(0x7777);
    host.world.animation.set_enabled(true);
    host.world.palette.set_enabled(true);
    animated::spawn(&mut host, 0x8000);
    host.tick();
    let mut expected = vec![
        Event::PaletteSpawn(0x8000),
        Event::Color(0, 0),
        Event::Color(1, 0),
        Event::Color(2, 0),
    ];
    expected.extend(
        [
            0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
        ]
        .into_iter()
        .enumerate()
        .map(|(i, v)| Event::Target(i, v)),
    );
    expected.extend([
        Event::Music(7),
        Event::Sound(1, 9),
        Event::Color(0, 0x1234),
        Event::Upload(Upload {
            source: 0xa000,
            size: 0x40,
            vram: 0x6000,
        }),
    ]);
    assert_eq!(host.events, expected);
    assert_eq!(host.world.palette.effects.slots[7].id, 0x8000);
    assert_eq!(host.world.palette.effects.slots[7].program, 0x9006);
    assert_eq!(&host.world.palette.colors[..4], &[0x1234, 0, 0, 0x7777]);
    assert_eq!(host.world.progression.events[1], 2);
    assert_eq!(host.world.progression.bosses[0], 2);
    assert_eq!(host.world.progression.scratch, 2);
    assert!(host.world.palette.inputs.miniboss_dead);
    assert_eq!(host.world.animation.slots[5].source, 0);
    let snapshot = host.world.clone();
    host.events.clear();
    host.tick();
    host.tick();
    let completed = host.world.clone();
    let outputs = host.events.clone();
    host.world = World::default();
    assert_eq!(host.world, World::default());
    // This snapshot contains both cores, their shared colors and all world inputs.
    host.world = snapshot;
    host.events.clear();
    host.tick();
    host.tick();
    assert_eq!(host.world, completed);
    assert_eq!(host.events, outputs);
}
fn progression_and_operands() {
    let mut host = Composed::new();
    host.world.progression.bosses[0] = 2;
    host.world.progression.scratch = 0xab55;
    host.byte(0x87, 0x8500, 2);
    host.word(0x87, 0x8501, 0x9123);
    assert_eq!(animated::command(&mut host, 8880416, 0, 0x8500), 0x9123);
    assert_eq!(host.world.progression.scratch, 0xab02);
    host.world.progression.scratch = 0xbeef;
    host.byte(0x87, 0x8510, 2);
    host.byte(0x87, 0x8511, 0);
    host.word(0x87, 0x8512, 0x9345);
    assert_eq!(animated::command(&mut host, 8880899, 0, 0x8510), 0x9345);
    assert_eq!(host.world.progression.scratch, 0xbeef); // explicit area has no scratch write
    host.world.progression.events[1] = 2;
    host.word(0x87, 0x8520, 9);
    host.word(0x87, 0x8522, 0x9567);
    assert_eq!(animated::command(&mut host, 8880447, 0, 0x8520), 0x9567);
    assert_eq!(host.world.progression.scratch, 2); // full event mask replaces high byte
    host.word(0x87, 0x8530, 64);
    animated::command(&mut host, 8880464, 0, 0x8530);
    assert_eq!(host.world.progression.bosses[0], 3); // event byte eight aliases area-zero flags
    host.word(0x87, 0x8600, 0xab09);
    host.word(0x8d, 0x8600, 0xab09);
    assert_eq!(animated::command(&mut host, 8880380, 0, 0x8600), 0x8601);
    assert_eq!(colors::command(&mut host, 9291370, 0, 0x8600), (0, 0x8601));
    assert_eq!(host.events, [Event::Sound(1, 9), Event::Sound(1, 0xab09)]);
    host.world.animation.slots[1].counter = 0xaabb;
    host.byte(0x87, 0x8602, 0x34);
    animated::command(&mut host, 8880355, 3, 0x8602);
    assert_eq!(host.world.animation.slots[1].counter, 0x34bb);
    assert_eq!(animated::command(&mut host, 0, 0, 0), 0x2345);
}
fn failures_and_uploads() {
    let mut host = Composed::new();
    host.image = RomImage::zeroed(0x39000);
    host.world.animation.slots[0].counter = 2;
    assert!(
        catch_unwind(AssertUnwindSafe(|| animated::command(
            &mut host, 8880340, 0, 0x9000
        )))
        .is_err()
    );
    assert_eq!(host.world.animation.slots[0].counter, 1);
    assert_eq!(animated::command(&mut host, 8880340, 0, 0x9000), 0x9002);
    host.image = RomImage::zeroed(0x38004);
    host.word(0x87, 0x8000, 0x9000);
    host.word(0x87, 0x8002, 0x40);
    host.world.animation.slots[5] = animation::state::Slot {
        id: 0,
        counter: 123,
        program: 0xaaaa,
        source: 0x9999,
        size: 3,
        vram: 0xbeef,
        delay: 77,
    };
    assert!(catch_unwind(AssertUnwindSafe(|| animated::spawn(&mut host, 0x8000))).is_err());
    assert_eq!(
        host.world.animation.slots[5],
        animation::state::Slot {
            id: 0x8000,
            counter: 0,
            program: 0x9000,
            source: 0,
            size: 0x40,
            vram: 0xbeef,
            delay: 77,
        }
    );
    host.world.animation.set_enabled(true);
    host.world.animation.slots[5].source = 0xa000;
    host.world.animation.slots[5].size = 0;
    host.world.animation.slots[5].vram = 0x6000;
    host.fail_upload = true;
    assert!(catch_unwind(AssertUnwindSafe(|| animated::upload_pending(&mut host))).is_err());
    assert_eq!(host.world.animation.slots[5].source, 0xa000);
    assert_eq!(
        host.events.pop(),
        Some(Event::Upload(Upload {
            source: 0xa000,
            size: 0,
            vram: 0x6000
        }))
    );
    host.fail_upload = false;
    host.on_upload = Some(|world| {
        world.animation.flags = 0;
        world.animation.selected = 0;
        world.animation.slots[5].source = 0xc000;
        world.animation.slots[4] = animation::state::Slot {
            id: 0x8000,
            source: 0xb000,
            size: 0x20,
            vram: 0x6100,
            ..Default::default()
        };
    });
    animated::upload_pending(&mut host);
    assert_eq!(
        host.events,
        [
            Event::Upload(Upload {
                source: 0xa000,
                size: 0,
                vram: 0x6000
            }),
            Event::Upload(Upload {
                source: 0xb000,
                size: 0x20,
                vram: 0x6100
            }),
        ]
    );
    assert_eq!(host.world.animation.slots[5].source, 0); // acknowledge captured slot despite changed selector
    assert_eq!(host.world.animation.slots[4].source, 0); // next slot reread despite changed flags
}
pub(super) fn run() {
    composition();
    progression_and_operands();
    failures_and_uploads();
}

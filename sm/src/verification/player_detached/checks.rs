//! Independent composed behavior, effect ordering, persistence and failure checks.
#![forbid(unsafe_code)]
use super::{Composed, Event, Upload, World, animated};

fn initial_music(host: &mut Composed) {
    assert_eq!(&host.world.player.visual.colors[..3], &[0, 0, 0]);
    assert_eq!(
        &host.world.player.visual.targets[..8],
        &[
            0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083
        ]
    );
}
pub(super) fn composition() {
    let mut host = Composed::new();
    host.on_visual_music = Some(initial_music);
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
    host.world.player.visual.colors.fill(0x7777);
    animated::set_enabled(&mut host.world.player, true);
    super::colors::set_enabled(&mut host.world.player, true);
    animated::spawn(&mut host, 0x8000);
    host.tick();
    let mut expected = vec![Event::PaletteSpawn(0x8000)];
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
    assert_eq!(
        &host.world.player.visual.targets[..8],
        &[
            0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083
        ]
    );
    assert_eq!(
        host.world.player.visual.palette_effects.slots[7].definition,
        0x8000
    );
    assert_eq!(
        host.world.player.visual.palette_effects.slots[7].program,
        0x9006
    );
    assert_eq!(
        &host.world.player.visual.colors[..4],
        &[0x1234, 0, 0, 0x7777]
    );
    assert_eq!(host.world.progression.events[1], 2);
    assert_eq!(host.world.progression.bosses[0], 2);
    assert_eq!(host.world.progression.scratch, 2);
    assert!(super::game::visual_host::Aliases::miniboss_defeated(&host));
    assert_eq!(host.world.player.visual.animated_tiles.slots[5].source, 0);
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

pub(super) fn run() {
    composition();
    super::player_checks::run();
}

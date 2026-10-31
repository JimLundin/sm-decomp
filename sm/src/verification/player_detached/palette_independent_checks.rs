//! Remaining former palette_detached checks on the actual shared Composed host.
//! Shared snapshot/reset/replay is already covered by checks::composition;
//! palette-to-room aliases and heat callbacks are covered by visual_checks.
#![forbid(unsafe_code)]
use super::{Composed, Event, RomImage, World, colors};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn overlapping_sound_operand_and_clear() {
    let mut h = Composed::new();
    for (cursor, value) in [
        (0x8000, 9291397u32 as u16),
        (0x8002, 0x9000),
        (0x9000, 2),
        (0x9002, 0x1234),
        (0x9004, 9291157u32 as u16),
        (0x9006, 9291358u32 as u16),
        (0x9009, 9291370u32 as u16),
        (0x900b, 9),
        (0x900c, 1),
        (0x900e, 0x2345),
        (0x9010, 9291157u32 as u16),
    ] {
        h.word(0x8d, cursor, value);
    }
    h.byte(0x8d, 0x9008, 7);
    colors::spawn(&mut h, 0x8000);
    assert_eq!(
        h.world.player.presentation.visual().palette_effects.slots[7].definition,
        0x8000
    );
    colors::set_enabled(&mut h.world.player, true);
    colors::advance(&mut h);
    assert_eq!(h.events, [Event::Color(0, 0x1234)]);
    assert_eq!(
        h.world.player.presentation.visual().palette_effects.slots[7].program,
        0x9006
    );
    h.events.clear();
    colors::advance(&mut h);
    assert!(
        h.events.is_empty(),
        "duration2 must defer the next script for one tick"
    );
    colors::advance(&mut h);
    assert_eq!(
        h.events,
        [
            Event::Music(7),
            Event::Sound(1, 0x0109),
            Event::Color(0, 0x2345)
        ],
        "sound reads a native word while its high byte starts the next duration"
    );
    let mut expected = h.world.clone();
    for slot in &mut expected
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .slots
    {
        slot.definition = 0;
    }
    colors::clear(&mut h.world.player);
    assert_eq!(
        h.world, expected,
        "clear must preserve every non-definition field"
    );
}

fn lazy_operands_and_failure_prefixes() {
    let mut h = Composed::new();
    h.image = RomImage::zeroed(0x69000);
    h.world
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .slots[0]
        .counter = 2;
    assert!(
        catch_unwind(AssertUnwindSafe(|| colors::command(
            &mut h, 0xc639, 0, 0x9000
        )))
        .is_err()
    );
    let mut expected = World::default();
    expected
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .slots[0]
        .counter = 1;
    assert_eq!(
        h.world, expected,
        "missing jump operand retains countdown prefix"
    );
    assert!(h.events.is_empty());
    assert_eq!(colors::command(&mut h, 0xc639, 0, 0x9000), (0, 0x9002));
    expected
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .slots[0]
        .counter = 0;
    assert_eq!(
        h.world, expected,
        "completed countdown never reads missing operand"
    );
    colors::set_enabled(&mut h.world.player, true);
    assert!(
        catch_unwind(AssertUnwindSafe(|| colors::command(
            &mut h, 0xc65e, 0, 0x9000
        )))
        .is_err()
    );
    expected
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .flags = 0x8000;
    assert_eq!(h.world, expected);
    assert!(
        h.events.is_empty(),
        "missing music operand delivers no event"
    );

    h.world
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .heat = 1;
    h.world.player.player.periodic_damage_fraction = u16::from_le_bytes(0xc000u16.to_ne_bytes());
    h.world.player.player.periodic_damage = u16::from_le_bytes(1u16.to_ne_bytes());
    assert!(catch_unwind(AssertUnwindSafe(|| colors::pre(&mut h, 9298809, 0))).is_err());
    expected
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .heat = 1;
    expected
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .previous_heat = 1;
    expected
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .slots[0]
        .delay = 1;
    expected.player.player.periodic_damage = u16::from_le_bytes(2u16.to_ne_bytes());
    assert_eq!(
        h.world, expected,
        "missing heat table preserves damage carry, previous_heat and delay"
    );
    assert!(h.events.is_empty());
}

fn replace_after_sound(h: &mut Composed) {
    assert_eq!(h.events.last(), Some(&Event::Sound(1, 0x0109)));
    // Mutate the captured allocation before replacing the current source.
    h.word(0x8d, 0x9100, 0x5678);
    h.image = RomImage::zeroed(0x70000);
    h.word(0x8d, 0x9003, 1);
    h.word(0x8d, 0x9005, 0x3456);
    h.word(0x8d, 0x9007, 9291157u32 as u16);
}
fn recapture_after_sound() {
    use super::game::visual_host::{Content, Record, Source};
    let mut h = Composed::new();
    h.word(0x8d, 0x9000, 9291370u32 as u16);
    h.word(0x8d, 0x9002, 0x0109);
    h.word(0x8d, 0x9003, 1);
    h.word(0x8d, 0x9005, 0x1234);
    h.word(0x8d, 0x9007, 9291157u32 as u16);
    h.word(0x8d, 0x9100, 0x4567);
    let captured = Content::record(&mut h, Source::Loaded, 0x8d, 0x9100);
    let slot = &mut h
        .world
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .slots[0];
    slot.definition = 1;
    slot.pre_action = 0xc526;
    slot.program = 0x9000;
    slot.delay = 1;
    h.on_visual_sound = Some(replace_after_sound);
    colors::process(&mut h, 0);
    assert_eq!(
        h.events,
        [Event::Sound(1, 0x0109), Event::Color(0, 0x3456)],
        "the interpreter recaptures subsequent records after the sound callback"
    );
    assert_eq!(
        captured.native_word(0),
        0x5678,
        "captured source remains live and retained after replacement"
    );
    assert_eq!(
        Content::record(&mut h, Source::Loaded, 0x8d, 0x9100).native_word(0),
        0
    );
}

pub(super) fn run() {
    overlapping_sound_operand_and_clear();
    lazy_operands_and_failure_prefixes();
    recapture_after_sound();
}

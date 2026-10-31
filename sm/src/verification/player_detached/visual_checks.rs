//! Real visual producers and consumers operate on the same gameplay Session.
use super::game::{
    samus_palette as samus,
    visual_state::{PaletteBuffer, Upload},
};
use super::{Composed, Event, World, animated, colors, room};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn words(h: &mut Composed, bank: u8, values: &[(u16, u16)]) {
    for &(cursor, value) in values {
        h.word(bank, cursor, value);
    }
}
fn palette_script(h: &mut Composed, definition: u16, program: u16, offset: u16, value: u16) {
    words(
        h,
        0x8d,
        &[
            (definition, 9291397u32 as u16),
            (definition + 2, program),
            (program, 9291349u32 as u16),
            (program + 2, offset),
            (program + 4, 1),
            (program + 6, value),
            (program + 8, 9291157u32 as u16),
        ],
    );
}
fn eye_then_palette() {
    let mut h = Composed::new();
    // Real EyeGlow initializer8829070 writes colors249..252 from maintained table.
    words(
        &mut h,
        0x86,
        &[
            (0xba6a, 8829070u32 as u16),
            (0xba6c, 0x84fb),
            (0xba6e, 0x9000),
            (0xba70, 0x0202),
            (0xba72, 0),
        ],
    );
    words(
        &mut h,
        0x87,
        &[
            (0x8100, 0xa100),
            (0x8102, 0x40),
            (0x8104, 0x6000),
            (0xa100, 8880928u32 as u16),
            (0xa102, 0),
            (0xa104, 8881010u32 as u16),
            (0xa106, 0x8200),
            (0xa108, 1),
            (0xa10a, 0xa200),
        ],
    );
    palette_script(&mut h, 0x8200, 0x9400, 500, 0x4321);
    animated::set_enabled(&mut h.world.player, true);
    colors::set_enabled(&mut h.world.player, true);
    animated::spawn(&mut h, 0x8100);
    animated::advance(&mut h);
    assert_eq!(
        &h.world.player.presentation.visual().colors[249..253],
        &[0x6bff, 0x033b, 0x0216, 0x0173],
        "visual.composition.eye_shared_colors"
    );
    let eye = &h.world.player.enemy_projectiles.slots[17];
    assert_eq!(
        (eye.definition, eye.x.whole, eye.y.whole),
        (0xba6a, 0x84, 0x90)
    );
    assert_eq!(
        h.world.player.presentation.visual().palette_effects.slots[7].definition,
        0x8200
    );
    colors::advance(&mut h);
    assert_eq!(
        &h.world.player.presentation.visual().colors[249..253],
        &[0x6bff, 0x4321, 0x0216, 0x0173],
        "visual.composition.animation_palette_after_eye"
    );
}
fn palette_timer_alias() {
    let mut h = Composed::new();
    palette_script(&mut h, 0x8100, 0x9500, 0x1e1c, 1);
    h.world.player.mechanisms.flags = 0x8000;
    let room_slot = &mut h.world.player.mechanisms.slots[0];
    room_slot.header = 1;
    room_slot.instruction_delay = 9;
    room_slot.pre_action = 8684649u32 as u16;
    room_slot.program = 0x9800;
    words(
        &mut h,
        0x84,
        &[
            (0x9800, 8685249u32 as u16),
            (0x9802, 0x1234),
            (0x9804, 8685236u32 as u16),
        ],
    );
    colors::set_enabled(&mut h.world.player, true);
    colors::spawn(&mut h, 0x8100);
    colors::advance(&mut h);
    assert_eq!(
        h.world.player.mechanisms.slots[0].instruction_delay, 1,
        "visual.composition.palette_timer_alias"
    );
    assert!(
        h.events.iter().all(|e| !matches!(e, Event::Color(..))),
        "alias has no canonical color observer"
    );
    let mut pass = room::host::Pass::Instructions;
    assert!(matches!(
        room::advance(&mut h, &mut pass),
        room::host::RunResult::Complete
    ));
    assert_eq!(
        h.world.player.mechanisms.slots[0].pre_action, 0x1234,
        "visual.composition.palette_timer_actual_room_consumer"
    );
}
fn heat_sound(h: &mut Composed) {
    assert_eq!(h.events.last(), Some(&Event::Sound(3, 0x2d)));
    assert_eq!(
        (
            h.world.player.player.periodic_damage_fraction,
            h.world.player.player.periodic_damage
        ),
        (0x3001, 8),
        "visual.composition.heat_fraction_before_sound"
    );
    h.world.player.inventory.equipment.active = 0x20;
    h.world
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .heat = 3;
}
fn heat_callback() {
    let mut h = Composed::new();
    h.world.player.player.periodic_damage_fraction = 0xf001;
    h.world.player.player.periodic_damage = 7;
    h.world.player.inventory.health.current = 99;
    h.world.player_inputs.frame = 0;
    h.world
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .heat = 1;
    h.world
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .previous_heat = 0;
    words(
        &mut h,
        0x8d,
        &[
            (0xe3e6, 0x9700),
            (0xe406, 0x9800),
            (0xe426, 0x9900),
            (0x9700, 1),
            (0x9702, 0x4321),
            (0x9704, 9291157u32 as u16),
        ],
    );
    h.on_visual_sound = Some(heat_sound);
    colors::pre(&mut h, 9298809, 0);
    assert_eq!(
        h.world.player.presentation.visual().palette_effects.slots[0].program,
        0x9700,
        "visual.composition.heat_equipment_reread"
    );
    assert_eq!(
        h.world
            .player
            .presentation
            .visual()
            .palette_effects
            .previous_heat,
        3
    );
    h.world
        .player
        .presentation
        .visual_mut()
        .palette_effects
        .slots[0]
        .pre_action = 0xc526;
    colors::process(&mut h, 0);
    assert_eq!(
        h.world.player.presentation.visual().colors[0],
        0x4321,
        "visual.composition.heat_actual_palette_consumer"
    );
}
fn upload_fixture() -> Composed {
    let mut h = Composed::new();
    animated::set_enabled(&mut h.world.player, true);
    let slot = &mut h
        .world
        .player
        .presentation
        .visual_mut()
        .animated_tiles
        .slots[5];
    slot.definition = 1;
    slot.source = 0xa000;
    slot.size = 0x40;
    slot.vram = 0x6000;
    h
}
fn upload_reuse(world: &mut World) {
    let tiles = &mut world.player.presentation.visual_mut().animated_tiles;
    tiles.selected = 0;
    tiles.flags = 0;
    tiles.slots[5].source = 0xb000;
    // Inactive slot0 retains its source; later slot4 is still submitted after disable.
    tiles.slots[0].source = 0xc000;
    tiles.slots[4].definition = 1;
    tiles.slots[4].source = 0xd000;
    tiles.slots[4].size = 0;
    tiles.slots[4].vram = 0x7000;
}
fn upload_ack() {
    let mut h = upload_fixture();
    h.on_upload = Some(upload_reuse);
    animated::upload_pending(&mut h);
    assert_eq!(
        (
            h.world.player.presentation.visual().animated_tiles.slots[5].source,
            h.world.player.presentation.visual().animated_tiles.slots[0].source
        ),
        (0, 0xc000),
        "visual.composition.upload_captured_ack"
    );
    assert_eq!(
        h.events,
        [
            Event::Upload(Upload {
                source: 0xa000,
                size: 0x40,
                vram: 0x6000
            }),
            Event::Upload(Upload {
                source: 0xd000,
                size: 0,
                vram: 0x7000
            })
        ],
        "visual.composition.upload_entry_enable_only"
    );
    assert_eq!(
        h.world.player.presentation.visual().animated_tiles.slots[4].source,
        0
    );
}
fn upload_partial(world: &mut World) {
    world.player.inventory.ammunition[0].current = 3;
}
fn upload_failure() {
    let mut h = upload_fixture();
    h.fail_upload = true;
    h.on_upload = Some(upload_partial);
    let error = catch_unwind(AssertUnwindSafe(|| animated::upload_pending(&mut h)))
        .expect_err("upload must fail");
    assert!(
        error.is::<super::visual_host::UploadFailed>(),
        "visual.composition.upload_failure_reached"
    );
    assert_eq!(
        h.world.player.presentation.visual().animated_tiles.slots[5].source,
        0xa000,
        "visual.composition.upload_failure_source"
    );
    assert_eq!(
        h.world.player.inventory.ammunition[0].current, 3,
        "visual.composition.upload_failure_effect_retained"
    );
}
fn fill_suit(h: &mut Composed, address: u16, first: u16) {
    for i in 0..16 {
        h.word(0x9b, address + i * 2, first + i);
    }
}
fn samus_shared_paths() {
    let mut h = Composed::new();
    h.world.player.player.grapple = 0xc4f0;
    // Actual inventory choice feeds both suit index and real current/target copies.
    h.world.player.inventory.equipment.active = 0x20;
    samus::update_suit_index(&mut h.world.player);
    fill_suit(&mut h, 0x9800, 0x100);
    samus::load_suit(&mut h, PaletteBuffer::Current);
    samus::load_suit(&mut h, PaletteBuffer::Target);
    assert_eq!(h.world.player.presentation.palette().suit_index, 4);
    assert_eq!(
        &h.world.player.presentation.visual().colors[192..208],
        &(0x100u16..0x110).collect::<Vec<_>>()[..],
        "visual.composition.suit_shared_inventory"
    );
    assert_eq!(
        &h.world.player.presentation.visual().targets[192..208],
        &h.world.player.presentation.visual().colors[192..208]
    );
    // Real player charge decisions call actual Samus output and native suit copy.
    h.world.player.projectiles.controls.charge = 60;
    h.world.player.projectiles.controls.charge_palette = 2;
    words(&mut h, 0x91, &[(0xd7d9, 0x9000), (0x9002, 0x9900)]);
    fill_suit(&mut h, 0x9900, 0x200);
    assert_eq!(samus::charge_feedback(&mut h), 0);
    assert_eq!(
        h.world.player.presentation.visual().colors[192],
        0x200,
        "visual.composition.charge_actual_output"
    );
    assert_eq!(h.world.player.projectiles.controls.charge_palette, 4);
    // Uncharged feedback invokes actual visor on the same player palette timer.
    h.world.player.projectiles.controls.charge = 0;
    h.world.player_inputs.blending_config = 40;
    h.world.player.presentation.palette_mut().visor_timer = 1;
    h.word(0x9b, 0xa3c0, 0x3456);
    samus::charge_feedback(&mut h);
    assert_eq!(
        (
            h.world.player.presentation.visual().colors[196],
            h.world.player.presentation.palette().visor_timer
        ),
        (0x3456, 0x0205),
        "visual.composition.charge_to_visor"
    );
    assert_eq!(h.world.player.projectiles.controls.charge_palette, 0);
}
fn samus_carry() {
    use super::game::visual_host::Source;
    let mut h = Composed::new();
    h.world.player.player.grapple = 0xc4f0;
    h.world.player.player.shine_tick = 2;
    h.world.player.presentation.palette_mut().hurt_flash = 0x803b;
    h.word(0x91, 0xd727, 0x9300);
    fill_suit(&mut h, 0x9300, 0x0600);
    fill_suit(&mut h, 0xa380, 0x0700);
    fill_suit(&mut h, 0x9400, 0x0800);
    samus::advance(&mut h);
    // First misc returns unsigned carry Set despite negative subtraction branch.
    // Second misc independently loads9400, so final color alone cannot show whether
    // an incorrect intervening default9300 copy ran. Observe real source captures.
    let default_copies = h
        .visual_records
        .iter()
        .filter(|&&(source, bank, address)| {
            source == Source::Native && bank == 0x9b && address == 0x9300
        })
        .count();
    assert_eq!(
        (
            h.world.player.presentation.palette().hurt_flash,
            default_copies
        ),
        (0x803d, 0),
        "visual.composition.misc_unsigned_carry"
    );
    let mut h = Composed::new();
    h.world.player.player.grapple = 0xc4f0;
    h.world.player.player.shine_tick = 2;
    h.world.player.presentation.palette_mut().hurt_flash = 8;
    h.word(0x91, 0xd727, 0x9300);
    fill_suit(&mut h, 0x9300, 0x0600);
    samus::advance(&mut h);
    assert_eq!(
        (
            h.world.player.presentation.palette().hurt_flash,
            h.world.player.presentation.visual().colors[192]
        ),
        (10, 0x0600),
        "visual.composition.selector2_double_misc"
    );
    use super::game::visual_state::PaletteCarry;
    for (grapple, charge, expected, label) in [
        (
            0xc4f0,
            0x8010,
            PaletteCarry::Set,
            "visual.composition.misc_charge_unsigned_carry",
        ),
        (
            0x4855,
            0,
            PaletteCarry::Clear,
            "visual.composition.misc_grapple_unsigned_carry",
        ),
    ] {
        let mut h = Composed::new();
        h.world.player.player.grapple = grapple;
        h.world.player.projectiles.controls.charge = charge;
        h.world.player.presentation.palette_mut().hurt_flash = 39;
        let carry = samus::misc(&mut h);
        assert_eq!(
            (carry, h.world.player.presentation.palette().hurt_flash),
            (expected, 40),
            "{label}"
        );
    }
}
fn charge_fixture() -> Composed {
    let mut h = Composed::new();
    h.world.player.player.grapple = 0xc4f0;
    h.world.player.presentation.palette_mut().suit_index = 5;
    h.world.player.player.contact.contact_attack = 3;
    h.world.player.projectiles.controls.charge = 60;
    h.world.player.projectiles.controls.charge_palette = 2;
    h.world.player.presentation.visual_mut().colors.fill(0x7777);
    h.world
        .player
        .presentation
        .visual_mut()
        .targets
        .fill(0x5555);
    words(
        &mut h,
        0x91,
        &[
            (0xd7d9, 0x9000),
            (0xd803, 0x9020),
            (0x9002, 0x9800),
            (0x9008, 0x9800),
            (0x900a, 0x9800),
            (0x9022, 0x9820),
            (0xd82b, 0x9840),
            (0xd847, 0x9860),
            (0xd72b, 0x9880),
            (0xd9a2, 0x98a0),
        ],
    );
    // Tables and dynamic charge pointers are Loaded; suit bytes are Native.
    h.transition_contract.loaded_image = Some(h.image.clone());
    h.image = super::RomImage::zeroed(0x100000);
    // Wrong Native indirection stays valid, so provenance faults reach assertions.
    words(
        &mut h,
        0x91,
        &[
            (0x9002, 0x9820),
            (0x9008, 0x9820),
            (0x900a, 0x9820),
            (0x9022, 0x9800),
        ],
    );
    for (address, first) in [
        (0x9800, 0x200),
        (0x9820, 0x300),
        (0x9840, 0x400),
        (0x9860, 0x500),
        (0x9880, 0x600),
        (0x98a0, 0x700),
    ] {
        fill_suit(&mut h, address, first);
    }
    h
}
fn charge_glow() {
    // Literal results distinguish odd hyper ticks, the 0x1e mask and restore.
    for (hyper, glow, result, remaining, color) in [
        (0, 1, 1, 0, 0),
        (0, 2, 0, 1, 1023),
        (0, 0xffff, 0, 0xfffe, 1023),
        (1, 1, 0, 0, 0),
        (1, 2, 0, 1, 0x400),
        (1, 30, 0, 29, 0x500),
        (1, 32, 1, 0, 0),
        (1, 34, 0, 33, 0x400),
        (1, 0xffff, 0, 0xfffe, 0),
    ] {
        let mut h = charge_fixture();
        h.world.player.inventory.hyper_beam = hyper;
        h.world.player.projectiles.controls.glow = glow;
        let mut expected = h.world.clone();
        expected.player.projectiles.controls.glow = remaining;
        if color == 1023 {
            expected.player.presentation.visual_mut().colors[193..208].fill(1023);
        } else if color != 0 {
            for i in 0..16 {
                expected.player.presentation.visual_mut().colors[192 + i] = color + i as u16;
            }
        }
        assert_eq!(
            samus::charge_feedback(&mut h),
            result,
            "charge.glow.return.{hyper}.{glow}"
        );
        assert_eq!(h.world, expected, "charge.glow.owner.{hyper}.{glow}");
        assert_eq!(
            h.visual_records.len(),
            usize::from(hyper != 0 && color != 0),
            "charge.glow.lazy_content.{hyper}.{glow}"
        );
    }
}
fn charge_held() {
    use super::game::visual_host::Source;
    for (charge, grapple, contact, phase, next, color) in [
        (59, 0xc4f0, 3, 2, 0, 0),
        (60, 0xc4f0, 3, 8, 10, 0x200),
        (60, 0xc4f0, 3, 10, 0, 0x200),
        (60, 0xc4f0, 4, 2, 4, 0x300),
        (0x803b, 0xc4f0, 3, 2, 4, 0x200),
        (0x803c, 0xc4f0, 3, 2, 0, 0),
        (60, 0xc4f1, 3, 2, 0, 0),
    ] {
        let mut h = charge_fixture();
        h.world.player.projectiles.controls.charge = charge;
        h.world.player.projectiles.controls.charge_palette = phase;
        h.world.player.player.grapple = grapple;
        h.world.player.player.contact.contact_attack = contact;
        let mut expected = h.world.clone();
        expected.player.projectiles.controls.charge_palette = next;
        if color == 0 {
            expected.player.presentation.palette_mut().visor_timer = 1537;
        } else {
            for i in 0..16 {
                expected.player.presentation.visual_mut().colors[192 + i] = color + i as u16;
            }
        }
        assert_eq!(samus::charge_feedback(&mut h), 0);
        assert_eq!(
            h.world, expected,
            "charge.held.owner.{charge}.{contact}.{phase}"
        );
        let records = if color == 0 {
            vec![]
        } else {
            vec![
                (
                    Source::Loaded,
                    0x91,
                    if contact == 4 { 0x9022 } else { 0x9000 + phase },
                ),
                (
                    Source::Native,
                    0x9b,
                    if contact == 4 { 0x9820 } else { 0x9800 },
                ),
            ]
        };
        assert_eq!(h.visual_records, records, "charge.held.provenance");
    }
}
fn charge_live() {
    use super::game::visual_host::Source;
    let mut h = charge_fixture();
    h.world.player.inventory.hyper_beam = 1;
    h.world.player.projectiles.controls.glow = 2;
    h.on_visual_record = Some(|h, source, bank, address| {
        assert_eq!((source, bank, address), (Source::Native, 0x9b, 0x9840));
        assert_eq!(h.world.player.projectiles.controls.glow, 2);
        h.world.player.projectiles.controls.glow = 0;
    });
    let mut expected = h.world.clone();
    expected.player.projectiles.controls.glow = 0xffff;
    for i in 0..16 {
        expected.player.presentation.visual_mut().colors[192 + i] = 0x400 + i as u16;
    }
    assert_eq!(samus::charge_feedback(&mut h), 0);
    assert_eq!(h.world, expected, "charge.live.glow_after_copy");
    let mut h = charge_fixture();
    h.on_visual_record = Some(|h, source, bank, address| {
        if source == Source::Loaded {
            assert_eq!((bank, address), (0x91, 0x9002));
            h.world.player.player.contact.contact_attack = 4;
            h.world.player.presentation.palette_mut().suit_index = 0;
            h.world.player.projectiles.controls.charge_palette = 8;
        } else {
            assert_eq!((bank, address), (0x9b, 0x9800));
            assert_eq!(h.world.player.projectiles.controls.charge_palette, 8);
            h.world.player.projectiles.controls.charge_palette = 10;
        }
    });
    let mut expected = h.world.clone();
    expected.player.player.contact.contact_attack = 4;
    expected.player.presentation.palette_mut().suit_index = 0;
    expected.player.projectiles.controls.charge_palette = 0;
    for i in 0..16 {
        expected.player.presentation.visual_mut().colors[192 + i] = 0x200 + i as u16;
    }
    assert_eq!(samus::charge_feedback(&mut h), 0);
    assert_eq!(h.world, expected, "charge.live.phase_after_copy");
    assert_eq!(
        h.visual_records,
        [
            (Source::Loaded, 0x91, 0x9002),
            (Source::Native, 0x9b, 0x9800)
        ],
        "charge.live.captured_selectors"
    );
}
fn charge_failure() {
    use super::game::visual_host::Source;
    for hyper in [0, 1] {
        let mut h = charge_fixture();
        h.world.player.inventory.hyper_beam = hyper;
        h.world.player.projectiles.controls.glow = if hyper == 0 { 0 } else { 2 };
        let source = if hyper == 0 { 0x9800 } else { 0x9840 };
        // 31 available bytes: the whole 32-byte source must precede any color write.
        h.image = super::RomImage::zeroed(0x1b * 0x8000 + (source & 0x7fff) + 31);
        let before = h.world.clone();
        assert!(
            catch_unwind(AssertUnwindSafe(|| samus::charge_feedback(&mut h))).is_err(),
            "charge short suit source must fail"
        );
        assert_eq!(h.world, before, "charge.failure.owner_prefix.{hyper}");
        let mut records = vec![];
        if hyper == 0 {
            records.push((Source::Loaded, 0x91, 0x9002));
        }
        records.push((Source::Native, 0x9b, source as u16));
        assert_eq!(
            h.visual_records, records,
            "charge.failure.content_prefix.{hyper}"
        );
    }
}
fn charge_caller() {
    use super::game::visual_host::Source;
    let mut h = charge_fixture();
    h.world.player.projectiles.controls.glow = 1;
    h.world.player.player.shine_tick = 3; // Deliberately invalid if dispatch runs.
    h.world.player.presentation.palette_mut().hurt_flash = 8;
    let mut expected = h.world.clone();
    expected.player.projectiles.controls.glow = 0;
    expected.player.presentation.palette_mut().hurt_flash = 9; // Unconditional final misc.
    for i in 0..16 {
        expected.player.presentation.visual_mut().colors[192 + i] = 0x600 + i as u16;
    }
    samus::advance(&mut h);
    assert_eq!(h.world, expected, "charge.caller.restore_skips_dispatch");
    assert_eq!(h.visual_records, [(Source::Native, 0x9b, 0x9880)]);
    let mut h = charge_fixture();
    h.world.player.projectiles.controls.glow = 2;
    h.world.player.player.special_palette_flags = 0x8000;
    h.world.player.player.shine_tick = 3;
    h.world.player.presentation.palette_mut().timer = 3;
    let mut expected = h.world.clone();
    expected.player.presentation.palette_mut().timer = 2;
    for i in 0..16 {
        expected.player.presentation.visual_mut().colors[192 + i] = 0x700 + i as u16;
    }
    samus::advance(&mut h);
    assert_eq!(
        h.world, expected,
        "charge.caller.flag_skips_charge_runs_misc"
    );
    assert_eq!(h.visual_records, [(Source::Native, 0x9b, 0x98a0)]);
}

fn atmosphere_fixture(frame: u16, timer: u16) -> Composed {
    let mut h = Composed::new();
    for kind in 0..9 {
        h.word(0x90, 0x8b93 + kind * 2, 0x9000);
        h.word(0x90, 0x8bef + kind * 2, 3);
    }
    h.transition_contract.loaded_image = Some(h.image.clone());
    h.image = super::RomImage::zeroed(0x100000);
    h.word(0x90, 0x9000, 7);
    h.word(0x90, 0x9002, 8);
    h.word(0x90, 0x9004, 9);
    h.word(0x90, 0x91fe, 10);
    let slot = &mut h.world.player.presentation.atmospheric_mut().slots[0];
    slot.frame_and_kind = frame;
    slot.remaining = timer;
    slot.x = 136;
    slot.y = 100;
    h
}
fn atmosphere_boundaries() {
    use super::game::player_transitions::draw_atmospheric;
    for (frame, timer, next, remaining, draws) in [
        (0x100, 0, 0x100, 0xffff, 0),
        (0x100, 1, 0x101, 7, 1),
        (0x100, 0x8000, 0x100, 0x7fff, 1),
        (0x100, 0x8001, 0x100, 7, 1),
        (0x100, 0xffff, 0x100, 0xfffe, 0),
        (0x102, 1, 0, 9, 0),
        (0x1ff, 1, 0x200, 10, 1),
    ] {
        let mut h = atmosphere_fixture(frame, timer);
        draw_atmospheric(&mut h);
        let slot = &h.world.player.presentation.atmospheric().slots[0];
        assert_eq!(
            (slot.frame_and_kind, slot.remaining, h.events.len()),
            (next, remaining, draws),
            "atmosphere.boundary.{frame}.{timer}"
        );
        if draws != 0 {
            assert_eq!(h.events, [Event::AtmosphericTile(0, 1, next, 136, 100)]);
        }
    }
}
fn atmosphere_live() {
    use super::game::{player_transitions::draw_atmospheric, visual_host::Source};
    let mut h = atmosphere_fixture(0x100, 1);
    h.on_visual_record = Some(|h, source, bank, at| {
        assert_eq!((source, bank, at), (Source::Native, 0x90, 0x9000));
        assert_eq!(
            h.world.player.presentation.atmospheric().slots[0].remaining,
            0
        );
        h.world.player.presentation.atmospheric_mut().slots[0].frame_and_kind = 0x700;
    });
    draw_atmospheric(&mut h);
    assert_eq!(
        h.events,
        [Event::AtmosphericTile(0, 1, 0x701, 136, 100)],
        "atmosphere.live.captured_kind_live_frame"
    );
    let mut h = atmosphere_fixture(0x300, 2);
    h.world.player_inputs.fx_y = 120;
    h.world.player_inputs.camera_x = 4;
    h.world.player_inputs.camera_y = 10;
    draw_atmospheric(&mut h);
    assert_eq!(
        h.events,
        [Event::SamusSprite(399, 132, 110)],
        "atmosphere.live.dive_position"
    );
    assert_eq!(h.world.player.presentation.atmospheric().slots[0].y, 120);
}
fn atmosphere_failure() {
    use super::game::player_transitions::draw_atmospheric;
    for invalid in [1, 0x800] {
        let mut h = atmosphere_fixture(invalid, 2);
        let slot = &mut h.world.player.presentation.atmospheric_mut().slots[3];
        slot.frame_and_kind = 0x400;
        slot.remaining = 2;
        slot.x = 8;
        slot.y = 10;
        assert!(catch_unwind(AssertUnwindSafe(|| draw_atmospheric(&mut h))).is_err());
        assert_eq!(
            h.world.player.presentation.atmospheric().slots[0].remaining,
            1,
            "atmosphere.failure.invalid_timer_prefix"
        );
        assert_eq!(
            h.events,
            [Event::AtmosphericTile(3, 4, 0x400, 7, 9)],
            "atmosphere.failure.previous_slot_prefix"
        );
    }
    let mut h = atmosphere_fixture(0x100, 1);
    // Safe owned record has one byte available. No undefined predecessor is run.
    h.image = super::RomImage::zeroed(0x81001);
    assert!(catch_unwind(AssertUnwindSafe(|| draw_atmospheric(&mut h))).is_err());
    assert_eq!(
        (
            h.world.player.presentation.atmospheric().slots[0].frame_and_kind,
            h.world.player.presentation.atmospheric().slots[0].remaining
        ),
        (0x100, 0),
        "atmosphere.failure.short_timer_prefix"
    );
    assert!(h.events.is_empty());
    let mut h = atmosphere_fixture(0x400, 2);
    h.on_sprite_draw = Some(|h| {
        h.world.player.inventory.health.current = 7;
        panic!("intentional atmospheric render failure");
    });
    assert!(catch_unwind(AssertUnwindSafe(|| draw_atmospheric(&mut h))).is_err());
    let slot = &h.world.player.presentation.atmospheric().slots[0];
    assert_eq!(
        (slot.remaining, slot.x, slot.y),
        (1, 137, 99),
        "atmosphere.failure.motion_before_render"
    );
    assert_eq!(
        h.world.player.inventory.health.current, 7,
        "atmosphere.failure.effect_prefix"
    );
}
#[test]
fn owned_atmospheric_boundaries() {
    atmosphere_boundaries();
}

pub(super) fn run_selected(case: &str) {
    match case {
        "atmosphere_boundaries" => atmosphere_boundaries(),
        "atmosphere_live" => atmosphere_live(),
        "atmosphere_failure" => atmosphere_failure(),
        "eye" => eye_then_palette(),
        "alias" => palette_timer_alias(),
        "heat" => heat_callback(),
        "upload_ack" => upload_ack(),
        "upload_failure" => upload_failure(),
        "samus" => samus_shared_paths(),
        "samus_carry" => samus_carry(),
        "charge_glow" => charge_glow(),
        "charge_held" => charge_held(),
        "charge_live" => charge_live(),
        "charge_failure" => charge_failure(),
        "charge_caller" => charge_caller(),
        _ => panic!("unknown visual composition case"),
    }
}
pub(super) fn run() {
    for case in [
        "atmosphere_boundaries",
        "atmosphere_live",
        "atmosphere_failure",
        "eye",
        "alias",
        "heat",
        "upload_ack",
        "upload_failure",
        "samus",
        "samus_carry",
        "charge_glow",
        "charge_held",
        "charge_live",
        "charge_failure",
        "charge_caller",
    ] {
        run_selected(case);
    }
}

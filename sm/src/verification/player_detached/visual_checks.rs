//! Real visual producers and consumers operate on the same gameplay Session.
use super::game::{
    player_projectiles as player, samus_palette as samus,
    visual_state::{PaletteBuffer, Upload},
};
use super::{Composed, Event, World, animated, colors, room};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn words(h: &Composed, bank: u8, values: &[(u16, u16)]) {
    for &(cursor, value) in values {
        h.word(bank, cursor, value);
    }
}
fn palette_script(h: &Composed, definition: u16, program: u16, offset: u16, value: u16) {
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
        &h,
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
        &h,
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
    palette_script(&h, 0x8200, 0x9400, 500, 0x4321);
    animated::set_enabled(&mut h.world.player, true);
    colors::set_enabled(&mut h.world.player, true);
    animated::spawn(&mut h, 0x8100);
    animated::advance(&mut h);
    assert_eq!(
        &h.world.player.visual.colors[249..253],
        &[0x6bff, 0x033b, 0x0216, 0x0173],
        "visual.composition.eye_shared_colors"
    );
    let eye = &h.world.player.enemy_projectiles.slots[17];
    assert_eq!(
        (eye.definition, eye.x.whole, eye.y.whole),
        (0xba6a, 0x84, 0x90)
    );
    assert_eq!(
        h.world.player.visual.palette_effects.slots[7].definition,
        0x8200
    );
    colors::advance(&mut h);
    assert_eq!(
        &h.world.player.visual.colors[249..253],
        &[0x6bff, 0x4321, 0x0216, 0x0173],
        "visual.composition.animation_palette_after_eye"
    );
}
fn palette_timer_alias() {
    let mut h = Composed::new();
    palette_script(&h, 0x8100, 0x9500, 0x1e1c, 1);
    h.world.player.mechanisms.flags = 0x8000;
    let room_slot = &mut h.world.player.mechanisms.slots[0];
    room_slot.header = 1;
    room_slot.instruction_delay = 9;
    room_slot.pre_action = 8684649u32 as u16;
    room_slot.program = 0x9800;
    words(
        &h,
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
    h.world.player.visual.palette_effects.heat = 3;
}
fn heat_callback() {
    let mut h = Composed::new();
    h.world.player.player.periodic_damage_fraction = 0xf001;
    h.world.player.player.periodic_damage = 7;
    h.world.player.inventory.health.current = 99;
    h.world.player_inputs.frame = 0;
    h.world.player.visual.palette_effects.heat = 1;
    h.world.player.visual.palette_effects.previous_heat = 0;
    words(
        &h,
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
        h.world.player.visual.palette_effects.slots[0].program, 0x9700,
        "visual.composition.heat_equipment_reread"
    );
    assert_eq!(h.world.player.visual.palette_effects.previous_heat, 3);
    h.world.player.visual.palette_effects.slots[0].pre_action = 0xc526;
    colors::process(&mut h, 0);
    assert_eq!(
        h.world.player.visual.colors[0], 0x4321,
        "visual.composition.heat_actual_palette_consumer"
    );
}
fn upload_fixture() -> Composed {
    let mut h = Composed::new();
    animated::set_enabled(&mut h.world.player, true);
    let slot = &mut h.world.player.visual.animated_tiles.slots[5];
    slot.definition = 1;
    slot.source = 0xa000;
    slot.size = 0x40;
    slot.vram = 0x6000;
    h
}
fn upload_reuse(world: &mut World) {
    let tiles = &mut world.player.visual.animated_tiles;
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
            h.world.player.visual.animated_tiles.slots[5].source,
            h.world.player.visual.animated_tiles.slots[0].source
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
    assert_eq!(h.world.player.visual.animated_tiles.slots[4].source, 0);
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
    let reason = error
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| error.downcast_ref::<String>().map(String::as_str));
    assert_eq!(reason, Some("intentional upload failure"));
    assert_eq!(
        h.world.player.visual.animated_tiles.slots[5].source, 0xa000,
        "visual.composition.upload_failure_source"
    );
    assert_eq!(
        h.world.player.inventory.ammunition[0].current, 3,
        "visual.composition.upload_failure_effect_retained"
    );
}
fn fill_suit(h: &Composed, address: u16, first: u16) {
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
    fill_suit(&h, 0x9800, 0x100);
    samus::load_suit(&mut h, PaletteBuffer::Current);
    samus::load_suit(&mut h, PaletteBuffer::Target);
    assert_eq!(h.world.player.player.palette.suit_index, 4);
    assert_eq!(
        &h.world.player.visual.colors[192..208],
        &(0x100u16..0x110).collect::<Vec<_>>()[..],
        "visual.composition.suit_shared_inventory"
    );
    assert_eq!(
        &h.world.player.visual.targets[192..208],
        &h.world.player.visual.colors[192..208]
    );
    // Real player charge decisions call actual Samus output and native suit copy.
    h.world.player.projectiles.controls.charge = 60;
    h.world.player.projectiles.controls.charge_palette = 2;
    words(&h, 0x91, &[(0xd7d9, 0x9000), (0x9002, 0x9900)]);
    fill_suit(&h, 0x9900, 0x200);
    assert_eq!(player::charge_palette(&mut h), 0);
    assert_eq!(
        h.world.player.visual.colors[192], 0x200,
        "visual.composition.charge_actual_output"
    );
    assert_eq!(h.world.player.projectiles.controls.charge_palette, 4);
    // Uncharged feedback invokes actual visor on the same player palette timer.
    h.world.player.projectiles.controls.charge = 0;
    h.world.player_inputs.blending_config = 40;
    h.world.player.player.palette.visor_timer = 1;
    h.word(0x9b, 0xa3c0, 0x3456);
    player::charge_palette(&mut h);
    assert_eq!(
        (
            h.world.player.visual.colors[196],
            h.world.player.player.palette.visor_timer
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
    h.world.player.player.palette.hurt_flash = 0x803b;
    h.word(0x91, 0xd727, 0x9300);
    fill_suit(&h, 0x9300, 0x0600);
    fill_suit(&h, 0xa380, 0x0700);
    fill_suit(&h, 0x9400, 0x0800);
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
        (h.world.player.player.palette.hurt_flash, default_copies),
        (0x803d, 0),
        "visual.composition.misc_unsigned_carry"
    );
    let mut h = Composed::new();
    h.world.player.player.grapple = 0xc4f0;
    h.world.player.player.shine_tick = 2;
    h.world.player.player.palette.hurt_flash = 8;
    h.word(0x91, 0xd727, 0x9300);
    fill_suit(&h, 0x9300, 0x0600);
    samus::advance(&mut h);
    assert_eq!(
        (
            h.world.player.player.palette.hurt_flash,
            h.world.player.visual.colors[192]
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
        h.world.player.player.palette.hurt_flash = 39;
        let carry = samus::misc(&mut h);
        assert_eq!(
            (carry, h.world.player.player.palette.hurt_flash),
            (expected, 40),
            "{label}"
        );
    }
}
pub(super) fn run_selected(case: &str) {
    match case {
        "eye" => eye_then_palette(),
        "alias" => palette_timer_alias(),
        "heat" => heat_callback(),
        "upload_ack" => upload_ack(),
        "upload_failure" => upload_failure(),
        "samus" => samus_shared_paths(),
        "samus_carry" => samus_carry(),
        _ => panic!("unknown visual composition case"),
    }
}
pub(super) fn run() {
    for case in [
        "eye",
        "alias",
        "heat",
        "upload_ack",
        "upload_failure",
        "samus",
        "samus_carry",
    ] {
        run_selected(case);
    }
}

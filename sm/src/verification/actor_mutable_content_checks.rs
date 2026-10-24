//! Actual Kraid writer and live shot consumer; RAM replacement is component evidence.
#![deny(unsafe_op_in_unsafe_fn)]
use super::{
    content::{ActorContent, ActorDefinitionKey, ActorTemplateId, SpeciesId},
    identity::{ActorBirthOrigin, ActorProvenance},
    media::AppearanceId,
};
use crate::{
    game::session_runtime::Runtime,
    game_state::Game,
    ram_map::{word_arrays as arrays, words as ram},
    rom::{RomImage, lorom_offset},
    types::{EnemyDef, ExtendedSpriteMap, Hitbox},
};
use core::mem::offset_of;

const DEFINITION: u16 = 0xd000;
const SHOT: usize = offset_of!(EnemyDef, shot_ai);
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_ne_bytes());
}
fn image_word(image: &RomImage, address: u32, offset: usize, value: u16) {
    let at = lorom_offset(address) as usize + offset;
    for (lane, byte) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + lane, byte);
    }
}
fn bytes(image: &RomImage) -> Vec<u8> {
    (0..image.len()).map(|at| image.read(at)).collect()
}
fn native(g: &Game) -> [u16; 8] {
    [
        g.eproj_spawn_rect.x,
        g.eproj_spawn_rect.y,
        g.eproj_spawn_rect.w,
        g.eproj_spawn_rect.h,
        g.eproj_spawn_r22,
        g.eproj_spawn_pt.x,
        g.eproj_spawn_pt.y,
        g.eproj_spawn_varE24,
    ]
}
pub(crate) fn image() -> RomImage {
    let image = RomImage::zeroed(0x200000);
    image_word(&image, 0xa0d000, SHOT, 0x9000); // Nonignored gate; not dispatched by extended contact.
    image_word(&image, 0xa0d000, offset_of!(EnemyDef, touch_ai), 0x804c);
    image_word(&image, 0xa7c5e7, 0, 0xffff); // Packed sink table terminator before any callback.
    for i in 0..32 {
        image.write(lorom_offset(0xa786c7) as usize + i, 0x40 + i as u8);
    }
    image_word(&image, 0xa79000, 0, 1);
    image_word(
        &image,
        0xa79000,
        2 + offset_of!(ExtendedSpriteMap, hitbox_ptr_),
        0x9100,
    );
    image_word(&image, 0xa79100, 0, 1);
    for field in [offset_of!(Hitbox, left), offset_of!(Hitbox, top)] {
        image_word(&image, 0xa79100, 2 + field, 0xfff8);
    }
    for field in [offset_of!(Hitbox, right), offset_of!(Hitbox, bottom)] {
        image_word(&image, 0xa79100, 2 + field, 8);
    }
    image_word(&image, 0xa79100, 2 + offset_of!(Hitbox, func_ptrA), 0x804c); // Real A7804C no-op.
    image
}
pub(crate) fn seed(g: &mut Game, image: &RomImage) {
    // This is fixture construction, not a reset claim. Register BOTH owners: legacy
    // packed sink reads use native ROM; rom_record and content capture use loaded ROM.
    g.g_ram.fill(0);
    g.register_native_rom(image.clone());
    g.register_rom(image.clone());
    g.g_fail = false;
    put(&mut g.g_ram, ram::kraid_unk9000, 2);
    put(&mut g.g_ram, ram::camera_distance_index, 0x4567);
    put(&mut g.g_ram, ram::palette_change_num, 9);
    g.eproj_spawn_rect.x = 1;
    g.eproj_spawn_rect.y = 2;
    g.eproj_spawn_rect.w = 3;
    g.eproj_spawn_rect.h = 4;
    g.eproj_spawn_r22 = 5;
    g.eproj_spawn_pt.x = 6;
    g.eproj_spawn_pt.y = 7;
    g.eproj_spawn_varE24 = 8;
    let mut runtime = Runtime::new(g);
    let state = runtime.session_mut();
    state.actors.controls.selected = 0;
    let actor = &mut state.actors.slots[0];
    actor.definition = DEFINITION;
    actor.x.whole = 128;
    actor.y.whole = 607;
    actor.properties = 0x400;
    actor.extra_properties = 4;
    actor.bank = 0xa7;
    actor.spritemap = 0x9000;
    // All slots occupied: actual scatter runs, but cannot call an eproj initializer.
    for slot in &mut state.enemy_projectiles.slots {
        slot.definition = 1;
    }
    state.enemy_projectiles.spawn.primary = 0x5555;
    let pending = state
        .actor_identity
        .prepare_birth(
            0,
            ActorProvenance {
                authored_species: Some(SpeciesId(7)),
                origin: ActorBirthOrigin::Dynamic {
                    template: ActorTemplateId(11),
                    part: 0,
                },
            },
            Some(AppearanceId(20)),
        )
        .unwrap();
    state.actor_identity.commit_birth(pending);
}
pub(crate) fn assert_image(g: &Game, image: &RomImage, expected: &[u8], label: &str) {
    assert!(
        g.capture_native_rom().same_allocation(image),
        "actor.mutable.native_allocation {label}"
    );
    assert!(
        g.loaded_rom.as_ref().unwrap().same_allocation(image),
        "actor.mutable.loaded_allocation {label}"
    );
    assert_eq!(
        bytes(image) == expected,
        true,
        "actor.mutable.full_image {label}"
    );
    assert!(!g.g_fail, "actor.mutable.no_diagnostic {label}");
}
pub(crate) fn written_image(image: &RomImage) -> Vec<u8> {
    let mut expected = bytes(image);
    put(
        &mut expected,
        lorom_offset(0xa0d000) as usize + SHOT,
        0x804c,
    );
    expected
}
pub(crate) fn write_checked(g: &mut Game, image: &RomImage) {
    let expected_image = written_image(image);
    let mut expected = g.g_ram.to_vec();
    put(&mut expected, 0xf78 + 6, 608);
    put(&mut expected, 0xf78 + 0xe, 0);
    for slot in 1..6 {
        put(&mut expected, 0xf78 + slot * 64 + 0xe, 0x600);
    }
    put(&mut expected, 0xf78 + 0x30, 0xc715);
    put(&mut expected, ram::kraid_unk9000, 1);
    put(&mut expected, ram::enemy_bg2_tilemap_size, 2);
    put(&mut expected, ram::camera_distance_index, 0);
    put(&mut expected, ram::palette_change_num, 0);
    put(&mut expected, ram::random_number, 0x7c1d); // Exact sixteenth RNG result from seed0.
    put(&mut expected, 0x1993, 0);
    put(&mut expected, ram::vram_write_queue_tail, 7);
    expected[0xd0..0xd7].copy_from_slice(&[0, 2, 0x16, 0xa7, 0xa7, 0, 0x3f]);
    for i in 0..32 {
        expected[0xc200 + 96 * 2 + i] = 0x40 + i as u8;
    }
    let identity = g.actor_identity().clone();
    let pinned = Runtime::new(g).capture_actor_definition(ActorDefinitionKey(DEFINITION));
    assert!(pinned.record.image.same_allocation(image));
    assert_eq!(
        u16::from_ne_bytes([
            pinned.record.image.read(pinned.record.base + SHOT),
            pinned.record.image.read(pinned.record.base + SHOT + 1)
        ]),
        0x9000
    );
    // Actual production routine; every raw access is inside valid RAM/full ROM.
    crate::actor_kraid_writer_entry::write(g);
    assert_eq!(
        u16::from_ne_bytes([
            pinned.record.image.read(pinned.record.base + SHOT),
            pinned.record.image.read(pinned.record.base + SHOT + 1)
        ]),
        0x804c,
        "actor.mutable.actual_writer_updates_pinned_definition"
    );
    assert_image(g, image, &expected_image, "writer");
    assert_eq!(
        g.g_ram.as_slice() == expected.as_slice(),
        true,
        "actor.mutable.writer_exact_ram"
    );
    assert_eq!(
        native(g),
        [1, 2, 3, 4, 5, 157, 412, 0xe2bf],
        "actor.mutable.writer_exact_native"
    );
    assert_eq!(
        g.actor_identity(),
        &identity,
        "actor.mutable.writer_retains_identity"
    );
    for slot in 0..18 {
        assert_eq!(
            &g.g_ram[arrays::eproj_id + slot * 2..arrays::eproj_id + slot * 2 + 2],
            &1u16.to_ne_bytes(),
            "actor.mutable.full_pool_no_initializer"
        );
    }
}
pub(crate) fn consume_checked(g: &mut Game, ignored: bool, label: &str) {
    {
        let mut runtime = Runtime::new(g);
        let state = runtime.session_mut();
        state.actors.controls.selected = 0;
        let actor = &mut state.actors.slots[0];
        actor.definition = DEFINITION;
        actor.properties = 0;
        actor.invincibility = 0;
        actor.extra_properties = 4;
        actor.spritemap = 0x9000;
        actor.bank = 0xa7;
        state.projectiles.count = 1;
        state.projectiles.bomb_count = 0;
        let shot = &mut state.projectiles.shots[0];
        shot.kind = 1;
        shot.direction = 0;
        shot.x = actor.x.whole;
        shot.y = actor.y.whole;
        shot.x_radius = 2;
        shot.y_radius = 2;
        state.room.collision_detection_index = 0x7777;
    }
    let mut expected = g.g_ram.to_vec();
    put(&mut expected, ram::enemy_processing_stage, 6); // Actual final extended touch stage.
    if !ignored {
        put(&mut expected, arrays::projectile_dir, 0x10);
        put(&mut expected, ram::collision_detection_index, 0);
    }
    let identity = g.actor_identity().clone();
    let before_native = native(g);
    super::contacts::process_contacts(&mut Runtime::new(g));
    assert_eq!(
        g.g_ram.as_slice() == expected.as_slice(),
        true,
        "actor.mutable.live_shot_consumer {label}"
    );
    assert_eq!(
        native(g),
        before_native,
        "actor.mutable.consumer_native {label}"
    );
    assert_eq!(
        g.actor_identity(),
        &identity,
        "actor.mutable.consumer_identity {label}"
    );
    assert!(!g.g_fail, "actor.mutable.consumer_no_diagnostic {label}");
}
pub(crate) fn run_checked() {
    let mut game = Game::new();
    let image = image();
    seed(&mut game, &image);
    let before_ram = game.g_ram.to_vec();
    let before_payload = game.capture_actor_snapshot();
    let expected_image = written_image(&image);
    consume_checked(&mut game, false, "before_writer_positive_control");
    game.g_ram.copy_from_slice(&before_ram); // Only restore the positive-control fixture.
    write_checked(&mut game, &image);
    consume_checked(&mut game, true, "after_actual_writer");
    let restore = before_payload
        .preflight(game.actor_identity(), game.world())
        .unwrap();
    game.g_ram.copy_from_slice(&before_ram); // Explicit RAM replacement, not CPU RestoreSnapshot.
    restore.commit(game.actor_identity_mut());
    assert_eq!(game.g_ram.as_slice(), before_ram.as_slice());
    assert_eq!(
        game.capture_actor_snapshot(),
        before_payload,
        "actor.mutable.actual_payload_restored"
    );
    assert_image(
        &game,
        &image,
        &expected_image,
        "actor_payload_and_ram_replacement",
    );
    consume_checked(&mut game, true, "after_actor_payload_and_ram_replacement");
    let transition = game.prepare_actor_canonical();
    game.g_ram.fill(0); // Explicit component RAM replacement, NOT RtlReset.
    game.commit_actor_canonical(transition);
    assert!(game.g_ram.iter().all(|&byte| byte == 0));
    assert_image(&game, &image, &expected_image, "canonical_ram_replacement");
    consume_checked(&mut game, true, "after_canonical_ram_replacement");
    assert_image(&game, &image, &expected_image, "after_final_consumer");
}

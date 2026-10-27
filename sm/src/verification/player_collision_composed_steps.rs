//! Verification-only authored inputs and observations for composed collision.
//! Registered at verification root; safe actor access is a separate child module.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::{game::session_runtime::Runtime, game_state::Game, ram_map::words as ram};
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes({
        let mut bytes = [0; 2];
        g.ram().copy_out(at, &mut bytes);
        bytes
    })
}
fn write(image: &crate::rom::RomImage, address: u32, bytes: &[u8]) {
    let at = crate::rom::lorom_offset(address) as usize;
    for (i, &byte) in bytes.iter().enumerate() {
        image.write(at + i, byte);
    }
}
/// Synthetic header only: invokes the real supported CollisionSurface setup,
/// then the real delete instruction if a subsequent mechanism tick reaches it.
/// Both allocations are initialized because lookup and script consumers differ.
pub(crate) fn install_surface_content(g: &mut Game) {
    let loaded = g
        .loaded_rom
        .clone()
        .expect("collision.composed.loaded-image");
    let native = g.capture_native_rom();
    for image in [&loaded, &native] {
        write(image, 0x949139, &0xf100u16.to_le_bytes());
        let setup = (8697028u32 as u16).to_le_bytes();
        write(image, 0x84f100, &[setup[0], setup[1], 0x10, 0xf1]);
        write(image, 0x84f110, &(8685244u32 as u16).to_le_bytes());
    }
}
fn cell(g: &mut Game, x: u16, y: u16, level: u16, bts: u8) {
    let index = usize::from(y >> 4) * 96 + usize::from(x >> 4);
    put(g, 0x10002 + index * 2, level);
    g.ram().write_byte(0x16402 + index, bts);
}
fn position(g: &mut Game, x: u16, y: u16) {
    // Authored geometry/fraction input, not a collision/continuation result.
    for (at, value) in [
        (ram::samus_x_pos, x),
        (ram::samus_y_pos, y),
        (ram::samus_x_subpos, 0),
        (ram::samus_y_subpos, 0),
    ] {
        put(g, at, value);
    }
}
fn trace(out: &mut Vec<u8>, phase: u8, g: &Game) {
    out.push(phase);
    // Preserve every intermediate byte before the next authored input edit.
    out.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    out.push(u8::from(g.g_fail));
}
pub(crate) unsafe fn run(g: &mut Game, reversed: bool, out: &mut Vec<u8>) {
    unsafe {
        // Existing authored births and scheduling already ran. Make one actual
        // source actor solid; retain its definition, handle, position and media.
        let actor_index =
            crate::game::actors::collision_composed_actor::prepare_solid_actor(g, reversed);
        // Source room contains full 96x96 geometry; small player radii make each
        // authored probe touch exactly the documented cells.
        for (at, value) in [
            (ram::samus_x_radius, 1),
            (ram::samus_y_radius, 1),
            (ram::room_width_in_blocks, 96),
            (ram::room_height_in_blocks, 96),
            (ram::samus_x_accel_mode, 0),
            (ram::samus_x_speed_divisor, 0),
            (ram::samus_x_extra_run_speed, 0),
            (ram::samus_x_extra_run_subspeed, 0),
            (ram::extra_samus_x_displacement, 0),
            (ram::extra_samus_x_subdisplacement, 0),
            (ram::samus_x_base_speed, 0),
            (ram::samus_x_base_subspeed, 0),
            (ram::samus_y_speed, 0),
            (ram::samus_y_subspeed, 0),
            (ram::enable_horiz_slope_coll, 0),
            (0xa1e, 8),
        ] {
            put(g, at, value);
        }
        position(g, 0x136, 0x240);
        crate::game::player_collision::verification::move_x(2 << 16, g);
        assert_eq!(
            word(g, ram::samus_x_pos),
            0x137,
            "collision.composed.solid-permitted-one-pixel"
        );
        assert_eq!(
            word(g, ram::samus_collision_flag),
            0xffff,
            "collision.composed.actual-solid-contact"
        );
        assert_eq!(
            word(g, ram::collision_detection_index),
            actor_index,
            "collision.composed.reordered-solid-identity"
        );
        assert_eq!(word(g, ram::samus_collides_with_solid_enemy), 8);
        trace(out, 10, g);

        // Move to a slope tile away from every source actor. Horizontal movement
        // traverses the nonsquare slope, then performs actual bottom alignment.
        position(g, 0x88, 0x8f);
        cell(g, 0x89, 0x8f, 0x1000, 5);
        put(g, ram::enable_horiz_slope_coll, 2);
        crate::game::player_collision::verification::move_x(1 << 16, g);
        assert_eq!(
            word(g, ram::samus_x_pos),
            0x89,
            "collision.composed.slope-horizontal-commit"
        );
        assert_eq!(
            word(g, ram::samus_y_pos),
            0x89,
            "collision.composed.actual-slope-alignment"
        );
        assert_eq!(word(g, ram::samus_pos_adjusted_by_slope_flag), 1);
        trace(out, 11, g);

        // Vertical outer entry reaches a real collision-spawned mechanism. The
        // original requested18.205-pixel amount becomes fraction0x3078 in setup;
        // committing that changed local proves caller/setup displacement sharing.
        position(g, 0xa8, 0xaf);
        cell(g, 0xa8, 0xc1, 0x3000, 0);
        put(g, ram::samus_y_dir, 0);
        put(g, ram::samus_contact_damage_index, 0);
        crate::game::player_collision::verification::move_down(0x123478, g);
        assert_eq!(
            word(g, ram::samus_y_pos),
            0xaf,
            "collision.composed.surface-local-whole"
        );
        assert_eq!(
            word(g, ram::samus_y_subpos),
            0x3078,
            "collision.composed.surface-local-fraction"
        );
        assert_eq!(
            word(g, 0x1e71),
            1,
            "collision.composed.actual-surface-visit"
        );
        {
            let h = Runtime::new(g);
            assert!(
                h.session()
                    .mechanisms
                    .slots
                    .iter()
                    .any(|s| s.header == 0xf100),
                "collision.composed.actual-mechanism-created"
            );
        }
        trace(out, 12, g);

        // Reposition at the still-live actor. Facing right while holding left
        // makes the retained spin controller stop X before probing the right wall.
        // Jump readiness/binding/pressed keys are fixture inputs; the operation
        // must produce input_to_pose_calc and actor shake identity itself.
        position(g, 0x137, 0x240);
        for (at, value) in [
            (ram::enable_horiz_slope_coll, 0),
            (ram::samus_pose, 0x1b),
            (0xa1e, 0x0308),
            (0xa26, 0x0308),
            (ram::samus_anim_frame, 11),
            (ram::button_config_jump_a, 0x8000),
            (ram::joypad1_lastkeys, 0x8200),
            (ram::joypad1_newkeys, 0x8000),
            (ram::samus_x_speed_table_pointer, 0x9f49),
            (ram::samus_x_accel_mode, 0),
            (ram::samus_x_base_speed, 0),
            (ram::samus_x_base_subspeed, 0),
        ] {
            put(g, at, value);
        }
        crate::game::player_movement::speed::spin_jumping(&mut Runtime::new(g));
        assert_eq!(
            word(g, ram::input_to_pose_calc),
            5,
            "collision.composed.actual-wall-admission"
        );
        assert_eq!(
            word(g, ram::enemy_index_to_shake),
            actor_index,
            "collision.composed.wall-same-live-identity"
        );
        assert_eq!(word(g, ram::samus_x_pos), 0x137);
        assert_eq!(
            word(g, ram::samus_y_pos),
            0x240,
            "collision.composed.wall-skips-vertical-movement"
        );
        trace(out, 13, g);
        assert!(!g.g_fail, "collision.composed.phases-no-diagnostic");
    }
}

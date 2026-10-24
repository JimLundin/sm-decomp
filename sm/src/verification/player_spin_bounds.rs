//! Independent byte-pattern proof through the actual owned normal-spin dispatch.
//! Candidate only: never execute the former unaligned Rust load as a reference.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::{game::player_movement, game_state::Game, ram_map::words as w, rom::RomImage};

// Authored ordinary-integer interval facts: air [640,1280), liquid [128,1280).
// Signed-wrap points are explicit rejection rows, not evaluated by the core helper.
const CASES: [(u16, bool, bool); 16] = [
    (0x0000, false, false),
    (0x007f, false, false),
    (0x0080, false, true),
    (0x027f, false, true),
    (0x0280, true, true),
    (0x04ff, true, true),
    (0x0500, false, false),
    (0x7fff, false, false),
    (0x8000, false, false),
    (0x807f, false, false),
    (0x8080, false, false),
    (0x827f, false, false),
    (0x8280, false, false),
    (0x84ff, false, false),
    (0x8500, false, false),
    (0xffff, false, false),
];
pub(crate) const COUNT: usize = CASES.len() * 2 * 2 * 3;
fn put(g: &mut Game, at: usize, value: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.g_ram[at], g.g_ram[at + 1]])
}

/// Called from the existing initialized collision capture, with Game authority.
/// No file output, CPU oracle claim, unsafe entry, or production decision replica.
pub(crate) fn run(g: &mut Game) -> usize {
    assert!(!g.actor_identity_is_with_session());
    let saved = g.g_ram.to_vec();
    let fail = g.g_fail;
    let loaded = g.loaded_rom.clone();
    let native = g.capture_native_rom_binding();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Only speed calculation reads content: actual Native movement3 record.
        let content = RomImage::zeroed(0x200000);
        let at = crate::rom::lorom_offset(0x90e024) as usize;
        for (i, value) in [0u16, 0, 8, 0, 2, 0].into_iter().enumerate() {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                content.write(at + i * 2 + lane, byte);
            }
        }
        g.loaded_rom = None;
        g.register_native_rom(content);
        let mut checked = 0;
        for (middle, air, liquid) in CASES {
            // Independent verification of the authored truth table uses unsigned
            // numerical intervals, not wrapping-subtraction or FixedMotion::middle.
            assert_eq!(air, (640..1280).contains(&u32::from(middle)));
            assert_eq!(liquid, (128..1280).contains(&u32::from(middle)));
            for liquid_physics in [0u16, 1] {
                for low_fraction in [0u8, 0xa5] {
                    for high_whole in [0u8, 0x80, 0xff] {
                        g.g_ram.fill(0);
                        g.g_fail = false;
                        for (at, value) in [
                            (w::room_width_in_blocks, 32),
                            (w::room_height_in_blocks, 32),
                            (w::samus_x_pos, 136),
                            (w::samus_y_pos, 136),
                            (w::samus_x_subpos, 0x1111),
                            (w::samus_y_subpos, 0x2222),
                            (w::samus_prev_x_pos, 136),
                            (w::samus_prev_y_pos, 136),
                            (w::samus_x_radius, 3),
                            (w::samus_y_radius, 3),
                            (w::equipped_items, 0x220),
                            (w::samus_suit_palette_index, 4),
                            (w::samus_pose, 1),
                            (0xa1e, 0x0308),
                            (w::samus_x_speed_table_pointer, 0xe000),
                            (w::speed_echoes_index, 0xffff),
                            (w::samus_y_dir, 2),
                            (w::samus_anim_frame_timer, 2),
                            (w::button_config_jump_a, 0x80),
                            (w::liquid_physics_type, liquid_physics),
                            (0xdfa, 0xa55a),
                        ] {
                            put(g, at, value);
                        }
                        // B2D and B2E are the middle two physical bytes. Vary both
                        // excluded outer bytes independently, including the sign bit.
                        let bytes = [low_fraction, middle as u8, (middle >> 8) as u8, high_whole];
                        g.g_ram[0xb2c..0xb30].copy_from_slice(&bytes);
                        assert_eq!(u16::from_le_bytes([bytes[1], bytes[2]]), middle);
                        // No jump is requested. Cancel the original Y displacement
                        // with authored external displacement, so arbitrary outer
                        // bytes cannot cause an unrelated long terrain traversal.
                        // Acceleration remains zero; this does not bypass movement.
                        let opposite = 0u32.wrapping_sub(u32::from_le_bytes(bytes));
                        put(g, w::extra_samus_y_subdisplacement, opposite as u16);
                        put(g, w::extra_samus_y_displacement, (opposite >> 16) as u16);
                        {
                            let mut runtime = Runtime::new(g);
                            player_movement::advance_normal(&mut runtime);
                        }
                        let accepted = if liquid_physics == 0 { air } else { liquid };
                        assert_eq!(
                            word(g, 0xdfa),
                            if accepted { 0xa501 } else { 0xa55a },
                            "owned spin window: liquid={liquid_physics} middle={middle:04x} low={low_fraction:02x} high={high_whole:02x}"
                        );
                        assert_eq!(
                            &g.g_ram[0xb2c..0xb30],
                            &bytes,
                            "no jump request changes speed bytes"
                        );
                        assert_eq!(word(g, w::samus_y_dir), 2);
                        assert_eq!(word(g, w::samus_y_pos), 136);
                        assert_eq!(word(g, w::samus_y_subpos), 0x2222);
                        assert!(!g.g_fail && !g.actor_identity_is_with_session());
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, COUNT);
        checked
    }));
    g.g_ram.copy_from_slice(&saved);
    g.g_fail = fail;
    g.loaded_rom = loaded;
    g.restore_native_rom_binding(native);
    match result {
        Ok(checked) => checked,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

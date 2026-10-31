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
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.ram().read_byte(at), g.ram().read_byte(at + 1)])
}

/// Called from the existing initialized collision capture, with Game authority.
/// No file output, CPU oracle claim, unsafe entry, or production decision replica.
pub(crate) fn run(g: &mut Game) -> usize {
    assert!(!g.actor_identity_is_with_session());
    let saved = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let fail = g.g_fail;
    let loaded = g.loaded_rom.clone();
    let native = g.capture_native_rom_binding();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Only speed calculation reads content: actual Native movement3 record.
        let mut content = vec![0; 0x200000];
        let at = crate::rom::lorom_offset(0x90e024) as usize;
        for (i, value) in [0u16, 0, 8, 0, 2, 0].into_iter().enumerate() {
            content[at + i * 2..at + i * 2 + 2].copy_from_slice(&value.to_le_bytes());
        }
        g.loaded_rom = None;
        g.register_native_rom(RomImage::from(content));
        let mut checked = 0;
        for (middle, air, liquid) in CASES {
            // Independent verification of the authored truth table uses unsigned
            // numerical intervals, not wrapping-subtraction or FixedMotion::middle.
            assert_eq!(air, (640..1280).contains(&u32::from(middle)));
            assert_eq!(liquid, (128..1280).contains(&u32::from(middle)));
            for liquid_physics in [0u16, 1] {
                for low_fraction in [0u8, 0xa5] {
                    for high_whole in [0u8, 0x80, 0xff] {
                        g.ram().fill(0..g.ram().len(), 0);
                        g.g_fail = false;
                        for (at, value) in [
                            (w::ROOM_WIDTH_IN_BLOCKS, 32),
                            (w::ROOM_HEIGHT_IN_BLOCKS, 32),
                            (w::SAMUS_X_POS, 136),
                            (w::SAMUS_Y_POS, 136),
                            (w::SAMUS_X_SUBPOS, 0x1111),
                            (w::SAMUS_Y_SUBPOS, 0x2222),
                            (w::SAMUS_PREV_X_POS, 136),
                            (w::SAMUS_PREV_Y_POS, 136),
                            (w::SAMUS_X_RADIUS, 3),
                            (w::SAMUS_Y_RADIUS, 3),
                            (w::EQUIPPED_ITEMS, 0x220),
                            (w::SAMUS_SUIT_PALETTE_INDEX, 4),
                            (w::SAMUS_POSE, 1),
                            (0xa1e, 0x0308),
                            (w::SAMUS_X_SPEED_TABLE_POINTER, 0xe000),
                            (w::SPEED_ECHOES_INDEX, 0xffff),
                            (w::SAMUS_Y_DIR, 2),
                            (w::SAMUS_ANIM_FRAME_TIMER, 2),
                            (w::BUTTON_CONFIG_JUMP_A, 0x80),
                            (w::LIQUID_PHYSICS_TYPE, liquid_physics),
                            (0xdfa, 0xa55a),
                        ] {
                            put(g, at, value);
                        }
                        // B2D and B2E are the middle two physical bytes. Vary both
                        // excluded outer bytes independently, including the sign bit.
                        let bytes = [low_fraction, middle as u8, (middle >> 8) as u8, high_whole];
                        g.ram().copy_in(0xb2c, &bytes);
                        assert_eq!(u16::from_le_bytes([bytes[1], bytes[2]]), middle);
                        // No jump is requested. Cancel the original Y displacement
                        // with authored external displacement, so arbitrary outer
                        // bytes cannot cause an unrelated long terrain traversal.
                        // Acceleration remains zero; this does not bypass movement.
                        let opposite = 0u32.wrapping_sub(u32::from_le_bytes(bytes));
                        put(g, w::EXTRA_SAMUS_Y_SUBDISPLACEMENT, opposite as u16);
                        put(g, w::EXTRA_SAMUS_Y_DISPLACEMENT, (opposite >> 16) as u16);
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
                            &{
                                let mut bytes = [0; 4];
                                g.ram().copy_out(0xb2c, &mut bytes);
                                bytes
                            },
                            &bytes,
                            "no jump request changes speed bytes"
                        );
                        assert_eq!(word(g, w::SAMUS_Y_DIR), 2);
                        assert_eq!(word(g, w::SAMUS_Y_POS), 136);
                        assert_eq!(word(g, w::SAMUS_Y_SUBPOS), 0x2222);
                        assert!(!g.g_fail && !g.actor_identity_is_with_session());
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, COUNT);
        checked
    }));
    g.ram().copy_in(0, &saved);
    g.g_fail = fail;
    g.loaded_rom = loaded;
    g.restore_native_rom_binding(native);
    match result {
        Ok(checked) => checked,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

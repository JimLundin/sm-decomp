//! Six candidate-only physical-byte contracts for actual owned minimap update.
//! No unaligned predecessor Rust access is executed or treated as an oracle.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::{
    game::{player_update, session_encoding},
    game_state::Game,
    ram_map::words as w,
    rom::RomImage,
};
pub(crate) const COUNT: usize = 6;
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn rom_bytes(image: &RomImage, address: u32, bytes: &[u8]) {
    let at = crate::rom::lorom_offset(address) as usize;
    for (i, b) in bytes.iter().copied().enumerate() {
        image.write(at + i, b);
    }
}
fn one(g: &mut Game, case: usize) {
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    // Deliberately distinct surrounding HUD words; only the15 minimap cells belong to this update.
    for word in 0..96 {
        g.ram()
            .copy_in(0xc608 + 2 * word, &(0x5000 + word as u16).to_le_bytes());
    }
    for (at, value) in [
        (w::samus_x_pos, 512),
        (w::samus_y_pos, 256),
        (w::room_width_in_blocks, 64),
        (w::room_height_in_blocks, 32),
        (w::room_x_coordinate_on_map, 0),
        (w::room_y_coordinate_on_map, 0),
        (w::has_area_map, if case == 3 { 0 } else { 1 }),
    ] {
        g.ram().copy_in(at, &(value as u16).to_le_bytes());
    }
    g.ram().write_byte(0x5b5, if case == 5 { 0 } else { 8 });
    g.loaded_rom = None;
    g.register_native_rom(RomImage::zeroed(0));
    match case {
        0 => g
            .ram()
            .copy_in(w::debug_disable_minimap, &0x8000u16.to_le_bytes()),
        1 => g
            .ram()
            .copy_in(w::room_width_in_blocks, &32u16.to_le_bytes()),
        2 => g
            .ram()
            .copy_in(w::room_height_in_blocks, &16u16.to_le_bytes()),
        _ => {
            let loaded = RomImage::zeroed(0x200000);
            let native = RomImage::zeroed(0x200000);
            // Legacy mask pointer is Loaded, but mask data is Native bank82.
            rom_bytes(&loaded, 0x829717, &0x9000u16.to_le_bytes());
            rom_bytes(&native, 0x829000, &[0xff; 256]);
            // Tile24 pointer is Native, including its fourth read byte (zero).
            rom_bytes(&native, 0x82964a, &[0x00, 0x80, 0x90, 0x00]);
            for (index, value) in [
                (32, 0xc001u16),
                (33, 0xc002),
                (34, 0xc003),
                (35, 0xc004),
                (36, 0xc005),
                (64, 0x4006),
                (65, 0x4007),
                (66, 0x4028),
                (67, 0x4009),
                (68, 0x400a),
                (96, 0x800b),
                (97, 0x800c),
                (98, 0x800d),
                (99, 0x800e),
                (100, 0x800f),
            ] {
                rom_bytes(&native, 0x908000 + 2 * index, &value.to_le_bytes());
            }
            g.register_rom(loaded);
            g.register_native_rom(native);
        }
    }
    let before = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let mut expected = before.clone();
    if case >= 3 {
        // Fixed authored coordinate: xscreen2, yscreen1 plus map's +1 yieldsrow2.
        // Physical exploration offsets8 (current) and4 (tile40's preceding row), bit20.
        expected[0x7ff] = 0x20;
        expected[0x7fb] = 0x20;
        let mapped = [
            [0xec01u16, 0xec02, 0xec03, 0xec04, 0xec05],
            [
                0x6c06,
                0x6c07,
                if case == 5 { 0x7c28 } else { 0x6828 },
                0x6c09,
                0x6c0a,
            ],
            [0xac0b, 0xac0c, 0xac0d, 0xac0e, 0xac0f],
        ];
        for row in 0..3 {
            for column in 0..5 {
                let value = if case == 3 && (row, column) != (1, 2) {
                    0x2c1f
                } else {
                    mapped[row][column]
                };
                put(
                    &mut expected,
                    0xc608 + 2 * ([26, 58, 90][row] + column),
                    value,
                );
            }
        }
    }
    {
        let mut runtime = Runtime::new(g);
        player_update::minimap::update(&mut runtime);
        let mut projected = before.clone();
        session_encoding::encode(runtime.session(), |at, span| {
            projected[at..at + span.len()].copy_from_slice(span)
        });
        assert_eq!(
            projected, expected,
            "owned minimap physical contract case{case}"
        );
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "minimap full RAM publication case{case}"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
pub(crate) fn run(g: &mut Game) -> usize {
    assert!(!g.actor_identity_is_with_session());
    let ram = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let fail = g.g_fail;
    let loaded = g.loaded_rom.clone();
    let native = g.capture_native_rom_binding();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for case in 0..COUNT {
            one(g, case);
        }
        codec_bytes(g);
        COUNT
    }));
    g.ram().copy_in(0, &ram);
    g.g_fail = fail;
    g.loaded_rom = loaded;
    g.restore_native_rom_binding(native);
    match result {
        Ok(count) => count,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

// Follow-on insertion into player_minimap_bytes before returning COUNT.
fn codec_bytes(g: &mut Game) {
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    for word in 0..96 {
        g.ram().copy_in(0xc608 + 2 * word, &0x5678u16.to_le_bytes());
    }
    for row in 0..3 {
        for column in 0..5 {
            g.ram().copy_in(
                0xc608 + 2 * ([26, 58, 90][row] + column),
                &0x1234u16.to_le_bytes(),
            );
        }
    }
    g.ram().copy_in(0x5f7, &0xb2a1u16.to_le_bytes());
    let before = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let mut expected = before.clone();
    {
        let mut runtime = Runtime::new(g);
        for row in 0..3 {
            for column in 0..5 {
                let at = 0xc608 + 2 * ([26, 58, 90][row] + column);
                assert!(session_encoding::write_projected_byte(
                    runtime.session_mut(),
                    at,
                    0xde
                ));
                assert_eq!(
                    runtime.session().minimap.tiles[row][column],
                    0x12de,
                    "HUD low-byte projection preserves high byte"
                );
                assert!(session_encoding::write_projected_byte(
                    runtime.session_mut(),
                    at + 1,
                    0xa5
                ));
                assert_eq!(
                    runtime.session().minimap.tiles[row][column],
                    0xa5de,
                    "HUD high-byte projection preserves low byte"
                );
                put(&mut expected, at, 0xa5de);
            }
        }
        assert!(session_encoding::write_projected_byte(
            runtime.session_mut(),
            0x5f7,
            0x7e
        ));
        assert_eq!(
            runtime.session().minimap.disabled,
            0xb27e,
            "suppression low-byte alias preserves high byte"
        );
        assert!(session_encoding::write_projected_byte(
            runtime.session_mut(),
            0x5f8,
            0xd4
        ));
        assert_eq!(
            runtime.session().minimap.disabled,
            0xd47e,
            "suppression high-byte alias preserves low byte"
        );
        put(&mut expected, 0x5f7, 0xd47e);
        for (index, value) in [(0, 0x39), (255, 0xc6)] {
            assert!(session_encoding::write_projected_byte(
                runtime.session_mut(),
                0x7f7 + index,
                value
            ));
            assert_eq!(runtime.session().minimap.explored[index], value);
            expected[0x7f7 + index] = value;
        }
        // Exercise the retained room-entry adapters, not just their numeric addresses.
        use crate::game::room_entry::effects::{EntryEffects, ExternalField};
        EntryEffects::write_external(&mut runtime, ExternalField::LoadingLowByte, 0);
        assert_eq!(
            runtime.session().minimap.disabled,
            0xd400,
            "LoadingLowByte preserves suppression high byte"
        );
        assert_eq!(
            EntryEffects::read_external(&mut runtime, ExternalField::LoadingLowByte),
            0
        );
        assert_eq!(
            EntryEffects::read_external(&mut runtime, ExternalField::DebugMinimap),
            0xd400
        );
        EntryEffects::write_external(&mut runtime, ExternalField::DebugMinimap, 0);
        assert_eq!(
            runtime.session().minimap.disabled,
            0,
            "DebugMinimap clears the owned suppression word"
        );
        put(&mut expected, 0x5f7, 0);
        let mut projected = before.clone();
        session_encoding::encode(runtime.session(), |at, span| {
            projected[at..at + span.len()].copy_from_slice(span)
        });
        assert_eq!(projected, expected, "sparse minimap byte projection");
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "sparse minimap codec preserves unrelated HUD bytes"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}

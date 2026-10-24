//! Candidate-only contracts for retained callbacks and physical map writes.
//! Runs on an initialized verification host; no predecessor unaligned u16 path.
use super::Runtime;
use crate::{game_state::Game, ram_map::words as w, rom::RomImage};
use std::cell::RefCell;
pub(crate) const COUNT: usize = 2;

#[derive(Default)]
struct Observation {
    special: bool,
    entered: bool,
    transfers: Vec<u8>,
    map: bool,
    shifted: usize,
    incremented: usize,
    completed: usize,
}
thread_local! {
    static OBS: RefCell<Observation> = RefCell::new(Observation::default());
}
fn read(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes(g.g_ram[at..at + 2].try_into().unwrap())
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn author(r: &RomImage, address: u32, bytes: &[u8]) {
    let at = crate::rom::lorom_offset(address) as usize;
    for (i, value) in bytes.iter().copied().enumerate() {
        r.write(at + i, value);
    }
}
// 0 = constructor entry; 1 = import entry; 2 = export entry.
pub(crate) fn transfer(g: &Game, stage: u8) {
    OBS.with(|cell| {
        let mut o = cell.borrow_mut();
        if !o.special {
            return;
        }
        assert!(
            o.entered,
            "retained movement must precede any Runtime/transfer"
        );
        assert_eq!(read(g, w::samus_movement_handler), 0xa337);
        assert_eq!(read(g, w::bomb_jump_dir), 0);
        assert_eq!(g.actor_identity_is_with_session(), stage == 2);
        o.transfers.push(stage);
    });
}
pub(crate) fn special_entry(g: &Game) {
    OBS.with(|cell| {
        let mut o = cell.borrow_mut();
        if !o.special {
            return;
        }
        assert!(!o.entered && o.transfers.is_empty());
        assert!(!g.actor_identity_is_with_session());
        assert_eq!(
            read(g, w::samus_contact_damage_index),
            0,
            "preclear before special"
        );
        assert_eq!(read(g, w::samus_movement_handler), 0xe07d);
        assert_eq!(read(g, w::bomb_jump_dir), 3);
        o.entered = true;
    });
}
fn explored(byte: usize) -> u8 {
    (byte as u8).wrapping_mul(73).wrapping_add(0x81)
}
fn tile(index: usize) -> u16 {
    0x8400 | index as u16
}
// 0 = after shift; 1 = after carry reinsertion; 2 = after tile publication.
pub(crate) fn map_write(g: &Game, byte: usize, index: usize, stage: u8) {
    OBS.with(|cell| {
        let mut o = cell.borrow_mut();
        if !o.map {
            return;
        }
        assert!(
            !g.actor_identity_is_with_session(),
            "retained writer owns physical RAM"
        );
        assert_eq!(byte, index / 8);
        let before = explored(byte).rotate_left((index % 8) as u32);
        let expected = if stage == 0 {
            before.wrapping_shl(1)
        } else {
            before.rotate_left(1)
        };
        assert_eq!(
            g.g_ram[0x7f7 + byte],
            expected,
            "map intermediate byte {index}, stage {stage}"
        );
        if stage != 2 {
            assert_eq!(
                read(g, 0x4000 + index * 2),
                0xdead,
                "exploration precedes tile publication"
            );
        } else {
            assert_eq!(
                read(g, 0x4000 + index * 2),
                if before & 0x80 == 0 {
                    31
                } else {
                    tile(index) & 0xfbff
                }
            );
        }
        match stage {
            0 => {
                assert_eq!(o.shifted, index);
                assert_eq!(o.completed, index);
                o.shifted += 1;
            }
            1 => {
                assert_ne!(before & 0x80, 0);
                o.incremented += 1;
            }
            2 => {
                assert_eq!(o.shifted, index + 1);
                assert_eq!(o.completed, index);
                o.completed += 1;
            }
            _ => unreachable!(),
        }
    });
}
fn special(g: &mut Game) {
    g.g_ram.fill(0);
    g.g_fail = false;
    let loaded = RomImage::zeroed(0x100000);
    let native = RomImage::zeroed(0x100000);
    let mut poses = vec![0; 256 * 8];
    for pose in poses.chunks_exact_mut(8) {
        pose[0] = 8;
        pose[6] = 2;
    }
    author(&loaded, 0x91b629, &poses);
    author(&native, 0x91b629, &poses);
    g.register_rom(loaded);
    g.register_native_rom(native);
    for (at, value) in [
        (w::samus_pose, 1),
        (0xa1e, 0x0908),
        (w::samus_prev_pose, 1),
        (w::samus_movement_handler, 0xe07d),
        (w::samus_input_handler, 0xe91d),
        (w::bomb_jump_dir, 3),
        (w::frame_handler_beta, 0xe8ec),
        (w::debug_disable_minimap, 1),
        (w::samus_contact_damage_index, 7),
        (w::samus_anim_frame_timer, 2),
        (w::samus_y_pos, 100),
        (w::samus_y_radius, 2),
        // An erroneous Normal redispatch would publish an echo and advance its index.
        (w::speed_boost_counter, 0x400),
        (w::samus_x_pos, 123),
    ] {
        put(&mut g.g_ram, at, value);
    }
    let mut expected = g.g_ram.to_vec();
    for (at, value) in [
        (w::samus_contact_damage_index, 0),
        (w::samus_movement_handler, 0xa337),
        (w::bomb_jump_dir, 0),
        (w::samus_anim_frame_timer, 1),
    ] {
        put(&mut expected, at, value);
    }
    OBS.with(|o| {
        *o.borrow_mut() = Observation {
            special: true,
            ..Default::default()
        }
    });
    unsafe {
        crate::game::sm_90::HandleSamusMovementAndPause(g);
    }
    OBS.with(|cell| {
        let o = cell.borrow();
        assert!(o.entered);
        assert_eq!(
            o.transfers,
            [0, 1, 2],
            "exactly one Runtime import/export after retained special"
        );
    });
    assert_eq!(
        g.g_ram.as_slice(),
        expected.as_slice(),
        "special changes handler without redispatch"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
    OBS.with(|o| *o.borrow_mut() = Observation::default());
}
fn map(g: &mut Game) {
    g.g_ram.fill(0);
    g.g_fail = false;
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    author(&loaded, 0x829717, &0x9000u16.to_le_bytes());
    author(&native, 0x82964a, &[0x00, 0x80, 0x90, 0x00]);
    for index in 0..2048 {
        author(
            &native,
            0x908000 + index as u32 * 2,
            &tile(index).to_le_bytes(),
        );
    }
    g.register_rom(loaded);
    g.register_native_rom(native);
    for byte in 0..256 {
        g.g_ram[0x7f7 + byte] = explored(byte);
    }
    for index in 0..2048 {
        put(&mut g.g_ram, 0x4000 + 2 * index, 0xdead);
    }
    let mut expected = g.g_ram.to_vec();
    for index in 0..2048 {
        put(
            &mut expected,
            0x4000 + 2 * index,
            if explored(index / 8) & (0x80 >> (index % 8)) == 0 {
                31
            } else {
                tile(index) & 0xfbff
            },
        );
    }
    OBS.with(|o| {
        *o.borrow_mut() = Observation {
            map: true,
            ..Default::default()
        }
    });
    // Flag zero is intentional: the mapped branch dereferences an odd u16 address.
    unsafe {
        crate::game::sm_82::LoadPauseMenuMapTilemap(g);
    }
    OBS.with(|cell| {
        let o = cell.borrow();
        assert_eq!((o.shifted, o.incremented, o.completed), (2048, 1024, 2048));
    });
    assert_eq!(
        g.g_ram.as_slice(),
        expected.as_slice(),
        "all map rotations restore exploration bytes"
    );
    OBS.with(|o| *o.borrow_mut() = Observation::default());
    {
        let runtime = Runtime::new(g);
        assert_eq!(
            runtime.session().minimap.explored.as_slice(),
            &expected[0x7f7..0x8f7]
        );
    }
    assert_eq!(
        g.g_ram.as_slice(),
        expected.as_slice(),
        "Session adopts retained physical map writes"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
pub(crate) fn run(g: &mut Game) -> usize {
    assert!(cfg!(target_endian = "little"));
    assert_eq!(g.g_ram.as_ptr() as usize % core::mem::align_of::<u16>(), 0);
    assert!(!g.actor_identity_is_with_session());
    let ram = g.g_ram.to_vec();
    let fail = g.g_fail;
    let loaded = g.loaded_rom.clone();
    let native = g.capture_native_rom_binding();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        special(g);
        map(g);
        COUNT
    }));
    OBS.with(|o| *o.borrow_mut() = Observation::default());
    g.g_ram.copy_from_slice(&ram);
    g.g_fail = fail;
    g.loaded_rom = loaded;
    g.restore_native_rom_binding(native);
    match result {
        Ok(count) => count,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

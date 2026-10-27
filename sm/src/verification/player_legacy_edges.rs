//! Candidate-only contracts for retained callbacks and physical map writes.
//! Runs on an initialized verification host; no predecessor unaligned u16 path.
use super::Runtime;
use crate::game::map_menu::{MapScreen, render_map};
use crate::{game_state::Game, ram_map::words as w, rom::RomImage};
use std::cell::RefCell;
pub(crate) const COUNT: usize = 1;

#[derive(Default)]
struct Observation {
    map: bool,
    shifted: usize,
    incremented: usize,
    completed: usize,
}
thread_local! {
    static OBS: RefCell<Observation> = RefCell::new(Observation::default());
}
fn read(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes({
        let mut bytes = [0; 2];
        g.ram().copy_out(at, &mut bytes);
        bytes
    })
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
            g.ram().read_byte(0x7f7 + byte),
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
fn map(g: &mut Game) {
    g.ram().fill(0..g.ram().len(), 0);
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
        g.ram().write_byte(0x7f7 + byte, explored(byte));
    }
    for index in 0..2048 {
        g.ram()
            .copy_in(0x4000 + 2 * index, &0xdeadu16.to_le_bytes());
    }
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
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
    // Retain the flag-zero rotations and every intermediate physical write.
    render_map(MapScreen::Pause, g);
    OBS.with(|cell| {
        let o = cell.borrow();
        assert_eq!((o.shifted, o.incremented, o.completed), (2048, 1024, 2048));
    });
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
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
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "Session adopts retained physical map writes"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
pub(crate) fn run(g: &mut Game) -> usize {
    assert!(cfg!(target_endian = "little"));
    assert!(!g.actor_identity_is_with_session());
    let ram = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let fail = g.g_fail;
    let loaded = g.loaded_rom.clone();
    let native = g.capture_native_rom_binding();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        map(g);
        COUNT
    }));
    OBS.with(|o| *o.borrow_mut() = Observation::default());
    g.ram().copy_in(0, &ram);
    g.g_fail = fail;
    g.loaded_rom = loaded;
    g.restore_native_rom_binding(native);
    match result {
        Ok(count) => count,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

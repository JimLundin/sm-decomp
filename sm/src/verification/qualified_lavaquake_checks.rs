//! Independent lavaquake byte expectations; NEVER run old unaligned-u32 paths.
//! Child of candidate carrier. Raw movement samples differ from loaded F words.
use super::{bytes, put, seed, slot};
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
pub(crate) const CASES: u32 = 14;
fn write(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn check_case(id: u32) -> bool {
    if id >= CASES {
        return false;
    }
    // Explicit independently calculated table-byte offsets and final phase counts.
    // Func1 visits descending count+1,count; Func2 visits ascending count-1,count.
    let (function, before, after, positions): (u32, u16, u16, Option<[isize; 2]>) = match id {
        0 | 10 | 12 => (8829974, 2, 0, Some([12, 4])),
        1 => (8829974, 3, 1, Some([20, 12])),
        2 => (8829974, 0x8001, 0x7fff, Some([4, 262140])),
        3 => (8829974, 0x8002, 0, None),
        4 | 11 | 13 => (8830095, 0, 2, Some([8, 16])),
        5 => (8830095, 62, 64, Some([504, 512])),
        6 => (8830095, 63, 64, Some([504, 512])),
        7 => (8830095, 0xffff, 1, Some([0, 8])),
        8 => (8830095, 0xfffe, 0, Some([-8, 0])),
        9 => (8830095, 0x8000, 64, Some([504, 512])),
        _ => unreachable!(),
    };
    let image = RomImage::zeroed(0x200000);
    let raw = RomImage::zeroed(0x200000);
    let mut game = Game::new();
    seed(&mut game, &image);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(raw.clone());
    let handle = [0u16, 1, 34, 0xffff][id as usize % 4];
    let offset = usize::from(handle & !1);
    let x = if id == 10 { 300 } else { 128 };
    let y = if id == 11 { 255 } else { 128 };
    put(&mut game, w::layer1_x_pos, 0);
    put(&mut game, w::layer1_y_pos, 0);
    for (field, value) in [
        (a::eproj_id, 0x8000),
        (a::eproj_x_pos, x),
        (a::eproj_y_pos, y),
        (a::eproj_x_subpos, 0xffff),
        (a::eproj_y_subpos, 0xffff),
        (a::eproj_x_vel, 0x0100),
        (a::eproj_y_vel, before),
        (a::eproj_E, function as u16),
        (a::eproj_F, 0x9876),
        (a::eproj_instr_timers, 7),
    ] {
        slot(&mut game, field, handle, value);
    }
    if let Some(positions) = positions {
        let base = crate::rtl::lorom_offset(0xa0cbc7) as usize;
        for (n, position) in positions.into_iter().enumerate() {
            let at = base.checked_add_signed(position).unwrap();
            // Raw increments: +0000:0001 then +0002:0003. Loaded source
            // differs and supplies only the sampled high word stored into F.
            let delta = if n == 0 { 1u32 } else { 0x00020003 };
            let loaded = if n == 0 { 0xabcd7654u32 } else { 0x1234fedc };
            bytes(&raw, at, &delta.to_le_bytes());
            bytes(&image, at, &loaded.to_le_bytes());
        }
    }
    let mut expected = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    write(&mut expected, a::eproj_y_vel + offset, after);
    if positions.is_some() {
        write(&mut expected, a::eproj_y_subpos + offset, 3);
        write(&mut expected, a::eproj_y_pos + offset, y + 3);
        write(&mut expected, a::eproj_F + offset, 0x1234);
        write(&mut expected, a::eproj_x_pos + offset, x + 1);
    } else {
        write(&mut expected, a::eproj_E + offset, 0xbc8f);
    }
    if id == 10 || id == 11 {
        write(&mut expected, a::eproj_id + offset, 0);
    }
    super::super::with_execution(&mut game, |ctx| {
        if id >= 10 {
            super::super::pre_actions::dispatch(8829967, handle, ctx);
        } else {
            super::super::shared_pre::CallNorfairLavaquakeRocksFunc(function, handle, ctx);
        }
    });
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected.as_slice(),
        "qualified lavaquake {id}"
    );
    assert!(!game.g_fail);
    true
}

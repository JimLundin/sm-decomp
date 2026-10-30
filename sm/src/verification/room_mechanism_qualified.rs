//! Independent checks for six former unaligned algorithms and two player wrappers.
//! Native pose callees are deliberately not executed: their existing odd ROM loads
//! remain a qualified interval requiring separate route coverage.
use super::{checks, content::Script};
use crate::game::session_runtime::Runtime;
use crate::rom::RomRecord;
use crate::room_background_support::{ram_bytes, ram_span, ram_word};
use crate::{game_state::Game, rom::RomImage};
use sm_game::enemy_script::{EnemyScript, bank_base};
use std::panic::{AssertUnwindSafe, catch_unwind};
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn rom(image: &RomImage, address: u32, bytes: &[u8]) {
    let at = crate::rtl::lorom_offset(address) as usize;
    for (i, b) in bytes.iter().enumerate() {
        image.write(at + i, *b);
    }
}
fn prepare(game: &mut Game, image: &RomImage) {
    game.ram().fill(0..game.ram().len(), 0);
    ram_word(game.ram(), 0x7a5, 16);
    ram_word(game.ram(), 0x1c87, 34);
    // Force Draw's ordinary queue-full return after its stream/visibility reads.
    ram_word(game.ram(), 0x330, 480);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image.clone());
}
fn script(image: &RomImage) -> EnemyScript {
    EnemyScript::at(image.clone(), crate::rtl::lorom_offset(0x849000) as usize)
}
fn record_call(identity: u32, game: &mut Game) {
    game.ram().copy_in(0x20, &identity.to_le_bytes());
    // A mutation in the callback must not change the already captured identity.
    let image = game.loaded_rom.as_ref().unwrap();
    rom(image, 0x849000, &[0, 0, 0, 0]);
}
fn observe_station(game: &mut Game) {
    assert_eq!(&ram_span::<2>(&game, 0xaf6), &[0x30, 0x12]);
    game.ram().write_byte(0x20, 0xa5);
}
// These verification-only adapters preserve the old qualified mock interval.
// They exercise maintained Script reads/advancement and aggregate publication,
// but deliberately do not claim execution of Varia/Gravity/face-forward callees.
fn captured(position: EnemyScript) -> Script<RomRecord> {
    let EnemyScript::At { image, position } = position else {
        panic!("qualified mock requires readable script");
    };
    let origin = (position as isize - bank_base(0x84)) as u16;
    Script::new(
        RomRecord {
            image,
            base: position,
        },
        origin,
    )
}
fn returned(position: Script<RomRecord>) -> EnemyScript {
    let (record, cursor) = position.into_parts();
    EnemyScript::at(
        record.image,
        record
            .base
            .checked_add_signed(cursor)
            .expect("qualified script position"),
    )
}
fn qualified_call_function(
    game: &mut Game,
    position: EnemyScript,
    call: fn(u32, &mut Game),
) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let position = captured(position);
    runtime.foreign(|game| call(position.load24(0), game));
    returned(position.advance(3))
}
fn qualified_place_on_station(
    game: &mut Game,
    position: EnemyScript,
    call: fn(&mut Game),
) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let position = captured(position);
    runtime.session_mut().player.x = runtime.session().player.x.wrapping_add(8) & 0xfff0;
    runtime.foreign(call);
    returned(position)
}
pub(super) fn run(game: &mut Game) -> bool {
    let saved_ram = ram_bytes(&game);
    let saved_rom = game.loaded_rom.clone();
    let saved_native = game.capture_native_rom_binding();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let image = RomImage::zeroed(0x300000);
        // Grey-door selector reads the deliberately odd native table as two bytes.
        prepare(game, &image);
        ram_word(game.ram(), 0x1e17, 2);
        rom(&image, 0x84be4d, &[0x34, 0x12]);
        let mut expected = ram_bytes(&game);
        put(&mut expected, 0x1cd7, u16::from_ne_bytes([0x34, 0x12]));
        // SAFETY: prepare registered the complete synthetic ROM and initialized
        // the RAM inputs required by this selected command fixture.
        let returned = unsafe { checks::command(0x84be3f, script(&image), 0, game) };
        assert_eq!(returned.offset_from(bank_base(0x84)), 0x9000);
        assert_eq!(&ram_bytes(&game), &expected[..]);
        // Both identities retain the command slot for terrain, but use Selected
        // for coordinates/drawing. The second selection has a separate valid stream.
        for opcode in [0x848b05, 0x848b17] {
            for selected in [0u16, 2] {
                prepare(game, &image);
                ram_word(game.ram(), 0x1e17, 0x123);
                ram_word(game.ram(), 0x1c27, selected);
                ram_word(game.ram(), 0x1c89, 100);
                ram_word(game.ram(), 0xde6e, 0xa020);
                rom(&image, 0x84a020, &1u16.to_ne_bytes());
                rom(&image, 0x84a022, &0x999u16.to_ne_bytes());
                rom(&image, 0x84a024, &0u16.to_ne_bytes());
                let mut expected = ram_bytes(&game);
                for (at, value) in [
                    (0x10024, 0x123),
                    (0x1e69, 0x123),
                    (0x1e67, 1),
                    (0x1e6b, 0),
                    (0xde1c, 1),
                    (0xde6c, 0x1e67),
                    (0x1d27, 0x9000),
                    (0x1c29, if selected == 0 { 1 } else { 2 }),
                    (0x1c2b, if selected == 0 { 1 } else { 3 }),
                ] {
                    put(&mut expected, at, value);
                }
                // SAFETY: prepare registered the complete synthetic ROM and initialized
                // the RAM inputs required by this selected command fixture.
                let returned = unsafe { checks::command(opcode, script(&image), 0, game) };
                assert!(matches!(returned, EnemyScript::Stop));
                assert_eq!(&ram_bytes(&game), &expected[..]);
            }
        }
        // The target pointer overwrites Selected, so the lower row uses slot zero
        // even though the command's captured slot is one.
        prepare(game, &image);
        ram_word(game.ram(), 0x1c87, 98);
        ram_word(game.ram(), 0x1c89, 34);
        ram_word(game.ram(), 0x1e19, 0x1c27);
        ram_word(game.ram(), 0x1c27, 0x44);
        for block in [34usize, 36, 130, 132] {
            game.ram().write_byte(0x10003 + block, 0x55);
        }
        let mut expected = ram_bytes(&game);
        expected[0x1c27..0x1c29].copy_from_slice(&1u16.to_ne_bytes());
        for (block, high, bts) in [
            (34usize, 0xa5, 3),
            (36, 0xa5, 3),
            (130, 0xa5, 3),
            (132, 5, 0),
        ] {
            expected[0x10003 + block] = high;
            expected[0x16402 + block / 2] = bts;
        }
        // SAFETY: prepare registered the complete synthetic ROM and initialized
        // the RAM inputs required by this selected command fixture.
        let returned = unsafe { checks::command(0x84dbb8, script(&image), 2, game) };
        assert_eq!(returned.offset_from(bank_base(0x84)), 0x9000);
        assert_eq!(&ram_bytes(&game), &expected[..]);
        // Both frame tables, both adjacent cells, and Selected != command slot.
        for (opcode, table) in [(0x84e04f, 0x84e05f), (0x84e067, 0x84e077)] {
            for graphic in [0u16, 2] {
                for selected in [0u16, 2] {
                    prepare(game, &image);
                    ram_word(game.ram(), 0xdf0c, graphic);
                    ram_word(game.ram(), 0x1c27, selected);
                    ram_word(game.ram(), 0x1c89, 100);
                    ram_word(game.ram(), 0xde6e, 0xa020);
                    for (address, tile) in [(0xa000u16, 0x321u16), (0xa010, 0x456), (0xa020, 0x999)]
                    {
                        rom(&image, 0x840000 + u32::from(address), &1u16.to_ne_bytes());
                        rom(&image, 0x840002 + u32::from(address), &tile.to_ne_bytes());
                        rom(&image, 0x840004 + u32::from(address), &0u16.to_ne_bytes());
                    }
                    rom(&image, table, &0xa000u16.to_ne_bytes());
                    rom(&image, table + 2, &0xa010u16.to_ne_bytes());
                    let (target, tile) = if graphic == 0 {
                        (0xa000, 0x321)
                    } else {
                        (0xa010, 0x456)
                    };
                    let mut expected = ram_bytes(&game);
                    for (at, value) in [
                        (0x10024, tile),
                        (0xde1c, 4),
                        (0xde6c, target),
                        (0x1d27, 0x9000),
                        (0x1c29, if selected == 0 { 1 } else { 2 }),
                        (0x1c2b, if selected == 0 { 1 } else { 3 }),
                    ] {
                        put(&mut expected, at, value);
                    }
                    // SAFETY: prepare registered the complete synthetic ROM and initialized
                    // the RAM inputs required by this selected command fixture.
                    let returned = unsafe { checks::command(opcode, script(&image), 0, game) };
                    assert!(matches!(returned, EnemyScript::Stop));
                    assert_eq!(&ram_bytes(&game), &expected[..]);
                }
            }
        }
        // Exercise content extraction with a verification-only recording boundary. Varia and
        // Gravity identity extraction includes and masks the adjacent fourth byte.
        for identity in [0x91d4e4u32, 0x91d5ba] {
            prepare(game, &image);
            let bytes = (identity | 0xa5000000).to_ne_bytes();
            rom(&image, 0x849000, &bytes);
            let returned = qualified_call_function(game, script(&image), record_call);
            assert_eq!(&ram_span::<4>(&game, 0x20), &identity.to_le_bytes());
            assert_eq!(returned.offset_from(bank_base(0x84)), 0x9003);
        }
        // The fourth byte must be readable even though it is masked out.
        prepare(game, &image);
        let short = RomImage::zeroed(3);
        let failed = catch_unwind(AssertUnwindSafe(|| {
            qualified_call_function(game, EnemyScript::at(short, 0), record_call);
        }));
        assert!(failed.is_err());
        assert_eq!(&ram_span::<4>(&game, 0x20), &[0, 0, 0, 0]);
        prepare(game, &image);
        ram_word(game.ram(), 0xaf6, 0x1229);
        let returned = qualified_place_on_station(game, script(&image), observe_station);
        assert_eq!(returned.offset_from(bank_base(0x84)), 0x9000);
        assert_eq!(game.ram().read_byte(0x20), 0xa5);
    }));
    game.ram().copy_in(0, &saved_ram);
    game.loaded_rom = saved_rom;
    game.restore_native_rom_binding(saved_native);
    result.is_ok()
}

//! Additional actual-old rendering observations. Expectations describe emitted
//! tiles and packed transfers independently of the production renderer.
use super::{put, rom_word};
use crate::game::{enemy_script::EnemyScript, room_mechanisms as mechanisms};
use crate::{game_state::Game, rom::RomImage};

fn word(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.g_ram[at], game.g_ram[at + 1]])
}

fn prepare(game: &mut Game, image: &RomImage) {
    game.g_ram.fill(0);
    put(game, 0x7a5, 16);
    put(game, 0x7a9, 16);
    put(game, 0x1c87, 34);
    put(game, 0xde6c, 0xa000);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image.clone());
    for (block, tiles) in [
        (1, [0x1101, 0x1102, 0x1103, 0x1104]),
        (2, [0x2201, 0x2202, 0x2203, 0x2204]),
    ] {
        for (corner, tile) in tiles.into_iter().enumerate() {
            put(game, 0xa000 + block * 8 + corner * 2, tile);
        }
    }
}

fn stream(image: &RomImage, address: u32, command: u16, blocks: &[u16]) {
    rom_word(image, address, command);
    for (i, block) in blocks.iter().enumerate() {
        rom_word(image, address + 2 + i as u32 * 2, *block);
    }
    rom_word(image, address + 2 + blocks.len() as u32 * 2, 0);
}

fn tiles(game: &Game, at: usize, expected: &[u16]) {
    for (i, expected) in expected.iter().enumerate() {
        assert_eq!(word(game, at + i * 2), *expected, "tile {i} at {at:x}");
    }
}

fn transfer(game: &Game, index: usize, size: u16, source: u16, destination: u16) {
    let at = 0xd0 + index * 7;
    // Queue words have native serialization, independently asserted bytewise.
    assert_eq!(&game.g_ram[at..at + 2], &size.to_ne_bytes());
    assert_eq!(&game.g_ram[at + 2..at + 4], &source.to_ne_bytes());
    assert_eq!(game.g_ram[at + 4], 0x7e);
    assert_eq!(&game.g_ram[at + 5..at + 7], &destination.to_ne_bytes());
}

fn pair(game: &Game, destination: u16) {
    assert_eq!(word(game, 0x1c25), 8);
    assert_eq!(word(game, 0x330), 14);
    tiles(game, 0xc6c8, &[0x2201, 0x2202, 0x2203, 0x2204]);
    transfer(game, 0, 4, 0xc6c8, destination);
    transfer(game, 1, 4, 0xc6cc, destination + 32);
}

pub(super) unsafe fn run(
    id: u32,
    game: &mut Game,
    image: &RomImage,
    trace: &mut Vec<u8>,
) -> (u8, u16) {
    prepare(game, image);
    match id {
        1022 => {
            // Terrain captures loaded content; rendering reads the independent
            // native binding. Both streams are valid and deliberately disagree.
            stream(image, 0x84a000, 1, &[1]);
            let native = RomImage::zeroed(0x200000);
            stream(&native, 0x84a000, 1, &[2]);
            game.register_native_rom(native);
            mechanisms::checks::process_draw(0, game);
            mechanisms::calculate_coordinates(0, game);
            unsafe { mechanisms::checks::draw(0, game) };
            assert_eq!(word(game, 0x10024), 1);
            pair(game, 0x5042);
        }
        1023 => {
            stream(image, 0x84a000, 1, &[1]);
            let replacement = RomImage::zeroed(0x200000);
            stream(&replacement, 0x84a000, 1, &[2]);
            super::callback_edges::begin_native(replacement);
            mechanisms::calculate_coordinates(0, game);
            unsafe { mechanisms::checks::draw(0, game) };
            let addresses = super::callback_edges::finish_native();
            assert_eq!(&addresses, &[0xa000, 0xa002, 0xa004]);
            for address in addresses {
                trace.extend_from_slice(&address.to_le_bytes());
            }
            pair(game, 0x5042);
        }
        1024 => {
            // A broad draw cursor sends this block into its own tile definition.
            // Horizontal corner order is 0,2,1,3: the first two writes replace
            // the source words that the last two reads must subsequently see.
            stream(image, 0x84a000, 1, &[0]);
            for (i, value) in [0x1111, 0x2222, 0x3333, 0x4444, 0x5555]
                .into_iter()
                .enumerate()
            {
                put(game, 0xa000 + i * 2, value);
            }
            put(game, 0x1c25, 0xd93a); // C6C8 + D93A wraps to A002.
            mechanisms::calculate_coordinates(0, game);
            unsafe { mechanisms::checks::draw(0, game) };
            tiles(game, 0xa000, &[0x1111, 0x1111, 0x1111, 0x3333, 0x3333]);
            assert_eq!(word(game, 0x1c25), 0xd942);
            assert_eq!(word(game, 0x330), 14);
            transfer(game, 0, 4, 0xa002, 0x5042);
            transfer(game, 1, 4, 0xa006, 0x5062);
        }
        1025..=1028 => {
            // Commands retain slot0 for terrain but draw the live selected slot1.
            // Unlike1019, queue admission succeeds and all four tiles are emitted.
            put(game, 0x1c27, 2);
            put(game, 0x1c89, 100);
            put(game, 0xde6e, 0xa020);
            stream(image, 0x84a020, 1, &[2]);
            let opcode = match id {
                1025 => 0x848b05,
                1026 => 0x848b17,
                1027 => 0x84e04f,
                _ => 0x84e067,
            };
            if id <= 1026 {
                put(game, 0x1e17, 0x123);
            } else {
                put(game, 0xdf0c, 2);
                stream(image, 0x84a010, 1, &[1]);
                rom_word(image, if id == 1027 { 0x84e061 } else { 0x84e079 }, 0xa010);
            }
            let script =
                EnemyScript::at(image.clone(), crate::rtl::lorom_offset(0x849000) as usize);
            let result = unsafe { mechanisms::checks::command(opcode, script, 0, game) };
            assert!(matches!(result, EnemyScript::Stop));
            assert_eq!(word(game, 0x10024), if id <= 1026 { 0x123 } else { 1 });
            assert_eq!(word(game, 0xde1c), if id <= 1026 { 1 } else { 4 });
            assert_eq!(word(game, 0x1d27), 0x9000);
            assert_eq!(word(game, 0x1c29), 2);
            assert_eq!(word(game, 0x1c2b), 3);
            pair(game, 0x50c4);
        }
        1029 => {
            // Terrain's stream cursor is linear, unlike the wrapping draw cursor.
            put(game, 0xde6c, 0xfffe);
            let start = crate::game::enemy_script::bank_base(0x84) as usize + 0xfffe;
            for (i, value) in [1u16, 0x345, 0].into_iter().enumerate() {
                for (lane, byte) in value.to_ne_bytes().into_iter().enumerate() {
                    image.write(start + i * 2 + lane, byte);
                }
            }
            // A wrapping mutant reads these different words instead.
            let wrapped = crate::game::enemy_script::bank_base(0x84) as usize;
            image.write(wrapped, 0x89);
            image.write(wrapped + 1, 0x07);
            mechanisms::checks::process_draw(0, game);
            assert_eq!(word(game, 0x10024), 0x345);
        }
        1030 => {
            // Horizontal row crosses the screen's sixteen-block page boundary.
            stream(image, 0x84a000, 2, &[1, 2]);
            put(game, 0x1c87, 30); // block x15, y0.
            mechanisms::calculate_coordinates(0, game);
            unsafe { mechanisms::checks::draw(0, game) };
            assert_eq!(word(game, 0x1c25), 16);
            assert_eq!(word(game, 0x330), 28);
            tiles(
                game,
                0xc6c8,
                &[
                    0x1101, 0x1102, 0x2201, 0x2202, 0x1103, 0x1104, 0x2203, 0x2204,
                ],
            );
            transfer(game, 0, 4, 0xc6c8, 0x501e);
            transfer(game, 1, 4, 0xc6cc, 0x5400);
            transfer(game, 2, 4, 0xc6d0, 0x503e);
            transfer(game, 3, 4, 0xc6d4, 0x5420);
        }
        1031 => {
            stream(image, 0x84a000, 0x8002, &[1, 2]);
            mechanisms::calculate_coordinates(0, game);
            unsafe { mechanisms::checks::draw(0, game) };
            assert_eq!(word(game, 0x1c25), 16);
            assert_eq!(word(game, 0x330), 14);
            tiles(
                game,
                0xc6c8,
                &[
                    0x1101, 0x1103, 0x2201, 0x2203, 0x1102, 0x1104, 0x2202, 0x2204,
                ],
            );
            transfer(game, 0, 8, 0xc6c8, 0xd042);
            transfer(game, 1, 8, 0xc6d0, 0xd043);
        }
        1032 => {
            // Low-RAM terrain programs also use a wide cursor. Zero-length
            // records reach the terrain array without writing it. The final
            // vertical record has width zero: first destination is the second
            // source, so the second read must see the first write (0345).
            // A snapshot of the stream incorrectly leaves0789 at that target.
            for at in (0x1ffe..0x1001e).step_by(4) {
                put(game, at, 0);
                put(game, at + 2, 1);
            }
            put(game, 0x1001e, 0x8002);
            put(game, 0x10020, 0x345);
            put(game, 0x10022, 0x789);
            put(game, 0x10024, 0);
            put(game, 0x7a5, 0);
            put(game, 0x1c87, 30);
            put(game, 0xde6c, 0x1ffe);
            mechanisms::checks::process_draw(0, game);
            assert_eq!(word(game, 0x10022), 0x345);
            assert_eq!(word(game, 0x10024), 0);
        }
        _ => unreachable!("unassigned rendering observation {id}"),
    }
    (6, id as u16)
}

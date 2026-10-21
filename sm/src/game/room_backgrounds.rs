//! Room background commands, with the original initial-load and door DMA effects.
use crate::game::decompression::decompress_to_ram;
use crate::game::prelude::*;
use crate::game::sm_80::CopyToVramNow;

const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;

#[derive(Clone, Copy)]
pub(crate) enum BackgroundLoad {
    Initial,
    Door,
}

impl BackgroundLoad {
    pub(crate) unsafe fn load(self, game: &mut Game) {
        self.clear_fx(game);
        match self {
            Self::Initial => {
                if word!(game.fx_tilemap_ptr) != 0 {
                    crate::rtl::register_writes! { game;
                        word(A1T1L as u16, word!(game.fx_tilemap_ptr)); word(VMADDL as u16, 0x5be0);
                        word(DMAP1 as u16, 0x1801); word(A1B1 as u16, 0x8a);
                        word(DAS1L as u16, 0x840); byte(VMAIN as u16, 0x80);
                        byte(MDMAEN as u16, 2);
                    }
                }
            }
            Self::Door => {
                let fx_tilemap_ptr = u16::from_le_bytes([game.g_ram[0x1964], game.g_ram[0x1965]]);
                if fx_tilemap_ptr != 0 {
                    CopyToVramNow(0x5be0, 0x8a0000 | u32::from(fx_tilemap_ptr), 2112, game);
                }
            }
        }
        let mut cursor = match self {
            Self::Initial => {
                (*(RomPtr_8F(word!(game.roomdefroomstate_ptr), game) as *const RoomDefRoomstate))
                    .bg_data_ptr
            }
            Self::Door => game.read_rom_word(
                0x8f0000 | u32::from(game.roomdefroomstate_ptr()),
                core::mem::offset_of!(RoomDefRoomstate, bg_data_ptr),
            ),
        };
        if cursor & 0x8000 != 0 {
            while cursor != 0 {
                let command = match self {
                    Self::Initial => *(RomPtr_8F(cursor, game) as *const u16),
                    Self::Door => game.read_rom_word(0x8f0000 | u32::from(cursor), 0),
                };
                cursor = self.execute(command, cursor.wrapping_add(2), game);
            }
        }
    }

    unsafe fn execute(self, command: u16, mut cursor: u16, game: &mut Game) -> u16 {
        // Odd commands alias the preceding even command, as in the original tables.
        match command >> 1 {
            0 => return 0,
            1 => return self.transfer(cursor, game),
            2 => {
                match self {
                    Self::Initial => {
                        let p = RomPtr_8F(cursor, game);
                        decompress_to_ram(
                            Load24(p as *mut LongPtr),
                            *(p.offset(3) as *mut u16) as usize,
                            game,
                        );
                    }
                    Self::Door => {
                        let address = 0x8f0000 | u32::from(cursor);
                        let source = u32::from(game.read_rom_word(address, 0))
                            | (u32::from(game.read_rom_byte(address, 2)) << 16);
                        let destination = usize::from(game.read_rom_word(address, 3));
                        decompress_to_ram(source, destination, game);
                    }
                }
                return cursor.wrapping_add(5);
            }
            3 => self.clear_fx(game),
            4 => {
                cursor = self.transfer(cursor, game);
                *ram_ptr!(game; 0x5e) = 2;
            }
            5 => self.clear_bg2(game),
            6 => match self {
                // Initial loading deliberately uses the ordinary BG2 clear here.
                Self::Initial => self.clear_bg2(game),
                Self::Door => {
                    word_transfers!(game, descending, 1024; xray(1023) <- value(824), xray(2047) <- value(824));
                    CopyToVramNow(0x4000, 0x7e4000, 0x1000, game);
                    CopyToVramNow(0x4800, 0x7e4000, 0x1000, game);
                }
            },
            7 => {
                let door = (RomPtr_8F(cursor, game) as *const u16).read_unaligned();
                return if word!(game.door_def_ptr) == door {
                    self.transfer(cursor.wrapping_add(2), game)
                } else {
                    cursor.wrapping_add(9)
                };
            }
            _ => panic!("invalid room background command: {command:#x}"),
        }
        cursor
    }

    unsafe fn transfer(self, cursor: u16, game: &mut Game) -> u16 {
        match self {
            Self::Initial => {
                let p = RomPtr_8F(cursor, game) as *mut u16;
                crate::rtl::register_writes! { game;
                    word(VMADDL as u16, *((p as *mut u8).offset(3) as *mut u16));
                    word(DMAP1 as u16, 0x1801); word(A1T1L as u16, *p);
                    word(A1B1 as u16, *p.offset(1));
                    word(DAS1L as u16, *((p as *mut u8).offset(5) as *mut u16));
                    byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
                }
            }
            Self::Door => {
                let address = 0x8f0000 | u32::from(cursor);
                let destination = game.read_rom_word(address, 3);
                let source = u32::from(game.read_rom_word(address, 0))
                    | (u32::from(game.read_rom_byte(address, 2)) << 16);
                let size = game.read_rom_word(address, 5);
                CopyToVramNow(destination, source, size, game);
            }
        }
        cursor.wrapping_add(7)
    }

    unsafe fn clear_fx(self, game: &mut Game) {
        match self {
            Self::Initial => clear_tilemap_with_dma(0x5880, 0xa2f7, 0xa2f8, 0x780, game),
            Self::Door => {
                word_transfers!(game, descending, 960;
                    little(0x477e) <- value(6222), little(0x4efe) <- value(6222),
                );
                CopyToVramNow(0x5880, 0x7e4000, 0xf00, game);
            }
        }
    }

    pub(crate) unsafe fn clear_bg2(self, game: &mut Game) {
        match self {
            Self::Initial => clear_tilemap_with_dma(0x4800, 0xa29a, 0xa29a, 0x800, game),
            Self::Door => {
                word_transfers!(game, descending, 1024;
                    little(0x47fe) <- value(824), little(0x4ffe) <- value(824),
                );
                CopyToVramNow(0x4800, 0x7e4000, 0x1000, game);
            }
        }
    }
}

unsafe fn clear_tilemap_with_dma(
    destination: u16,
    low: u16,
    high: u16,
    size: u16,
    game: &mut Game,
) {
    crate::rtl::register_writes! { game;
        word(VMADDL as u16, destination); word(DMAP1 as u16, 0x1808);
        word(A1T1L as u16, low); word(A1B1 as u16, 0x80);
        word(DAS1L as u16, size); byte(VMAIN as u16, 0);
        byte(MDMAEN as u16, 2); word(VMADDL as u16, destination);
        word(DMAP1 as u16, 0x1908); word(A1T1L as u16, high);
        word(A1B1 as u16, 0x80); word(DAS1L as u16, size);
        byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
    }
}

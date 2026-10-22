//! Room background commands, with the original initial-load and door DMA effects.
use crate::game::decompression::decompress_to_ram;
use crate::game::prelude::*;
use crate::game::sm_80::CopyToVramNow;

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
                if u16::from_le_bytes([game.g_ram[0x1964], game.g_ram[0x1965]]) != 0 {
                    crate::rtl::register_writes! { game;
                        word(A1T1L as u16, u16::from_le_bytes([game.g_ram[0x1964], game.g_ram[0x1965]])); word(VMADDL as u16, 0x5be0);
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
        let mut cursor = self
            .record(game.roomdefroomstate_ptr(), game)
            .word(core::mem::offset_of!(RoomDefRoomstate, bg_data_ptr));
        if cursor & 0x8000 != 0 {
            while cursor != 0 {
                let command = self.record(cursor, game).word(0);
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
                let data = self.record(cursor, game);
                decompress_to_ram(data.source(), usize::from(data.word(3)), game);
                return cursor.wrapping_add(5);
            }
            3 => self.clear_fx(game),
            4 => {
                cursor = self.transfer(cursor, game);
                game.g_ram[0x5e] = 2;
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
                let door = Self::Initial.record(cursor, game).word(0);
                return if game.door_def_ptr() == door {
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
        let data = self.record(cursor, game);
        match self {
            Self::Initial => {
                crate::rtl::register_writes! { game;
                    word(VMADDL as u16, data.word(3));
                    word(DMAP1 as u16, 0x1801); word(A1T1L as u16, data.word(0));
                    word(A1B1 as u16, data.word(2));
                    word(DAS1L as u16, data.word(5));
                    byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
                }
            }
            Self::Door => {
                let destination = data.word(3);
                let source = data.source();
                let size = data.word(5);
                CopyToVramNow(destination, source, size, game);
            }
        }
        cursor.wrapping_add(7)
    }

    unsafe fn record(self, cursor: u16, game: &mut Game) -> BackgroundRecord {
        if cursor & 0x8000 == 0 {
            match self {
                Self::Initial => {
                    printf(
                        c"RomPtr - Invalid access 0x%x!\n".as_ptr(),
                        0x8f0000 | u32::from(cursor),
                    );
                    game.g_fail = true;
                }
                Self::Door => panic!("ROM address must select a LoROM window"),
            }
        }
        BackgroundRecord {
            image: game
                .loaded_rom
                .as_ref()
                .expect("ROM must be loaded")
                .clone(),
            base: crate::rtl::lorom_offset(0x8f0000 | u32::from(cursor)) as usize,
            mode: self,
        }
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

// Keep the allocation and read live bytes between effects, as the old record pointer did.
struct BackgroundRecord {
    image: crate::rom::RomImage,
    base: usize,
    mode: BackgroundLoad,
}

#[forbid(unsafe_code)]
impl BackgroundRecord {
    fn bytes<const N: usize>(&self, offset: usize) -> [u8; N] {
        let start = self.base + offset;
        assert!(
            start + N <= self.image.len(),
            "ROM read outside loaded allocation"
        );
        std::array::from_fn(|i| self.image.read(start + i))
    }

    fn word(&self, offset: usize) -> u16 {
        let bytes = self.bytes(offset);
        match self.mode {
            BackgroundLoad::Initial => u16::from_ne_bytes(bytes),
            BackgroundLoad::Door => u16::from_le_bytes(bytes),
        }
    }

    fn source(&self) -> u32 {
        match self.mode {
            BackgroundLoad::Initial => u32::from_ne_bytes(self.bytes(0)) & 0xffffff,
            BackgroundLoad::Door => {
                u32::from(self.word(0)) | (u32::from(self.bytes::<1>(2)[0]) << 16)
            }
        }
    }
}

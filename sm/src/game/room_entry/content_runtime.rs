//! Live legacy/prepared content acquisition through the shared Session interval.
//! Captured values retain allocations; diagnostics precede replacement captures.
use super::{
    content::*,
    effects::{AssetRead, AssetSlot},
};
use crate::{
    game::session_runtime::{Runtime, invalid_window, native_rom_diagnostic},
    rom::RomRecord,
};

impl Runtime<'_> {
    fn room_pointer_diagnostic(&mut self, bank: u8, address: u16) {
        if address & 0x8000 == 0 {
            self.foreign(|game| {
                // Keep the actual RomPtr observer and native binding acquisition.
                native_rom_diagnostic((u32::from(bank) << 16) | u32::from(address), game);
                let _ = game.legacy_native_rom_ptr();
            });
        }
    }
    fn contact_word(&self, bank: u8, address: u16) -> u16 {
        let image = self.loaded_image().expect("ROM must be loaded");
        let at = crate::rtl::lorom_offset((u32::from(bank) << 16) | 0x8000 | u32::from(address))
            as usize;
        assert!(at + 2 <= image.len(), "ROM read outside loaded allocation");
        u16::from_le_bytes([image.read(at), image.read(at + 1)])
    }
}
impl EntryContent for Runtime<'_> {
    fn contact_bts(&mut self) -> u8 {
        let block = self.session().room.collision_cursor;
        self.byte(0x16402 + usize::from(block))
    }
    fn legacy_word(&mut self, bank: u8, address: u16) -> u16 {
        self.room_pointer_diagnostic(bank, address);
        self.contact_word(bank, address)
    }
    fn load_station_index(&self) -> u16 {
        self.word(0x78b)
    }
    fn legacy_station(&mut self) -> StationCapture {
        // Table allocation is captured before evaluating its area index.
        let image = self.loaded_image().expect("ROM must be loaded");
        let base = image.table_word(
            crate::rtl::lorom_offset(0x80c4b5) as usize,
            self.session().room.area as isize,
        );
        let address = (i32::from(base) + 14 * i32::from(self.word(0x78b))) as u16;
        self.native_room_capture(0x80, address)
    }
    /// Normal room_record policy: loaded image, native words, room observer.
    fn room_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        let record = if address & 0x8000 == 0 {
            // Retain the real rom_record boundary, including its cfg observers.
            self.foreign(|game| unsafe { crate::rtl::rom_record(bank, address, game) })
        } else {
            RomRecord {
                image: self.loaded_image().expect("ROM must be loaded"),
                base: crate::rtl::lorom_offset((u32::from(bank) << 16) | u32::from(address))
                    as usize,
            }
        };
        Capture::Legacy {
            image: record.image,
            base: record.base,
            access: RecordAccess::Native,
            world: self.session().world.clone(),
        }
    }
    /// RomPtr policy used by load stations and their final header acquisition.
    fn native_room_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        self.room_pointer_diagnostic(bank, address);
        Capture::Legacy {
            image: self.native_image(),
            base: crate::rtl::lorom_offset((u32::from(bank) << 16) | u32::from(address)) as usize,
            access: RecordAccess::Native,
            world: self.session().world.clone(),
        }
    }
    /// PLM loaded_record policy has no room_record or RomPtr observer hook.
    fn closing_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        let absolute = (u32::from(bank) << 16) | u32::from(address);
        if address & 0x8000 == 0 {
            self.foreign(|game| invalid_window(game, absolute));
        }
        Capture::Legacy {
            image: self.loaded_image().expect("ROM must be loaded"),
            base: crate::rtl::lorom_offset(absolute) as usize,
            access: RecordAccess::Native,
            world: self.session().world.clone(),
        }
    }
    fn background_capture<P>(&mut self, address: u16, mode: LoadMode) -> Capture<P> {
        let absolute = 0x8f0000 | u32::from(address);
        if address & 0x8000 == 0 {
            match mode {
                LoadMode::Initial => self.foreign(|game| {
                    invalid_window(game, absolute);
                    #[cfg(sm_verify_dll)]
                    crate::room_background_observer::diagnostic(absolute, game);
                }),
                LoadMode::Door => panic!("ROM address must select a LoROM window"),
            }
        }
        Capture::Legacy {
            image: self.loaded_image().expect("ROM must be loaded"),
            base: crate::rtl::lorom_offset(absolute) as usize,
            access: match mode {
                LoadMode::Initial => RecordAccess::Native,
                LoadMode::Door => RecordAccess::LittleEndian,
            },
            world: self.session().world.clone(),
        }
    }
    fn invalid_selector(&mut self, _address: u32) -> Option<SelectorCursor> {
        let value = self.foreign(|game| crate::rtl::report_unreachable(game)) as u16;
        (value != 0).then_some(SelectorCursor::Legacy(value))
    }
    fn legacy_tileset(&mut self, index: u8) -> TilesetCapture {
        let image = self.loaded_image().expect("ROM must be loaded");
        let at = crate::rtl::lorom_offset(0x8fe7a7) as usize + usize::from(index) * 2;
        assert!(at + 2 <= image.len(), "ROM read outside loaded allocation");
        let address = u16::from_le_bytes([image.read(at), image.read(at + 1)]);
        self.room_capture(0x8f, address)
    }
    fn asset_bytes(&self, slot: AssetSlot, read: AssetRead) -> [u8; 3] {
        let base = match slot {
            AssetSlot::Level => 0x7bd,
            AssetSlot::TileTable => 0x7c0,
            AssetSlot::Tiles => 0x7c3,
            AssetSlot::Palette => 0x7c6,
        };
        match read {
            AssetRead::ThreeLittleEndian => {
                [self.byte(base), self.byte(base + 1), self.byte(base + 2)]
            }
            AssetRead::FourNativeMasked => {
                let value =
                    u32::from_ne_bytes(std::array::from_fn(|i| self.byte(base + i))) & 0xffffff;
                let bytes = value.to_le_bytes();
                [bytes[0], bytes[1], bytes[2]]
            }
        }
    }
}

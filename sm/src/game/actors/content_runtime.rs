//! The actor content reads run in `sm_game::actors::content_records` over the Runtime's
//! cartridge images and records.
use crate::game::session_runtime::Runtime;
use crate::rom::{RomImage, RomRecord};
use sm_game::actors::content_records::ContentSource;

impl ContentSource for Runtime<'_> {
    fn content_image(&self) -> RomImage {
        self.loaded_image().expect("ROM must be loaded")
    }
    fn native_content_image(&self) -> RomImage {
        self.native_image()
    }
    fn content_record(&mut self, bank: u8, address: u16) -> RomRecord {
        self.actor_legacy_record(bank, address)
    }
}

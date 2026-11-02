//! The finite current-owner contract used by PreparedActorState and Runtime.
//! `current_*` operations below remain measured alias dependencies.
#![forbid(unsafe_code)]
use super::bindings::VisualTable;
use crate::display::preparation::{Access, Field, Store};
use crate::fx_domain::{Controls, Sources};
use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::{DisplayError, WordOrder};
use crate::session_access::SessionAccess;
pub trait Context:
    Sized
    + SessionAccess
    + crate::hdma_objects::HdmaTable
    + crate::actors::boss_hdma_owned::Host
    + crate::actors::access::ActorAccess
    + crate::cartridge::NativeDiagnostics
    + crate::cartridge::LoadedImage
    + Controls
    + Sources
    + Access
    + crate::display_scroll::Access
    + crate::progress::FlagHost
    + crate::records::Records
    + crate::fx_lighting::Access
    + crate::effects::SoundEffects
    + crate::enemy_projectiles::host::Host
    + crate::room_mechanisms::host::Host
    + crate::room_mechanisms::host::Effects
    + crate::room_scrolls::Host
    + crate::player_projectiles::host::Host
    + crate::player_transitions::effects::Host
    + crate::player_movement::effects::Host
    + crate::visual_host::Content<Record = sm_data::cartridge::RomRecord>
    + crate::presentation_workspace::Access
    + crate::presentation_workspace::QueueAccess
{
    fn current_byte(&self, at: usize) -> u8;
    fn set_current_byte(&mut self, at: usize, value: u8);
    fn write_current_slice(&mut self, at: usize, bytes: &[u8]);
    fn current_word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.current_byte(at), self.current_byte(at + 1)])
    }
    fn current_native_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes([self.current_byte(at), self.current_byte(at + 1)])
    }
    fn hdma_slice_le(&mut self, at: usize, value: u16) {
        self.write_current_slice(at, &value.to_le_bytes())
    }
    fn hdma_slice_native(&mut self, at: usize, value: u16) {
        self.write_current_slice(at, &value.to_ne_bytes())
    }
    fn hdma_workspace_word(
        &mut self,
        at: usize,
        value: u16,
        store: crate::player_projectiles::access::Store,
    ) {
        use crate::player_projectiles::access::Store;
        match store {
            Store::PrefixLe => {
                let [low, high] = value.to_le_bytes();
                self.set_current_byte(at, low);
                self.set_current_byte(at + 1, high)
            }
            Store::SliceLe => self.hdma_slice_le(at, value),
            Store::SliceNative => self.hdma_slice_native(at, value),
        }
    }
    fn hdma_fixed_colors(&mut self, colors: [u8; 3]) {
        for (field, value) in [Field::FixedColor0, Field::FixedColor1, Field::FixedColor2]
            .into_iter()
            .zip(colors)
        {
            self.store_display_control_byte(field, value, Store::Owned);
        }
    }
    fn calculate_xray_window(&mut self, window: crate::beam_windows::XrayWindow) {
        super::beam::calculate_xray_window(window, self)
    }
    fn hdma_loaded_word(&self, address: u32, offset: usize) -> u16 {
        assert!(address & 0x8000 != 0);
        let image = crate::cartridge::LoadedImage::loaded_image(self);
        let first = usize::try_from(sm_data::cartridge::lorom_offset(address))
            .expect("HDMA loaded address")
            .checked_add(offset)
            .expect("HDMA loaded offset");
        let end = first.checked_add(2).expect("HDMA loaded extent");
        assert!(end <= image.len());
        u16::from_le_bytes([image.read(first), image.read(first + 1)])
    }
    fn legacy_native_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        crate::cartridge::native_record(bank, address, self)
    }
    fn hdma_music(&mut self) -> Result<(), DisplayError>;
    fn hdma_register(&mut self, address: u16, value: u8);
    fn hdma_readback(
        &mut self,
        register: usize,
        offset: u16,
        destination: u16,
        size: u16,
        retain: bool,
    );
    fn hdma_visual_word(&mut self, table: VisualTable, index: isize) -> u16;
    fn hdma_overlay_origin_assertion(&mut self);
    fn hdma_unreachable(&mut self);
    fn hdma_rainbow(&mut self, initialize: bool);
    fn hdma_player_command(&mut self, command: u16);
    fn hdma_title_gradient(&mut self, backdrop: bool, advance: bool) {
        if backdrop {
            let zoom = self.current_word(crate::ram::words::MODE7_ZOOM);
            let mut cursor = self.hdma_visual_word(
                VisualTable::TitleGradient,
                isize::try_from((zoom & 0xf0) >> 4).expect("title gradient"),
            );
            for offset in 0usize.. {
                let value =
                    crate::menus::menu_tilemap::native_record(0x8c0000 | u32::from(cursor), self)
                        .word(0);
                self.store_source_word(LiveOwner::HdmaTable1, 2 * offset, WordOrder::Little, value);
                if value == 0 {
                    break;
                }
                cursor = cursor.wrapping_add(2);
            }
        }
        if advance && self.session().flow.cinematic_function == 0x9b68 {
            super::effects::Effects::advance_hdma_program(self);
        }
    }
}

//! Compatibility sources and live aliases for the owned beta interaction.
use crate::{
    game::{
        player_transitions::state::FrameGammaMode,
        player_update::{
            content::Content,
            effects::{Effects, GammaAction},
        },
        session_runtime::Runtime,
        visual_host::{self, Source},
    },
    rom::RomRecord,
};

fn native_record(runtime: &mut Runtime<'_>, address: u32) -> RomRecord {
    if address & 0x8000 == 0 {
        runtime.foreign(|game| unsafe {
            crate::rtl::RomPtr(address, game);
        });
    }
    visual_host::Content::table(runtime, Source::Native, address)
}

impl Content for Runtime<'_> {
    type BossCoordinates = u16;
    fn area_exploration_mask(&mut self, area: u16) -> RomRecord {
        let table = visual_host::Content::table(self, Source::Loaded, 0x829717);
        let cursor = table.word(usize::from(area) * 2);
        native_record(self, 0x820000 | u32::from(cursor))
    }
    fn area_map_tiles(&mut self, area: u16) -> RomRecord {
        let table = visual_host::Content::table(self, Source::Native, 0x82964a);
        // Load24 reads the fourth adjacent byte before masking it away.
        let address = table.load24(usize::from(area) * 3);
        native_record(self, address)
    }
    fn boss_map_id(&self, index: usize) -> u16 {
        visual_host::Content::table(self, Source::Native, 0x90a83a).word(index * 4)
    }
    fn boss_coordinates(&self, index: usize) -> u16 {
        visual_host::Content::table(self, Source::Native, 0x90a83a).word(index * 4 + 2)
    }
    fn boss_coordinate(&mut self, coordinates: u16, index: usize) -> RomRecord {
        let cursor = coordinates.wrapping_add((index * 4) as u16);
        native_record(self, 0x900000 | u32::from(cursor))
    }
}

fn exploration_address(offset: isize) -> usize {
    0x7f7usize
        .checked_add_signed(offset)
        .expect("exploration offset overflow")
}
impl Effects for Runtime<'_> {
    fn imported_exploration_byte(&self, offset: isize) -> u8 {
        self.byte(exploration_address(offset))
    }
    fn set_imported_exploration_byte(&mut self, offset: isize, value: u8) {
        self.set_byte(exploration_address(offset), value);
    }
    fn area_map_available(&self) -> bool {
        self.word(0x789) != 0
    }
    fn minimap_blink(&self) -> u8 {
        self.byte(0x5b5)
    }
    fn minimap_boss_id(&self) -> u16 {
        self.word(0x179c)
    }
    fn gamma_action(&self) -> GammaAction {
        match self.session().player.transitions.frame_gamma {
            FrameGammaMode::Empty | FrameGammaMode::Disabled => GammaAction::Inactive,
            mode => GammaAction::Legacy(mode.to_raw()),
        }
    }
    fn run_gamma(&mut self, selector: u16) {
        self.foreign(|game| unsafe {
            crate::game::sm_90::CallFrameHandlerGamma(0x900000 | u32::from(selector), game);
        });
    }
}

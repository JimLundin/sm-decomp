//! Botwoon runs in `sm_game::actors::botwoon`; its tables, flags, PLMs and
//! history hooks come from here, and its death signals run in the Runtime lease.
#![forbid(unsafe_code)]
use crate::game::actors::api;
use crate::game::enemy_projectiles::{self, scatter_drop};
use crate::game::gameplay_flag_runtime::{SetBossBitForCurArea, boss_byte_at};
use crate::game::imported::Imported;
use crate::game::session_runtime::{Runtime, actor_b3_pre_instruction};
use crate::game_state::Game;
use crate::rom::RomImage;
use crate::rtl::rom_table_word;
use sm_data::botwoon_holes::{ADDRESS as HOLES, BYTES as HOLE_BYTES, BotwoonHoles, Hole};
use sm_data::botwoon_speeds::{
    ADDRESS as SPEEDS, BYTES as SPEED_BYTES, BotwoonSpeed, BotwoonSpeeds,
};
use sm_game::actors::access::ActorKey as Key;
pub(crate) use sm_game::actors::botwoon::*;
use sm_game::enemy_projectiles::drops::DropSource;

pub(crate) type ImportedSpeeds = Imported<BotwoonSpeeds>;

pub(crate) fn import_speeds(image: &RomImage) -> Option<ImportedSpeeds> {
    Imported::import(image, SPEEDS, SPEED_BYTES, BotwoonSpeeds::import)
}

pub(crate) type ImportedHoles = Imported<BotwoonHoles>;

pub(crate) fn import_holes(image: &RomImage) -> Option<ImportedHoles> {
    Imported::import(image, HOLES, HOLE_BYTES, BotwoonHoles::import)
}

impl Host for Game {
    /// The speed level whose two words start at word `index` of $B3:94BB. The
    /// record serves it while the loaded image is the imported allocation; any
    /// other image or index runs the original indexed loads.
    fn botwoon_speed_level(&self, index: isize) -> BotwoonSpeed {
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        Imported::from(&self.botwoon_speeds, rom)
            .filter(|_| index % 2 == 0)
            .and_then(|speeds| speeds.level(usize::try_from(index / 2).ok()?))
            .unwrap_or_else(|| BotwoonSpeed {
                speed: rom_table_word!(self, 0xb394bb, index),
                body_travel_time: rom_table_word!(self, 0xb394bb, index + 1),
            })
    }

    /// The hole whose four words start at word `index` of $B3:949B. The record
    /// serves a hole while the loaded image is the imported allocation; any other
    /// image or index runs the original indexed loads.
    fn botwoon_hole(&self, index: isize) -> Hole {
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        Imported::from(&self.botwoon_holes, rom)
            .filter(|_| index % 4 == 0)
            .and_then(|holes| holes.hole(usize::try_from(index / 4).ok()?))
            .unwrap_or_else(|| Hole {
                left: rom_table_word!(self, 0xb3949b, index),
                right: rom_table_word!(self, 0xb3949b, index + 1),
                top: rom_table_word!(self, 0xb3949b, index + 2),
                bottom: rom_table_word!(self, 0xb3949b, index + 3),
            })
    }

    fn angle_to_player(&mut self, actor: u16) -> u16 {
        api::angle_to_player(&Runtime::new(self), Key::from_legacy(actor))
    }

    fn set_boss_bit(&mut self, mask: u16) {
        SetBossBitForCurArea(mask, self);
    }

    fn boss_byte(&mut self, area: u16) -> u8 {
        boss_byte_at(area, self)
    }

    fn open_passage(&mut self) {
        crate::game::room_mechanisms::open_passage(self);
    }

    fn scatter_botwoon_drop(&mut self, area: u16) {
        scatter_drop(DropSource::Botwoon, area, self);
    }

    fn b3_pre_instruction(&mut self, address: u32, k: u16) {
        actor_b3_pre_instruction(address, k, self);
    }

    fn record_botwoon_head(&mut self) {
        enemy_projectiles::record_botwoon_head(self);
    }

    fn follow_botwoon_history(&mut self) {
        enemy_projectiles::follow_botwoon_history(self);
    }

    fn orient_botwoon_body(&mut self) {
        enemy_projectiles::orient_botwoon_body(self);
    }

    fn advance_botwoon_history(&mut self) {
        enemy_projectiles::advance_botwoon_history(self);
    }
}

impl RuntimeHost for Runtime<'_> {
    type Game = Game;

    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        Runtime::foreign(self, action)
    }
}

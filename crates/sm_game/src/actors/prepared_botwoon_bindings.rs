//! Actual prepared Botwoon content, effects and exceptional alias ports.
#![forbid(unsafe_code)]
use crate::actors::botwoon_owned::Host;
use crate::actors::owned_execution::PreparedActorState;
impl Host for PreparedActorState<'_> {
    fn exceptional_byte(&self, at: usize) -> u8 {
        self.current_byte(at)
    }
    fn set_exceptional_byte(&mut self, at: usize, value: u8) {
        self.set_current_byte(at, value);
    }
    fn botwoon_speed_level(&self, index: isize) -> sm_data::botwoon_speeds::BotwoonSpeed {
        // Preserve the original signed fallback's two independent word reads.
        // Imported records contain these same immutable allocation words.
        sm_data::botwoon_speeds::BotwoonSpeed {
            speed: self.loaded_table_word(0xb394bb, index),
            body_travel_time: self.loaded_table_word(0xb394bb, index + 1),
        }
    }
    fn botwoon_hole(&self, index: isize) -> sm_data::botwoon_holes::Hole {
        sm_data::botwoon_holes::Hole {
            left: self.loaded_table_word(0xb3949b, index),
            right: self.loaded_table_word(0xb3949b, index + 1),
            top: self.loaded_table_word(0xb3949b, index + 2),
            bottom: self.loaded_table_word(0xb3949b, index + 3),
        }
    }
    fn native_path_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, address)
    }
    fn loaded_botwoon_word(&self, address: u32, index: isize) -> u16 {
        self.loaded_table_word(address, index)
    }
    fn set_palette_word(&mut self, index: u16, value: u16) {
        crate::enemy_projectiles::host::Graphics::set_palette_buffer(
            self,
            isize::try_from(index).expect("palette index"),
            value,
        );
    }
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) {
        crate::sprite_objects::spawn(
            self,
            crate::sprite_effect_state::SpriteSpawn {
                position: crate::geometry::Point { x, y },
                definition: crate::sprite_effect_state::LegacySpriteDefinition(kind),
                appearance,
            },
        );
    }
    fn queue_music_delayed(&mut self, track: u16) {
        crate::music_requests::queue_delayed_8(track, self);
    }
    fn set_boss_bit(&mut self, mask: u16) {
        crate::progress::mark_boss_defeated(self, mask);
    }
    fn boss_byte(&mut self, area: u16) -> u8 {
        crate::progress::boss_byte_at(self, area)
    }
    fn spawn_plm(&mut self, x: u8, y: u8, header: u16) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Hardcoded { x, y, header },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    fn open_passage(&mut self) {
        crate::room_mechanisms::host::ImportedAliases::scroll_pair(self, 0, 0x101);
    }
    fn unreachable(&mut self) -> ! {
        self.unknown_function()
    }
}

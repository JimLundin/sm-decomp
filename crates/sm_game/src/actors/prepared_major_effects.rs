//! Concrete body explosions, palette acquisition and neck arithmetic.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use crate::session_access::SessionAccess;
impl crate::actors::boss_fields_owned::State for PreparedActorState<'_> {
    fn exceptional_byte(&self, at: usize) -> u8 {
        self.current_byte(at)
    }
    fn set_exceptional_byte(&mut self, at: usize, value: u8) {
        self.set_current_byte(at, value);
    }
}
impl crate::actors::mother_brain_effects_owned::Host for PreparedActorState<'_> {
    fn loaded_effects_word(&self, address: u32, index: isize) -> u16 {
        self.loaded_table_word(address, index)
    }
    fn native_effects_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, address)
    }
    fn native_palette_chain_word(&self, next: u16) -> u16 {
        // This original acquisition is an allocation offset, not a wrapped
        // bank address; it performs no low-window diagnostic.
        let image = self.allocations.native();
        let at = usize::try_from(
            sm_data::cartridge::lorom_offset(0xade434)
                + isize::try_from(next).expect("palette next")
                - 2,
        )
        .expect("palette chain offset");
        u16::from_ne_bytes([image.read(at), image.read(at + 1)])
    }
    fn next_random(&mut self) -> u16 {
        self.ordinary_random()
    }
    fn set_projectile_spawn_point(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = crate::geometry::Point { x, y };
    }
    fn spawn_death_explosion(&mut self, kind: u16) {
        crate::enemy_projectiles::lifecycle::spawn_room(
            self,
            {
                let [low, high, _, _] =
                    crate::constants::bank_86::ADDR_K_EPROJ_MOTHER_BRAIN_DEATH_EXPLOSION
                        .to_le_bytes();
                u16::from_le_bytes([low, high])
            },
            kind,
        );
    }
    fn guard<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        self.guarded_actor_call(action)
    }
}
impl crate::actors::mother_brain_neck_owned::Host for PreparedActorState<'_> {
    fn report_unreachable(&mut self) {
        self.unknown_function()
    }
    fn sine_times(&mut self, length: u16, angle: u16) -> u16 {
        self.acquired_mode7_multiply(length, angle)
    }
    fn cosine_times(&mut self, length: u16, angle: u16) -> u16 {
        self.acquired_mode7_multiply(length, angle.wrapping_add(64))
    }
}
impl crate::actors::baby_metroid_owned::Host for PreparedActorState<'_> {
    fn current_enemy(&self) -> u16 {
        self.session().actors.controls.selected
    }
    fn set_camera_x(&mut self, value: u16) {
        self.session_mut().room.camera.layer1[0] = value;
        self.refresh_inputs();
    }
    fn scroll_byte(&self, index: usize) -> u8 {
        crate::room_scrolls::state::byte(self, index)
    }
    fn set_scroll_word_native(&mut self, index: usize, value: u16) {
        crate::room_scrolls::state::store_word(
            self,
            index,
            value,
            crate::room_mechanisms::host::ByteOrder::Native,
        );
    }
    fn spawn_wall(&mut self, x: u8, y: u8, header: u16) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Hardcoded { x, y, header },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
}

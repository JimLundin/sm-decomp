//! Finite retained services for the shared owned minor-boss kernels.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use sm_game::actors::remaining_core::Core;
use sm_game::actors::{owned_corpse_bindings, remaining_owned::spore_spawn, statues};
impl sm_game::actors::minor_interaction::NativeCapture for Runtime<'_> {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        sm_game::cartridge::native_record(bank, address, self)
    }
}

impl statues::owned::Host for Runtime<'_> {
    fn tourian_statue(&self, index: isize) -> sm_data::statues::Statue {
        sm_game::statue_tables::tourian_statue(self, index)
    }
    fn chozo_statue(&self, index: isize) -> sm_data::statues::Statue {
        sm_game::statue_tables::chozo_statue(self, index)
    }
    fn statue_movement(&self, index: isize) -> sm_data::statues::Movement {
        sm_game::statue_tables::movement(self, index)
    }
    fn install_tourian_palettes(&mut self) {
        statues::install_tourian_palettes(self);
    }
    fn install_tube_palette(&mut self) {
        statues::install_tube_palette(self);
    }
    fn install_chozo_palettes(&mut self, lower_norfair: bool) {
        statues::install_chozo_palettes(lower_norfair, self);
    }
    fn boss_word_native(&mut self, area: u16) -> u16 {
        sm_game::progress::boss_word_at(self, area, sm_game::progress::BossWordOrder::Native)
    }
    fn start_lowering_acid(&mut self) {
        self.session_mut().room.liquid_delay = 32;
        self.session_mut().room.liquid_velocity = 64;
    }
    fn lower_acid(&mut self) {
        sm_game::room_mechanisms::rules::hands::lower_acid(self);
    }
    fn select_block_at(&mut self, x: u16, y: u16) {
        sm_game::actors::movement::select_block_at(self, sm_game::geometry::Point { x, y });
    }
    fn spawn_collision_plm(&mut self, header: u16) {
        sm_game::room_mechanisms::creation::spawn(
            self,
            sm_game::room_mechanisms::host::SpawnSource::Collision { header },
            &mut sm_game::room_mechanisms::host::Contact::Absent,
        );
    }
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    fn spawn_plm(&mut self, x: u16, y: u16, header: u16) {
        self.remaining_spawn_plm(x.to_le_bytes()[0], y.to_le_bytes()[0], header);
    }
    fn command_player(&mut self, command: u16) {
        self.remaining_player_command(command);
    }
    fn invalid_function(&mut self, _address: u16) -> ! {
        self.remaining_unknown()
    }
}
impl spore_spawn::Host for Runtime<'_> {
    fn update_spore_stem(&mut self) {
        sm_game::enemy_projectiles::attachments_spore_maw::update_spore_stem(self);
    }
    fn scatter_spore_spawn_drop(&mut self, key: u16) {
        sm_game::actors::deaths::Deaths::scatter_enemy_drop(
            self,
            sm_game::enemy_projectiles::drops::DropSource::SporeSpawn,
            key,
        );
    }
    fn clear_spore_fx_word(&mut self, word: u16) {
        if word <= 13 {
            self.session_mut().enemy_projectiles.slots[13 - usize::from(word)].definition = 0;
        } else {
            let at = 0x19b1usize - 2 * usize::from(word);
            self.set_byte(at, 0);
            self.set_byte(at + 1, 0);
        }
    }
    fn run_spore_return(&mut self, phase: spore_spawn::SporeReturn, key: u16) {
        use sm_game::actors::spore_spawn::owned_return;
        match phase {
            spore_spawn::SporeReturn::Descend => owned_return::descend(self),
            spore_spawn::SporeReturn::Orbit => owned_return::orbit(key, self),
            spore_spawn::SporeReturn::BeginReturn => owned_return::begin_return(self),
            spore_spawn::SporeReturn::FinishReturn => owned_return::finish_return(self),
        }
    }
}
impl owned_corpse_bindings::Host for Runtime<'_> {
    fn corpse_tile_alias_byte(&self, at: usize) -> u8 {
        self.byte(at)
    }
    fn write_corpse_tile_alias_slice(&mut self, at: usize, bytes: &[u8]) {
        self.write_slice(at, bytes);
    }
    fn corpse_native_word(&self, at: usize) -> u16 {
        self.native_word(at)
    }
    fn write_corpse_native_word(&mut self, at: usize, value: u16) {
        self.write_slice_native(at, value);
    }
    fn draw_corpse_spritemap(&mut self, map: u16, x: u16, y: u16, palette: u16) {
        sm_game::actors::mother_brain_fight::add_spritemap_to_oam(map, x, y, palette, self);
    }
}

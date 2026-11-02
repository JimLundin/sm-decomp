//! Concrete acquired small-family effects on the current actor owner.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use crate::actors::session_slots::session_slot;
use crate::actors::{
    access::{AccessMode, ActorAccess, ActorKey},
    api, combat, host, movement,
};
use crate::session_access::SessionAccess;
impl PreparedActorState<'_> {
    fn acquired_fault(&mut self) -> ! {
        self.unknown_function()
    }
    /// Byte-at-a-time palette copies keep the native record's lazy byte reads.
    /// Normal sprite palette destinations mutate the actual palette owner.
    fn small_family_palette_byte(&mut self, at: usize, value: u8) {
        if let Some(offset) = at.checked_sub(0xc000).filter(|offset| *offset < 512) {
            let palette = &mut self.owner.gameplay.presentation.visual_mut().colors[offset / 2];
            let mut bytes = palette.to_le_bytes();
            bytes[offset % 2] = value;
            *palette = u16::from_le_bytes(bytes);
            self.refresh_inputs();
        } else {
            self.set_current_byte(at, value);
        }
    }
}
impl crate::actors::etecoon::owned::Host for PreparedActorState<'_> {
    fn probe_horizontal(&mut self, key: u16, amount: i32) -> bool {
        movement::probe_horizontal(self, ActorKey::from_legacy(key), amount)
            == api::MovementResult::Blocked
    }
    fn invalid_function(&mut self, _address: u16) -> ! {
        self.acquired_fault()
    }
}
impl crate::actors::dachora_run::owned::Host for PreparedActorState<'_> {
    fn frame(&self) -> u16 {
        self.owner
            .gameplay
            .presentation
            .owner()
            .expect("owned dachora frame")
            .state
            .nmi
            .frame_word
    }
    fn load_palette(&mut self, source: u16, key: u16) {
        crate::probe::hit("dachora.load_palette");
        let index = self
            .resolve_actor(ActorKey::from_legacy(key), AccessMode::LittleEndian)
            .palette;
        let destination = usize::from(self.loaded_table_word(
            0xa7_f55f,
            isize::try_from(index.swap_bytes() >> 1).expect("dachora palette index"),
        ));
        let source = self.native_actor_record(0xa7, source);
        for offset in 0..32 {
            let value = source.byte(offset);
            self.small_family_palette_byte(destination + offset, value);
        }
    }
    fn load_shine_palette(&mut self, key: u16) {
        use crate::actors::dachora_run::Dachora;
        let me = i32::from(key);
        let step = isize::from(session_slot!(byte self,Dachora[me].shine[1]));
        let palette = crate::boss_lists::dachora_palette(self, true, step);
        crate::actors::dachora_run::owned::Host::load_palette(self, palette, key);
        let next = session_slot!(self, Dachora[me].shine).wrapping_add(256);
        session_slot!(
            self,
            Dachora[me].shine = if host::below(next, 0x400) { next } else { 0 }
        );
    }
    fn dachora_palette(&mut self, charging: bool, index: isize) -> u16 {
        crate::boss_lists::dachora_palette(self, charging, index)
    }
    fn invalid_function(&mut self, _address: u16) -> ! {
        self.acquired_fault()
    }
}
impl crate::actors::mini_kraid::owned::Host for PreparedActorState<'_> {
    fn spawn_actor_eproj(&mut self, parameter: u16, key: u16, definition: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, key, definition)
    }
    fn normal_death(&mut self, animation: u16) {
        combat::die(self, api::DeathKind::Normal { animation });
    }
    fn scatter_enemy_drop(
        &mut self,
        source: crate::enemy_projectiles::drops::DropSource,
        key: u16,
    ) {
        crate::enemy_projectiles::drops::scatter(source, key, self);
    }
}
impl crate::actors::kraid_lint::owned::Host for PreparedActorState<'_> {
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, address)
    }
    fn react_to_contact(&mut self, death: host::Death) {
        host::ActorRuntime::contact_reaction(self, death);
    }
}
impl crate::actors::zebetites::owned::Host for PreparedActorState<'_> {
    fn placement(&self, destroyed: isize) -> sm_data::zebetites::Placement {
        crate::zebetite_data::placement(self, destroyed)
    }
    fn health_lists(&self, stage: isize) -> sm_data::zebetites::HealthLists {
        crate::zebetite_data::health_lists(self, stage)
    }
    fn event_happened(&mut self, event: u16) -> u16 {
        u16::from(crate::progress::event_happened(self, event))
    }
    fn set_event(&mut self, event: u16) {
        crate::progress::set_event(self, event);
    }
    fn clear_event(&mut self, event: u16) {
        crate::progress::clear_event(self, event);
    }
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16 {
        crate::actors::population::spawn_legacy(self, bank, population)
    }
    fn glow(&mut self) {
        use crate::actors::zebetites::Zebetite;
        let selected = i32::from(self.owner.gameplay.actors.controls.selected);
        if self.owner.gameplay.presentation.visual().palette_fade.step == 0
            && session_slot!(self, Zebetite[selected].second_of_pair) == 0
        {
            let first = 0;
            let frame = session_slot!(self, Zebetite[first].glow_frame).wrapping_add(1) & 7;
            session_slot!(self, Zebetite[first].glow_frame = frame);
            let source = 0xfd87u16.wrapping_add(4 * frame);
            for index in 0..2 {
                let cursor = source.wrapping_add(u16::try_from(2 * index).expect("glow cursor"));
                let word = self.native_actor_record(0xa6, cursor).word(0);
                crate::enemy_projectiles::host::Graphics::set_palette_buffer(
                    self,
                    0xac + isize::try_from(index).expect("glow index"),
                    word,
                );
            }
        }
    }
    fn normal_death(&mut self, animation: u16) {
        combat::die(self, api::DeathKind::Normal { animation });
    }
    fn invalid_function(&mut self, _state: u16) -> ! {
        self.acquired_fault()
    }
    fn react_to_contact(&mut self, death: host::Death) {
        host::ActorRuntime::contact_reaction(self, death);
    }
    fn react_to_shot(&mut self, shot: host::Shot) {
        host::ActorRuntime::shot_reaction(self, shot);
    }
}

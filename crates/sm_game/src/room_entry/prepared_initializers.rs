//! Room FX producers execute on the current prepared actor owner.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use crate::display::preparation::Field as DisplayField;
use crate::session_access::SessionAccess;

impl PreparedActorState<'_> {
    pub(super) fn room_initialize_fx(&mut self, initializer: crate::room_state::fx::Initializer) {
        crate::owned_hdma_update::lifecycle::FxHost::room_initialize_fx(self, initializer);
    }
    pub(super) fn room_initialize_haze(&mut self) {
        crate::owned_hdma_update::lifecycle::FxHost::room_initialize_haze(self);
    }
    pub(super) fn room_initialize_land(&mut self) {
        crate::owned_hdma_update::lifecycle::FxHost::room_initialize_land(self);
    }
    pub(super) fn room_initialize_ocean(&mut self) {
        crate::owned_hdma_update::lifecycle::FxHost::room_initialize_ocean(self);
    }
    fn room_display_byte(&mut self, field: DisplayField, value: u8) {
        crate::display::preparation::Access::store_display_control_byte(
            self,
            field,
            value,
            crate::display::preparation::Store::Owned,
        );
    }
    pub(super) fn room_clear_hdma(&mut self, physical: bool) {
        if physical {
            self.aliases
                .room_initialization_register(self.owner, 0x420b, 0);
            self.refresh_inputs();
            self.aliases
                .room_initialization_register(self.owner, 0x420c, 0);
            self.refresh_inputs();
        }
        self.room_display_byte(DisplayField::HdmaEnable, 0);
        for slot in (0..crate::hdma::OBJECTS).rev() {
            crate::hdma_word!(self, slot, Channels => 0);
        }
    }
    pub(super) fn room_initialize_special_effects(&mut self) {
        crate::owned_hdma_update::lifecycle::ResetHost::room_initialize_special_effects(self);
    }
    pub(super) fn room_load_explored_map(&mut self) {
        let first = self.session().room.area.swap_bytes();
        for active in 0u16..128 {
            let saved = first.wrapping_add(active * 2) >> 1;
            let at = 0xcd52 + (usize::from(saved) * 2);
            let value = if let Some(pair) = self
                .session()
                .save_state
                .area_maps
                .get(usize::from(saved) * 2..usize::from(saved) * 2 + 2)
            {
                u16::from_le_bytes(pair.try_into().expect("saved map word"))
            } else {
                self.current_word(at)
            };
            let at = usize::from(active) * 2;
            self.session_mut().minimap.explored[at..at + 2].copy_from_slice(&value.to_ne_bytes());
        }
        let area = usize::from(self.session().room.area);
        let value = self
            .session()
            .gameplay_flags
            .map_stations
            .get(area)
            .copied()
            .unwrap_or_else(|| self.current_byte(0xd908 + area));
        self.session_mut().menu_screens.save_selection.has_area_map = u16::from(value);
        self.refresh_inputs();
    }
    pub(super) fn room_save_explored_map(&mut self) {
        let first = self.session().room.area.swap_bytes();
        for active in 0u16..128 {
            let saved = first.wrapping_add(active * 2) >> 1;
            let active = usize::from(active) * 2;
            let value = u16::from_ne_bytes(
                self.session().minimap.explored[active..active + 2]
                    .try_into()
                    .expect("active map word"),
            );
            let saved = usize::from(saved) * 2;
            if let Some(pair) = self
                .session_mut()
                .save_state
                .area_maps
                .get_mut(saved..saved + 2)
            {
                pair.copy_from_slice(&value.to_le_bytes());
            } else {
                self.write_current_word_le(0xcd52 + saved, value);
            }
        }
        if self.session().menu_screens.save_selection.has_area_map != 0 {
            let at = 0xd908 + usize::from(self.session().room.area);
            let value = self.current_native_word(at) | 0xff;
            self.write_current_slice(at, &value.to_ne_bytes());
        }
        self.refresh_inputs();
    }
    pub(super) fn room_mark_elevators_used(&mut self) {
        let table = crate::menu_lists::elevator_door_table(
            self,
            isize::try_from(self.session().room.area).expect("elevator area"),
        );
        let offset = (self.session().room.entry.elevator_properties & 15)
            .wrapping_sub(1)
            .wrapping_mul(4);
        let cursor = table.wrapping_add(offset);
        if cursor & 0x8000 == 0 {
            crate::cartridge::NativeDiagnostics::native_rom_diagnostic(
                self,
                0x800000 | u32::from(cursor),
            );
        }
        let image = self.allocations.native().clone();
        let base = usize::try_from(sm_data::cartridge::lorom_offset(
            0x800000 | u32::from(cursor),
        ))
        .expect("elevator record");
        for record in [0, 2] {
            let index = usize::from(image.read(base + record));
            let previous = self
                .session()
                .gameplay_flags
                .used_stations
                .get(index)
                .copied()
                .unwrap_or_else(|| self.current_byte(0xd8f8 + index));
            let value = previous | image.read(base + record + 1);
            if let Some(byte) = self
                .session_mut()
                .gameplay_flags
                .used_stations
                .get_mut(index)
            {
                *byte = value;
            } else {
                self.set_current_byte(0xd8f8 + index, value);
            }
        }
        self.refresh_inputs();
    }
}
impl crate::fx_lighting::Access for PreparedActorState<'_> {
    fn lighting(&self) -> crate::visual_state::FirefleaLighting {
        crate::fx_encoding::session_lighting(self.session())
    }
    fn publish_lighting(
        &mut self,
        value: crate::visual_state::FirefleaLighting,
        fields: &[crate::fx_lighting::Field],
    ) {
        for &field in fields {
            crate::fx_encoding::write_control_word(
                self.session_mut(),
                crate::fx_encoding::lighting_address(field),
                field.get(&value),
            );
        }
        self.refresh_inputs();
    }
}
impl crate::owned_hdma_update::lifecycle::FxHost for PreparedActorState<'_> {}
impl crate::owned_hdma_update::lifecycle::ResetHost for PreparedActorState<'_> {
    fn room_mode7(&mut self, value: u16) {
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.set_current_byte(0x783 + lane, byte);
        }
    }
    fn room_register(&mut self, register: u16, value: u8) {
        self.aliases
            .room_initialization_register(self.owner, register, value);
        self.refresh_inputs();
    }
}

//! Acquired and major-boss hardware effects on the exclusive current owner.
#![forbid(unsafe_code)]
use super::owned_execution::PreparedActorState;
use crate::session_access::SessionAccess;

impl PreparedActorState<'_> {
    pub(crate) fn hardware_initialize_bomb_torizo_haze(&mut self) {
        use crate::display::hdma_spawn::HdmaObjectDefinition;
        crate::actors::boss_hdma_owned::spawn_hdma_owned(
            0x88,
            &HdmaObjectDefinition::new(2, 0x32, 0xdd4a),
            self,
        );
        crate::actors::boss_hdma_owned::spawn_hdma_owned(
            0x88,
            &HdmaObjectDefinition::new(0, 0x32, 0xdd62),
            self,
        );
    }
    pub(crate) fn hardware_initialize_morph_eye_beam(&mut self) {
        crate::actors::boss_hdma_owned::spawn_hdma_owned(
            0x88,
            &crate::display::hdma_spawn::HdmaObjectDefinition::new(0x41, 0x26, 0xe8ec),
            self,
        );
    }
    pub(crate) fn acquired_draw_room_sprite(
        &mut self,
        bank: u8,
        map: u16,
        position: crate::geometry::Point,
        palette: u16,
    ) {
        self.presentation
            .draw_room_sprite(
                &mut self.owner.gameplay,
                crate::scene::actor_presentation::RoomSpriteRequest {
                    bank,
                    map,
                    position,
                    palette,
                    clip: crate::scene::objects::RoomClip::Rows,
                },
                self.aliases.presentation_diagnostics(),
            )
            .expect("admitted room boss sprite");
    }
    pub(crate) fn acquired_multiplicand(&mut self, operand: u8) {
        self.session_mut()
            .presentation
            .owner_mut()
            .expect("owned Maw arithmetic")
            .state
            .arithmetic
            .multiplicand = operand;
    }
    pub(crate) fn acquired_mode7_multiply(&mut self, amplitude: u16, angle: u16) -> u16 {
        crate::scene::hardware::sine_times(
            &mut self
                .session_mut()
                .presentation
                .owner_mut()
                .expect("owned boss affine hardware")
                .state,
            amplitude,
            angle,
        )
    }
    pub(crate) fn acquired_mode7_irq(&mut self, value: u16) {
        self.session_mut()
            .presentation
            .owner_mut()
            .expect("owned Ceres IRQ")
            .state
            .interrupt
            .mode7 = value;
    }
    pub(crate) fn acquired_install_draygon_layers(&mut self) {
        crate::actors::boss_hdma_owned::spawn_hdma_owned(
            0x88,
            &crate::display::hdma_spawn::HdmaObjectDefinition::new(0, 0x2c, 0xdf4f),
            self,
        );
        self.session_mut().room.entry.irq_policy = 12;
        crate::actors::boss_hdma_owned::spawn_bg3_scroll(self);
    }
    pub(crate) fn acquired_mode7_queue(&mut self, bank: u8, cursor: u16) {
        crate::display::mode7_owned::enqueue(bank, cursor, self);
    }
}

impl super::boss_hdma_owned::Host for PreparedActorState<'_> {
    fn write_dma_channel(
        &mut self,
        channel: usize,
        field: super::boss_hdma_owned::DmaField,
        value: u8,
    ) {
        let field = match field {
            super::boss_hdma_owned::DmaField::Control => crate::scene::hardware::DmaField::Control,
            super::boss_hdma_owned::DmaField::Target => crate::scene::hardware::DmaField::Target,
        };
        crate::scene::hardware::write_dma_channel(
            &mut self
                .session_mut()
                .presentation
                .owner_mut()
                .expect("owned boss DMA channels")
                .state,
            channel,
            field,
            value,
        )
        .expect("admitted boss DMA channel");
    }
}

impl crate::hdma_objects::HdmaTable for PreparedActorState<'_> {
    fn hdma_word(&self, key: crate::hdma::Word) -> u16 {
        self.session().hdma_word(key)
    }
    fn store_hdma_word(
        &mut self,
        key: crate::hdma::Word,
        value: u16,
        order: crate::scene::state::WordOrder,
    ) {
        self.session_mut().store_hdma_word(key, value, order);
    }
    fn store_hdma_byte(&mut self, key: crate::hdma::Word, high: bool, value: u8) {
        self.session_mut().store_hdma_byte(key, high, value);
    }
}

impl crate::display::mode7_owned::Queue for PreparedActorState<'_> {
    fn native_list(&mut self, bank: u8, cursor: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, cursor)
    }
    fn tail(&self) -> u16 {
        self.session()
            .presentation
            .owner()
            .expect("owned Mode7 queue")
            .state
            .mode7_tail
    }
    fn write_byte(&mut self, at: usize, value: u8) {
        self.presentation
            .mode7_queue_byte(
                &mut self.owner.gameplay,
                at,
                value,
                self.aliases.presentation_diagnostics(),
            )
            .expect("admitted Mode7 producer byte");
        crate::session_access::SessionAccess::refresh_inputs(self);
    }
    fn publish_tail(&mut self, value: u16) {
        self.session_mut()
            .presentation
            .owner_mut()
            .expect("owned Mode7 tail")
            .state
            .mode7_tail = value;
    }
}

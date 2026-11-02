//! Actual dialogue polling on the already admitted actor context.
#![forbid(unsafe_code)]
use super::owned_execution::PreparedActorState;
use crate::{
    gameplay_context::Word,
    session_access::SessionAccess,
    session_words::{SessionWords, Word as SequenceWord},
};

impl crate::gameplay_context::Access for PreparedActorState<'_> {
    fn inventory_state(&self) -> crate::inventory::Inventory {
        self.session().inventory.clone()
    }
    fn commit_inventory(
        &mut self,
        _before: &crate::inventory::Inventory,
        after: crate::inventory::Inventory,
    ) {
        self.session_mut().inventory = after;
        self.refresh_inputs();
    }
    fn gameplay_word(&self, word: Word) -> u16 {
        word.get_session(self.session())
    }
    fn set_gameplay_byte(&mut self, word: Word, lane: usize, value: u8) {
        sm_data::table::check_index(lane, 2);
        self.write_current_slice(word.address() + lane, &[value]);
    }
}
impl crate::sequence_state::Access for PreparedActorState<'_> {
    fn sequence_flow(&self) -> crate::flow::Flow {
        self.session().flow
    }
    fn set_cinematic_function(&mut self, value: u16) {
        self.session_mut().flow.cinematic_function = value;
        self.refresh_inputs();
    }
    fn sequence_words(&self) -> SessionWords {
        SessionWords::from_session(self.session())
    }
    fn commit_flow(&mut self, _before: &crate::flow::Flow, after: crate::flow::Flow) {
        self.session_mut().flow = after;
        self.refresh_inputs();
    }
    fn commit_sequence_words(&mut self, before: &SessionWords, after: SessionWords) {
        after.apply_changed(before, self.session_mut());
        self.refresh_inputs();
    }
    fn set_sequence_word(&mut self, word: SequenceWord, value: u16) {
        self.write_current_slice(word as usize, &value.to_le_bytes());
    }
}
impl crate::menus::controller_input::Sampling for PreparedActorState<'_> {
    fn sampling_controller_word(&mut self, register: u16) -> u16 {
        let result = self.aliases.dialogue_controller_word(self.owner, register);
        self.refresh_inputs();
        result
    }
    fn sampling_inputs(&self) -> crate::external_inputs::ExternalInputs {
        self.owner.observations.clone()
    }
    fn sampling_uploading_apu(&self) -> u16 {
        self.current_word(0x617)
    }
    fn sampling_toggle_nmi(&mut self) {
        let value = self.current_word(crate::ram::mirror::NMITIMEN) ^ 0x30;
        self.write_current_slice(crate::ram::mirror::NMITIMEN, &value.to_le_bytes());
    }
    fn sampling_reset(&mut self) {
        self.aliases.dialogue_request_reset(self.owner);
        self.refresh_inputs();
    }
}
impl PreparedActorState<'_> {
    pub fn sample_dialogue_input(
        &mut self,
    ) -> Result<crate::message_composition::prepared::InputSample, sm_data::message::Fault> {
        crate::menus::controller_input::sample(self);
        let input = &self
            .owner
            .gameplay
            .control
            .metadata()
            .map_err(|_| sm_data::message::Fault::MissingDisplay)?
            .input_history;
        Ok(crate::message_composition::prepared::InputSample {
            new_keys: input.ports[0].pressed,
            held_keys: input.ports[0].sampled,
            bug_fix_counter: input.bug_fix_counter,
        })
    }
}

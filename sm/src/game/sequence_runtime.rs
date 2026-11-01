//! Sequence controls on the already admitted aggregate lease.
#![forbid(unsafe_code)]
use super::Runtime;
use sm_game::{
    flow::Flow,
    sequence_state::{Access, State},
    session_words::SessionWords,
};
impl Access for Runtime<'_> {
    fn set_cinematic_function(&mut self, value: u16) {
        // Preserve the prior whole-extent check and byte-refresh sequence.
        self.write_slice_le(0x1f51, value);
    }
    fn set_sequence_word(&mut self, word: sm_game::session_words::Word, value: u16) {
        self.write_word_le(value, word as usize);
    }
    fn sequence_flow(&self) -> Flow {
        if self.persistent_is_current() {
            self.session().flow
        } else {
            sm_game::session_encoding::flow(&|at| self.byte(at))
        }
    }
    fn sequence_words(&self) -> SessionWords {
        if self.persistent_is_current() {
            State::from_session(self.session()).words
        } else {
            use sm_game::session_cohort::Cohort as _;
            let mut words = SessionWords::default();
            words.words(|at, value| *value = self.word(at));
            words
        }
    }
    fn commit_flow(&mut self, before: &Flow, after: Flow) {
        if *before == after {
            return;
        }
        if self.persistent_is_current() {
            after.apply_changed(before, &mut self.session_mut().flow);
        } else {
            for (at, value) in after.changes(before) {
                self.write_word_le(value, at);
            }
        }
        self.refresh_inputs();
    }
    fn commit_sequence_words(&mut self, before: &SessionWords, after: SessionWords) {
        if *before == after {
            return;
        }
        if self.persistent_is_current() {
            after.apply_changed(before, self.session_mut());
        } else {
            for (at, value) in after.changes(before) {
                self.write_word_le(value, at);
            }
        }
        self.refresh_inputs();
    }
}

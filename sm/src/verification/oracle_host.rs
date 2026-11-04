//! The verification host over the oracle's work RAM and backend (D6, D7).
//!
//! The oracle's settlement (`cpu_infra::draw_frame_to_ppu` after an emulated
//! frame) and the routes that step frames by hand run sm_game's scanline
//! interrupt, display and RAM views on Game's image, outside any lease. They
//! take this host, not Game: Game serves no sm_game host trait for them. Each
//! method is the projection Game's own byte path makes (its held frame display,
//! scroll and sequence domains, then RAM), so the reads and stores are the
//! ones the former Game impls made, in the same order.
use crate::game_state::Game;
use sm_game::display::preparation::{Field, Store, Word as DisplayWord};
use sm_game::display_scroll::Word as ScrollWord;

/// Game's image, lent to sm_game for the oracle's settlement and the
/// hand-stepped verification routes.
pub(crate) struct OracleHost<'a>(pub(crate) &'a mut Game);

impl OracleHost<'_> {
    /// The Game whose image this host projects.
    pub(crate) fn game(&mut self) -> &mut Game {
        self.0
    }
}

impl sm_game::display::video::Video for OracleHost<'_> {
    fn apply(&mut self, producer: &'static str, command: sm_game::display::video::Command) {
        self.0.apply(producer, command);
    }
}

impl sm_game::display::preparation::Access for OracleHost<'_> {
    fn display_control_byte(&self, field: Field) -> u8 {
        self.0.display_control_byte(field)
    }
    fn store_display_control_byte(&mut self, field: Field, value: u8, store: Store) {
        self.0.store_display_control_byte(field, value, store);
    }
    fn display_history_word(&self, word: DisplayWord) -> u16 {
        self.0.display_history_word(word)
    }
    fn store_display_history_word(&mut self, word: DisplayWord, value: u16, store: Store) {
        self.0.store_display_history_word(word, value, store);
    }
    fn hud_tile(&self, index: isize) -> u16 {
        self.0.hud_tile(index)
    }
    fn store_hud_tile(&mut self, index: isize, value: u16, store: Store) {
        self.0.store_hud_tile(index, value, store);
    }
    fn physical_display_control_byte(&self, field: Field) -> u8 {
        self.0.physical_display_control_byte(field)
    }
    fn store_physical_display_control_byte(&mut self, field: Field, value: u8) {
        self.0.store_physical_display_control_byte(field, value);
    }
}

impl crate::projection::ProjectedBytes for OracleHost<'_> {
    fn compatibility_byte(&self, at: usize) -> u8 {
        self.0.compatibility_byte(at)
    }
    fn compatibility_write_byte(&mut self, at: usize, value: u8) {
        self.0.compatibility_write_byte(at, value);
    }
}
crate::projection::projected_views!(OracleHost<'_>);

impl sm_game::ram::CompatibilityRam for OracleHost<'_> {
    fn observed_inputs(&self) -> sm_game::external_inputs::ExternalInputs {
        self.0.observed_inputs()
    }
}

impl sm_game::display_scroll::Access for OracleHost<'_> {
    fn scroll_word(&self, word: ScrollWord) -> u16 {
        self.0.scroll_word(word)
    }
    fn actor_scroll_word(&self, word: ScrollWord) -> u16 {
        self.0.actor_scroll_word(word)
    }
    fn set_scroll_word(&mut self, word: ScrollWord, value: u16) {
        self.0.set_scroll_word(word, value);
    }
    fn physical_scroll_byte(&self, word: ScrollWord, high: bool) -> u8 {
        self.0.physical_scroll_byte(word, high)
    }
    fn write_physical_scroll_byte(&mut self, word: ScrollWord, high: bool, value: u8) {
        self.0.write_physical_scroll_byte(word, high, value);
    }
}

impl sm_game::sequence_state::Access for OracleHost<'_> {
    fn sequence_flow(&self) -> sm_game::flow::Flow {
        self.0.sequence_flow()
    }
    fn set_cinematic_function(&mut self, value: u16) {
        self.0.set_cinematic_function(value);
    }
    fn sequence_words(&self) -> sm_game::session_words::SessionWords {
        self.0.sequence_words()
    }
    fn commit_flow(&mut self, before: &sm_game::flow::Flow, after: sm_game::flow::Flow) {
        self.0.commit_flow(before, after);
    }
    fn commit_sequence_words(
        &mut self,
        before: &sm_game::session_words::SessionWords,
        after: sm_game::session_words::SessionWords,
    ) {
        self.0.commit_sequence_words(before, after);
    }
    fn set_sequence_word(&mut self, word: sm_game::session_words::Word, value: u16) {
        self.0.set_sequence_word(word, value);
    }
}

impl sm_game::gameplay_context::Access for OracleHost<'_> {
    fn grapple_animation(&self) -> sm_game::gameplay_context::GrappleAnimation {
        self.0.grapple_animation()
    }
    fn commit_grapple_animation(
        &mut self,
        before: sm_game::gameplay_context::GrappleAnimation,
        after: sm_game::gameplay_context::GrappleAnimation,
    ) {
        self.0.commit_grapple_animation(before, after);
    }
    fn inventory_state(&self) -> sm_game::inventory::Inventory {
        self.0.inventory_state()
    }
    fn commit_inventory(
        &mut self,
        before: &sm_game::inventory::Inventory,
        after: sm_game::inventory::Inventory,
    ) {
        self.0.commit_inventory(before, after);
    }
    fn gameplay_word(&self, word: sm_game::gameplay_context::Word) -> u16 {
        self.0.gameplay_word(word)
    }
    fn set_gameplay_byte(&mut self, word: sm_game::gameplay_context::Word, lane: usize, byte: u8) {
        self.0.set_gameplay_byte(word, lane, byte);
    }
}

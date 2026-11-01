//! Scroll domain residency, publication, and selected physical-image boundaries.
#![forbid(unsafe_code)]
use super::Runtime;
use sm_game::display_scroll::{Access, Word};

fn address(word: Word) -> usize {
    0xb1 + 2 * usize::from(word)
}

impl Access for Runtime<'_> {
    fn scroll_word(&self, word: Word) -> u16 {
        if self.display_owned() {
            return self
                .session()
                .presentation
                .owner()
                .expect("owned Display")
                .state
                .controls
                .scroll_words[usize::from(word)];
        }
        self.game.scroll_word(word)
    }
    fn set_scroll_word(&mut self, word: Word, value: u16) {
        if self.display_owned() {
            for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.set_byte(address(word) + offset, byte);
            }
        } else if self.persistent_is_current() {
            self.game.set_scroll_word(word, value);
            self.inputs =
                crate::game::session_encoding::inputs(&|at| self.game.ram().read_byte(at));
        } else {
            // Match the original dirty Runtime's raw-byte fallback and refresh.
            for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.set_byte(address(word) + offset, byte);
            }
        }
    }
    fn physical_scroll_byte(&self, word: Word, high: bool) -> u8 {
        self.game.physical_scroll_byte(word, high)
    }
    fn write_physical_scroll_byte(&mut self, word: Word, high: bool, value: u8) {
        self.game.write_physical_scroll_byte(word, high, value);
    }
}

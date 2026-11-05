//! Scroll domain residency, publication, and selected physical-image boundaries.
#![forbid(unsafe_code)]
use super::Runtime;
use sm_game::display_scroll::{Access, Word};

fn address(word: Word) -> usize {
    0xb1 + 2 * usize::from(word)
}

impl Access for Runtime<'_> {
    fn scroll_word(&self, word: Word) -> u16 {
        // A current Session holds the words: the owned Display's or the Legacy
        // presentation's.
        if self.persistent_is_current() {
            return self.session().presentation.scroll_word(word);
        }
        self.game.scroll_word(word)
    }
    fn set_scroll_word(&mut self, word: Word, value: u16) {
        if self.display_owned() {
            for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.set_byte(address(word) + offset, byte);
            }
        } else if self.persistent_is_current() {
            self.session_mut().presentation.set_scroll_word(word, value);
            self.refresh_scroll_inputs();
        } else {
            // Match the original dirty Runtime's raw-byte fallback and refresh.
            for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.set_byte(address(word) + offset, byte);
            }
        }
    }
    fn physical_scroll_byte(&self, word: Word, high: bool) -> u8 {
        // The Legacy presentation's word is the byte its export publishes; an
        // owned Display's physical byte is RAM's, which it publishes later.
        if self.persistent_is_current() && !self.has_owned_display() {
            return self.session().presentation.scroll_word(word).to_le_bytes()[usize::from(high)];
        }
        self.game.physical_scroll_byte(word, high)
    }
    fn write_physical_scroll_byte(&mut self, word: Word, high: bool, value: u8) {
        if self.persistent_is_current() && !self.has_owned_display() {
            let presentation = &mut self.session_mut().presentation;
            let mut bytes = presentation.scroll_word(word).to_le_bytes();
            bytes[usize::from(high)] = value;
            presentation.set_scroll_word(word, u16::from_le_bytes(bytes));
            return;
        }
        self.game.write_physical_scroll_byte(word, high, value);
    }
}

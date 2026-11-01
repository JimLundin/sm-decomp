//! The eight presentation scroll words and their direct ownership contract.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Bg1H,
    Bg1V,
    Bg2H,
    Bg2V,
    Bg3H,
    Bg3V,
    Bg4H,
    Bg4V,
}
pub const WORDS: [Word; 8] = [
    Word::Bg1H,
    Word::Bg1V,
    Word::Bg2H,
    Word::Bg2V,
    Word::Bg3H,
    Word::Bg3V,
    Word::Bg4H,
    Word::Bg4V,
];
impl From<Word> for usize {
    fn from(word: Word) -> Self {
        match word {
            Word::Bg1H => 0,
            Word::Bg1V => 1,
            Word::Bg2H => 2,
            Word::Bg2V => 3,
            Word::Bg3H => 4,
            Word::Bg3V => 5,
            Word::Bg4H => 6,
            Word::Bg4V => 7,
        }
    }
}

/// Residency metadata belongs to the adapter, never to persisted gameplay.
/// Legacy presentation has no Display; this owner never manufactures one.
pub enum ScrollDomain {
    Unowned,
    Owned { scroll: [u16; 8], epoch: u64 },
    InSession { epoch: u64 },
    Leased,
    Revoked,
}

pub trait Access {
    fn scroll_word(&self, word: Word) -> u16;
    fn set_scroll_word(&mut self, word: Word, value: u16);
    /// Actor slot aliases are physical on Game and projected on its Runtime.
    fn actor_scroll_word(&self, word: Word) -> u16 {
        self.scroll_word(word)
    }
    fn scroll_byte(&self, word: Word, high: bool) -> u8 {
        self.scroll_word(word).to_le_bytes()[usize::from(high)]
    }
    /// A physical byte snapshot deliberately bypasses the projected owner.
    /// Pause saves and odd/native aliases retain this distinct sampling phase.
    fn physical_scroll_byte(&self, word: Word, high: bool) -> u8;
    fn write_physical_scroll_byte(&mut self, word: Word, high: bool, value: u8);
}
pub fn read<H: Access + ?Sized>(host: &H, word: Word) -> u16 {
    host.scroll_word(word)
}
pub fn write<H: Access + ?Sized>(host: &mut H, word: Word, value: u16) {
    host.set_scroll_word(word, value);
}

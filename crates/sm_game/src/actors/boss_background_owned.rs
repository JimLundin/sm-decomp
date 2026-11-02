//! Kraid/Crocomire BG2 staging on the existing owned actor extension.
#![forbid(unsafe_code)]
use crate::actors::boss_fields_owned::{State, read_byte, store_byte};
pub fn get<H: State + ?Sized>(h: &H, index: usize) -> u16 {
    let at = 0x2000 + 2 * index;
    u16::from_le_bytes([read_byte(h, at), read_byte(h, at + 1)])
}
pub fn set<H: State + ?Sized>(h: &mut H, index: usize, value: u16) {
    let [low, high] = value.to_le_bytes();
    store_byte(h, 0x2000 + 2 * index, low);
    store_byte(h, 0x2001 + 2 * index, high);
}
pub fn fill<H: State + ?Sized>(h: &mut H, words: impl Iterator<Item = usize>, value: u16) {
    for index in words {
        set(h, index, value);
    }
}

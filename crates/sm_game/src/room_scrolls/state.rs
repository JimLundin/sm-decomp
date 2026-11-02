//! The room's scroll bytes, including native and odd adjacent accesses.
#![forbid(unsafe_code)]
use crate::room_mechanisms::host::ByteOrder;
use crate::session_access::SessionAccess;

/// Exceptional offsets retain their full identity. The caller resolves the
/// adjacent canonical owner or an imported semantic alias, never a scroll copy.
pub trait Aliases: SessionAccess {
    fn scroll_alias_byte(&self, byte_offset: usize) -> u8;
    fn store_scroll_alias_byte(&mut self, byte_offset: usize, value: u8);
    /// Validate the complete selected destination before publishing either byte.
    /// Implementations also resolve pairs crossing canonical owner boundaries.
    fn store_scroll_alias_pair(&mut self, byte_offset: usize, value: [u8; 2]);
}

pub fn byte<H: Aliases + ?Sized>(h: &H, byte_offset: usize) -> u8 {
    0xcd20usize.checked_add(byte_offset).expect("scroll extent");
    h.session()
        .room
        .scrolls
        .get(byte_offset)
        .copied()
        .unwrap_or_else(|| h.scroll_alias_byte(byte_offset))
}

pub fn store_byte<H: Aliases + ?Sized>(h: &mut H, byte_offset: usize, value: u8) {
    0xcd20usize.checked_add(byte_offset).expect("scroll extent");
    if let Some(cell) = h.session_mut().room.scrolls.get_mut(byte_offset) {
        *cell = value;
    } else {
        h.store_scroll_alias_byte(byte_offset, value);
    }
}

pub fn word<H: Aliases + ?Sized>(h: &H, byte_offset: usize, order: ByteOrder) -> u16 {
    let first = byte(h, byte_offset);
    let second = byte(h, byte_offset.checked_add(1).expect("scroll extent"));
    match order {
        ByteOrder::Little => u16::from_le_bytes([first, second]),
        ByteOrder::Native => u16::from_ne_bytes([first, second]),
    }
}

/// Prefix publication: failure on the second byte preserves the first store.
pub fn store_word<H: Aliases + ?Sized>(
    h: &mut H,
    byte_offset: usize,
    value: u16,
    order: ByteOrder,
) {
    let [first, second] = match order {
        ByteOrder::Little => value.to_le_bytes(),
        ByteOrder::Native => value.to_ne_bytes(),
    };
    store_byte(h, byte_offset, first);
    store_byte(
        h,
        byte_offset.checked_add(1).expect("scroll extent"),
        second,
    );
}

/// Whole pair publication, as used by room installation and mechanisms.
pub fn store_pair<H: Aliases + ?Sized>(h: &mut H, byte_offset: usize, value: [u8; 2]) {
    let end = byte_offset.checked_add(2).expect("scroll extent");
    0xcd20usize.checked_add(end).expect("scroll extent");
    if end <= h.session().room.scrolls.len() {
        h.session_mut().room.scrolls[byte_offset..end].copy_from_slice(&value);
    } else {
        h.store_scroll_alias_pair(byte_offset, value);
    }
}

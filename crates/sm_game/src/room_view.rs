//! The room's area and geometry for code on the compatibility host.
//!
//! `Session::room` owns these words (`area`, `width`, `height`,
//! `scroll_width`, `scroll_height`); `session_encoding` projects them and
//! names their layout once. Code that runs outside a lease (the menus, the
//! display refresh, the intro) reads them with [`current`] and stores with
//! [`set`], through the host's projected bytes: the owned Session's
//! projection while one is installed, the byte image otherwise, with the same
//! bytes and order as the named words they replace. Lease code uses
//! `session().room` directly.
#![forbid(unsafe_code)]
use crate::ram::CompatibilityRam;

/// The owned room words this view covers.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RoomGeometry {
    /// The area index (`area_index`).
    pub area: u16,
    /// The room's size in blocks and in scrolls.
    pub width: u16,
    pub height: u16,
    pub scroll_width: u16,
    pub scroll_height: u16,
}

/// A room word, by its Session field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Area,
    Width,
    Height,
    ScrollWidth,
    ScrollHeight,
}

/// The room words as the host's projection holds them.
pub fn current<H: CompatibilityRam + ?Sized>(h: &H) -> RoomGeometry {
    crate::session_encoding::room_geometry(&crate::projection::reader(h))
}

/// Store `value` to `word`, low byte then high, as the named word store did.
pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
    let at = crate::session_encoding::room_geometry_addresses()[word as usize];
    crate::projection::set_word(h, at, value);
}

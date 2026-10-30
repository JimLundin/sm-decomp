//! The room's area, geometry, effect and placement words for code on the
//! compatibility host.
//!
//! `Session::room` owns these words (`area`, `width`, `height`,
//! `scroll_width`, `scroll_height`); `session_encoding` projects them and
//! names their layout once. Code that runs outside a lease (the menus, the
//! display refresh, the intro) reads them with [`current`] and stores with
//! [`set`], through the host's projected bytes: the owned Session's
//! projection while one is installed, the byte image otherwise, with the same
//! bytes and order as the named words they replace. Lease code uses
//! `session().room` directly.
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

/// The room's effect and placement words: the earthquake, the Tourian
/// entrance statue and the room's map position and elevator door.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RoomEffects {
    pub earthquake_kind: u16,
    pub earthquake_timer: u16,
    pub statue_animation: u16,
    pub statue_finished: u16,
    pub map_x: u16,
    pub map_y: u16,
    pub elevator_properties: u16,
}

/// An effect or placement word, by its Session field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectWord {
    /// `feedback.earthquake_kind` (`earthquake_type`).
    EarthquakeKind,
    /// `feedback.earthquake_timer`.
    EarthquakeTimer,
    /// `mechanisms.statue_animation` (`tourian_entrance_statue_animstate`).
    StatueAnimation,
    /// `mechanisms.statue_finished` (`tourian_entrance_statue_finished`).
    StatueFinished,
    /// `room.entry.map_x` (`room_x_coordinate_on_map`).
    MapX,
    /// `room.entry.map_y` (`room_y_coordinate_on_map`).
    MapY,
    /// `room.entry.elevator_properties` (`elevator_door_properties_orientation`).
    ElevatorProperties,
}

/// The effect and placement words as the host's projection holds them.
pub fn effects<H: CompatibilityRam + ?Sized>(h: &H) -> RoomEffects {
    crate::session_encoding::room_effects(&crate::projection::reader(h))
}

/// Store `value` to `word`, low byte then high, as the named word store did.
pub fn set_effect<H: CompatibilityRam + ?Sized>(h: &mut H, word: EffectWord, value: u16) {
    crate::projection::set_word(h, crate::session_encoding::room_effect_address(word), value);
}

/// Take the area and map position from room header bytes 1..=3, storing only
/// each word's low byte, as the original's byte stores did.
pub fn enter_header<H: CompatibilityRam + ?Sized>(h: &mut H, bytes: [u8; 3]) {
    let area = crate::session_encoding::room_geometry_addresses()[Word::Area as usize];
    let [x, y] =
        [EffectWord::MapX, EffectWord::MapY].map(crate::session_encoding::room_effect_address);
    for (at, byte) in [area, x, y].into_iter().zip(bytes) {
        crate::projection::set_byte(h, at, byte);
    }
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

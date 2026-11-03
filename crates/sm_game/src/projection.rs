//! What sm_game's typed views require of the compatibility host.
//!
//! A view (`camera`, `flow`, `inventory`, the liquid words, the room words,
//! the cinematic and options object tables, the palette fade, the
//! [`crate::session_cohort`] copies) gives code outside a lease typed access
//! to Session-owned state. It names what it reads and stores by field
//! ([`Field`]) or by layout ([`Layout`], [`Decode`]), and the host
//! ([`Views`]) resolves them. The compatibility layer (`sm`) implements
//! [`Views`] for the Runtime over its lease and for `Game` over the installed
//! Session's projection or its RAM image. The byte layout stays with
//! `session_encoding` and the views' layouts; the byte access stays in `sm`.
//!
//! The `image_*` helpers address the work RAM image directly, for a view that
//! must not admit the owned Session (the enemy projectile table when no lease
//! can open).
use crate::cinematic_state as cinematic;
use crate::menus::options_objects::objects;
use crate::ram::{RawRam as _, WorkRamAccess};

/// A word a view reads or stores, named by its owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Flow(crate::flow::Word),
    Inventory(crate::inventory::Word),
    Cinematic(cinematic::Word),
    /// Word `index` of a cinematic table; like the named tables, the index
    /// may run past the table's end or before its start.
    CinematicElement(cinematic::Array, isize),
    OptionsObject(objects::Word),
    /// Word `index` of an options object table.
    OptionsObjectElement(objects::Array, usize),
    Room(crate::room_view::Word),
    RoomEffect(crate::room_view::EffectWord),
    /// Current colour `index` of the palette fade.
    PaletteColor(usize),
    /// Target colour `index` of the palette fade.
    PaletteTarget(usize),
    /// The scanline interrupt's previous and current CRE bit sets.
    PreviousCre,
    CurrentCre,
    /// Whether a door transition is under way.
    DoorTransition,
    /// The current room's header pointer.
    RoomHeader,
}

impl Field {
    /// Where the projection encodes the word.
    pub fn address(self) -> usize {
        use crate::ram::words;
        match self {
            Self::Flow(word) => crate::session_encoding::FLOW + 2 * word as usize,
            Self::Inventory(word) => word.address(),
            Self::Cinematic(word) => word as usize,
            Self::CinematicElement(array, index) => cinematic::at(array, index),
            Self::OptionsObject(word) => word as usize,
            Self::OptionsObjectElement(array, index) => array as usize + 2 * index,
            Self::Room(word) => crate::session_encoding::room_geometry_addresses()[word as usize],
            Self::RoomEffect(word) => crate::session_encoding::room_effect_address(word),
            Self::PaletteColor(index) => 0xc000 + 2 * index,
            Self::PaletteTarget(index) => 0xc200 + 2 * index,
            Self::PreviousCre => words::PREVIOUS_CRE_BITSET,
            Self::CurrentCre => words::CRE_BITSET,
            Self::DoorTransition => words::DOOR_TRANSITION_FLAG,
            Self::RoomHeader => words::ROOM_PTR,
        }
    }
}

/// A group of words laid out in the projection, visited in order.
pub trait Layout {
    /// Visit each word with where the projection encodes it.
    fn visit(&mut self, visit: impl FnMut(usize, &mut u16));
}

/// Every Session copy is laid out by its words.
impl<C: crate::session_cohort::Cohort> Layout for C {
    fn visit(&mut self, visit: impl FnMut(usize, &mut u16)) {
        self.words(visit);
    }
}

/// A value `session_encoding` decodes from the projection.
pub trait Decode {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self;
}

impl Decode for crate::camera::Camera {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self {
        crate::session_encoding::camera(bytes)
    }
}
impl Decode for crate::flow::Flow {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self {
        crate::session_encoding::flow(bytes)
    }
}
impl Decode for crate::room_view::RoomGeometry {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self {
        crate::session_encoding::room_geometry(bytes)
    }
}
impl Decode for crate::room_view::RoomEffects {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self {
        crate::session_encoding::room_effects(bytes)
    }
}

/// The host's projection of Session-owned state, by field and layout. Words
/// are read and stored low byte then high, as the named word accesses were.
pub trait Views {
    /// `field` as the host's projection holds it.
    fn view_word(&self, field: Field) -> u16;
    /// Store `value` to `field`.
    fn set_view_word(&mut self, field: Field, value: u16);
    /// Store `value` to `field`'s low byte only, as a byte store did.
    fn set_view_low_byte(&mut self, field: Field, value: u8);
    /// Decode `T` from the host's projection.
    fn decode_view<T: Decode>(&self) -> T;
    /// Load each of `words` from the host's projection.
    fn load_words(&self, words: &mut impl Layout);
    /// Store each of `words` whose index in visit order `select` accepts, in
    /// visit order.
    fn store_words(&mut self, words: &mut impl Layout, select: impl FnMut(usize) -> bool);
}

/// The little-endian word at `at` of the work RAM image.
pub fn image_word<H: WorkRamAccess + ?Sized>(h: &H, at: usize) -> u16 {
    let ram = h.ram();
    u16::from_le_bytes([ram.read_byte(at), ram.read_byte(at + 1)])
}

/// Store `value` as the little-endian word at `at` of the work RAM image,
/// low byte then high.
pub fn set_image_word<H: WorkRamAccess + ?Sized>(h: &H, at: usize, value: u16) {
    let [low, high] = value.to_le_bytes();
    let ram = h.ram();
    ram.write_byte(at, low);
    ram.write_byte(at + 1, high);
}

/// Copy `bytes` into the work RAM image at `at`.
pub fn copy_into_image<H: WorkRamAccess + ?Sized>(h: &H, at: usize, bytes: &[u8]) {
    h.ram().copy_in(at, bytes);
}

/// A host over a Session's projection, as the compat game's is while one is
/// installed, for the views' tests.
#[cfg(test)]
pub(crate) struct SessionProjection(pub crate::session::Session);

#[cfg(test)]
impl SessionProjection {
    fn projected(&self, at: usize) -> u8 {
        crate::session_encoding::projected_byte(&self.0, at).expect("view byte is projected")
    }
    fn project(&mut self, at: usize, value: u8) {
        assert!(crate::session_encoding::write_projected_byte(
            &mut self.0,
            at,
            value
        ));
    }
    fn projected_word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.projected(at), self.projected(at + 1)])
    }
    fn project_word(&mut self, at: usize, value: u16) {
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.project(at + lane, byte);
        }
    }
}

#[cfg(test)]
impl crate::display_scroll::Access for SessionProjection {
    fn scroll_word(&self, word: crate::display_scroll::Word) -> u16 {
        self.0
            .presentation
            .owner()
            .expect("provided Display")
            .state
            .controls
            .scroll_words[usize::from(word)]
    }
    fn set_scroll_word(&mut self, word: crate::display_scroll::Word, value: u16) {
        self.0
            .presentation
            .owner_mut()
            .expect("provided Display")
            .state
            .controls
            .scroll_words[usize::from(word)] = value;
    }
    fn physical_scroll_byte(&self, _: crate::display_scroll::Word, _: bool) -> u8 {
        panic!("physical image was not supplied")
    }
    fn write_physical_scroll_byte(&mut self, _: crate::display_scroll::Word, _: bool, _: u8) {
        panic!("physical image was not supplied")
    }
}
#[cfg(test)]
impl crate::sequence_state::Access for SessionProjection {
    fn set_cinematic_function(&mut self, value: u16) {
        self.project_word(0x1f51, value);
    }
    fn set_sequence_word(&mut self, word: crate::session_words::Word, value: u16) {
        self.project_word(word as usize, value);
    }
    fn sequence_flow(&self) -> crate::flow::Flow {
        self.0.flow
    }
    fn sequence_words(&self) -> crate::session_words::SessionWords {
        crate::session_words::SessionWords::from_session(&self.0)
    }
    fn commit_flow(&mut self, before: &crate::flow::Flow, after: crate::flow::Flow) {
        after.apply_changed(before, &mut self.0.flow);
    }
    fn commit_sequence_words(
        &mut self,
        before: &crate::session_words::SessionWords,
        after: crate::session_words::SessionWords,
    ) {
        after.apply_changed(before, &mut self.0);
    }
}

#[cfg(test)]
impl crate::ram::CompatibilityRam for SessionProjection {
    fn compatibility_byte(&self, at: usize) -> u8 {
        self.projected(at)
    }
    fn compatibility_write_byte(&mut self, at: usize, value: u8) {
        self.project(at, value);
    }
    fn observed_inputs(&self) -> crate::external_inputs::ExternalInputs {
        crate::session_encoding::inputs(&|at| self.projected(at))
    }
}

#[cfg(test)]
impl Views for SessionProjection {
    fn view_word(&self, field: Field) -> u16 {
        self.projected_word(field.address())
    }
    fn set_view_word(&mut self, field: Field, value: u16) {
        self.project_word(field.address(), value);
    }
    fn set_view_low_byte(&mut self, field: Field, value: u8) {
        self.project(field.address(), value);
    }
    fn decode_view<T: Decode>(&self) -> T {
        T::decode(&|at| self.projected(at))
    }
    fn load_words(&self, words: &mut impl Layout) {
        words.visit(|at, word| *word = self.projected_word(at));
    }
    fn store_words(&mut self, words: &mut impl Layout, mut select: impl FnMut(usize) -> bool) {
        let mut index = 0;
        words.visit(|at, word| {
            if select(index) {
                self.project_word(at, *word);
            }
            index += 1;
        });
    }
}

#[cfg(test)]
impl crate::gameplay_context::Access for SessionProjection {
    fn inventory_state(&self) -> crate::inventory::Inventory {
        self.0.inventory.clone()
    }
    fn commit_inventory(
        &mut self,
        before: &crate::inventory::Inventory,
        mut after: crate::inventory::Inventory,
    ) {
        use crate::session_cohort::Cohort as _;
        let mut old = Vec::new();
        before.clone().words(|_, value| old.push(*value));
        let mut changes = Vec::new();
        let mut index = 0;
        after.words(|at, value| {
            if old[index] != *value {
                changes.push((at, *value));
            }
            index += 1;
        });
        for (at, value) in changes {
            self.project_word(at, value);
        }
    }

    fn gameplay_word(&self, word: crate::gameplay_context::Word) -> u16 {
        word.get_session(&self.0)
    }
    fn set_gameplay_byte(&mut self, word: crate::gameplay_context::Word, lane: usize, byte: u8) {
        assert!(lane < 2);
        let field = word.get_session_mut(&mut self.0);
        let mut bytes = field.to_le_bytes();
        bytes[lane] = byte;
        *field = u16::from_le_bytes(bytes);
    }
}

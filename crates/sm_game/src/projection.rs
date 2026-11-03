//! What sm_game's typed views require of the compatibility host.
//!
//! A view (`camera`, `flow`, `inventory`, the liquid words, the room words,
//! the cinematic and options object tables, the palette fade, the
//! [`crate::session_cohort`] copies) gives code outside a lease typed access
//! to Session-owned state. It names what it reads and stores by field
//! ([`Field`]) or by layout ([`Layout`], [`Decode`]), and the host
//! ([`Views`]) resolves them. Each field, layout and decoded value also names
//! its Session fields ([`Field::session_word`], [`Layout::session_copy`],
//! [`Decode::from_session`]); the tests below prove they are the words the
//! projection encodes. The compatibility layer (`sm`) implements [`Views`]
//! for the Runtime, which reads and stores those Session fields while its
//! lease is current, and for `Game` over the installed Session's projection
//! or its RAM image. The byte layout stays with `session_encoding` and the
//! views' layouts; the byte access stays in `sm`.
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

    /// The word as `session` holds it: the field the projection encodes at
    /// [`Self::address`]. `None` where the Session has no word there (a
    /// table element past its table), which the host's projection resolves.
    pub fn session_word(self, session: &Session) -> Option<u16> {
        let (room, entry) = (&session.room, &session.room.entry);
        let visual = session.presentation.visual();
        Some(match self {
            Self::Flow(word) => session.flow.get(word),
            Self::Inventory(word) => session.inventory.get(word),
            Self::Cinematic(word) => return cinematic_word(session, word),
            Self::CinematicElement(array, index) => {
                return cinematic_element(session, array, index)
                    .or_else(|| session.enemy_projectiles.get(self.address()));
            }
            Self::OptionsObject(_) | Self::OptionsObjectElement(..) => {
                return session.enemy_projectiles.get(self.address());
            }
            Self::Room(word) => match word {
                RoomWord::Area => room.area,
                RoomWord::Width => room.width,
                RoomWord::Height => room.height,
                RoomWord::ScrollWidth => room.scroll_width,
                RoomWord::ScrollHeight => room.scroll_height,
            },
            Self::RoomEffect(word) => match word {
                EffectWord::EarthquakeKind => session.feedback.earthquake_kind,
                EffectWord::EarthquakeTimer => session.feedback.earthquake_timer,
                EffectWord::MapX => entry.map_x,
                EffectWord::MapY => entry.map_y,
                EffectWord::ElevatorProperties => entry.elevator_properties,
            },
            Self::PaletteColor(index) => *visual.colors.get(index)?,
            Self::PaletteTarget(index) => *visual.targets.get(index)?,
            Self::PreviousCre => entry.previous_cre,
            Self::CurrentCre => entry.cre,
            Self::DoorTransition => entry.completion,
            Self::RoomHeader => session.world.room_key(&entry.room),
        })
    }

    /// The Session field that holds the word, for a store. `None` where the
    /// Session has no word there, and where it holds no plain word: the room
    /// header and the door exit, which the projection stores a byte at a
    /// time through the world's keys, and the FX blending control lane.
    pub fn session_word_mut(self, session: &mut Session) -> Option<&mut u16> {
        Some(match self {
            Self::Flow(word) => session.flow.get_mut(word),
            Self::Inventory(word) => session.inventory.get_mut(word),
            Self::Cinematic(word) => return cinematic_word_mut(session, word),
            Self::CinematicElement(array, index) => {
                if cinematic_element(session, array, index).is_some() {
                    return cinematic_element_mut(session, array, index);
                }
                return session.enemy_projectiles.word_at(self.address());
            }
            Self::OptionsObject(_) | Self::OptionsObjectElement(..) => {
                return session.enemy_projectiles.word_at(self.address());
            }
            Self::Room(word) => match word {
                RoomWord::Area => &mut session.room.area,
                RoomWord::Width => &mut session.room.width,
                RoomWord::Height => &mut session.room.height,
                RoomWord::ScrollWidth => &mut session.room.scroll_width,
                RoomWord::ScrollHeight => &mut session.room.scroll_height,
            },
            Self::RoomEffect(word) => match word {
                EffectWord::EarthquakeKind => &mut session.feedback.earthquake_kind,
                EffectWord::EarthquakeTimer => &mut session.feedback.earthquake_timer,
                EffectWord::MapX => &mut session.room.entry.map_x,
                EffectWord::MapY => &mut session.room.entry.map_y,
                EffectWord::ElevatorProperties => &mut session.room.entry.elevator_properties,
            },
            Self::PaletteColor(index) => session.presentation.visual_mut().colors.get_mut(index)?,
            Self::PaletteTarget(index) => {
                session.presentation.visual_mut().targets.get_mut(index)?
            }
            Self::PreviousCre => &mut session.room.entry.previous_cre,
            Self::CurrentCre => &mut session.room.entry.cre,
            Self::DoorTransition => &mut session.room.entry.completion,
            Self::RoomHeader => return None,
        })
    }
}

use crate::cinematic_state::{Array as CinematicArray, Word as CinematicWord};
use crate::room_view::{EffectWord, Word as RoomWord};
use crate::session::Session;

/// Cinematic `word`'s Session field: a named word outside the enemy
/// projectile table, else the table's word at its address.
fn cinematic_word(session: &Session, word: CinematicWord) -> Option<u16> {
    let (player, controls) = (&session.player, &session.projectiles.controls);
    Some(match word {
        CinematicWord::JapaneseText => session.options.japanese_text,
        CinematicWord::Layer1X => session.room.camera.layer1[0],
        CinematicWord::Layer1Y => session.room.camera.layer1[1],
        CinematicWord::DoorExit => session.world.exit_key(&session.room.entry.exit),
        CinematicWord::FxBlending => crate::fx_encoding::control_word(session, word as usize),
        CinematicWord::SelectedWeapon => controls.selected_weapon,
        CinematicWord::AutomaticWeapon => controls.automatic_weapon,
        CinematicWord::PowerBombX => session.projectiles.power_bomb.x,
        CinematicWord::PowerBombY => session.projectiles.power_bomb.y,
        CinematicWord::SequenceStage => session.shared_sequence.stage,
        CinematicWord::SequenceValue => session.shared_sequence.value,
        CinematicWord::SamusPose => player.pose,
        CinematicWord::KnockbackXDirection => player.knockback_x_direction,
        CinematicWord::SamusYSubspeed => player.contact.y_speed.fraction,
        CinematicWord::SamusYSpeed => player.contact.y_speed.whole,
        _ => return session.enemy_projectiles.get(word as usize),
    })
}

/// [`cinematic_word`]'s field for a store; `None` for the door exit key and
/// the FX blending control lane.
fn cinematic_word_mut(session: &mut Session, word: CinematicWord) -> Option<&mut u16> {
    let player = &mut session.player;
    let controls = &mut session.projectiles.controls;
    Some(match word {
        CinematicWord::DoorExit | CinematicWord::FxBlending => return None,
        CinematicWord::JapaneseText => &mut session.options.japanese_text,
        CinematicWord::Layer1X => &mut session.room.camera.layer1[0],
        CinematicWord::Layer1Y => &mut session.room.camera.layer1[1],
        CinematicWord::SelectedWeapon => &mut controls.selected_weapon,
        CinematicWord::AutomaticWeapon => &mut controls.automatic_weapon,
        CinematicWord::PowerBombX => &mut session.projectiles.power_bomb.x,
        CinematicWord::PowerBombY => &mut session.projectiles.power_bomb.y,
        CinematicWord::SequenceStage => &mut session.shared_sequence.stage,
        CinematicWord::SequenceValue => &mut session.shared_sequence.value,
        CinematicWord::SamusPose => &mut player.pose,
        CinematicWord::KnockbackXDirection => &mut player.knockback_x_direction,
        CinematicWord::SamusYSubspeed => &mut player.contact.y_speed.fraction,
        CinematicWord::SamusYSpeed => &mut player.contact.y_speed.whole,
        _ => return session.enemy_projectiles.word_at(word as usize),
    })
}

/// Word `index` of a cinematic table outside the enemy projectile table,
/// when the index is within it.
fn cinematic_element(session: &Session, array: CinematicArray, index: isize) -> Option<u16> {
    let index = usize::try_from(index).ok()?;
    let shots = &session.projectiles.shots;
    match array {
        CinematicArray::PaletteBuffer => session.presentation.visual().colors.get(index).copied(),
        CinematicArray::TilemapStuff => fade_plane(&session.actors.extended.blocks, index),
        CinematicArray::ProjectileX => shots.get(index).map(|shot| shot.x),
        CinematicArray::ProjectileType => shots.get(index).map(|shot| shot.kind),
        _ => None,
    }
}

/// [`cinematic_element`]'s field for a store.
fn cinematic_element_mut(
    session: &mut Session,
    array: CinematicArray,
    index: isize,
) -> Option<&mut u16> {
    let index = usize::try_from(index).ok()?;
    let shots = &mut session.projectiles.shots;
    match array {
        CinematicArray::PaletteBuffer => session.presentation.visual_mut().colors.get_mut(index),
        CinematicArray::TilemapStuff => {
            let (block, word) = fade_plane_word(index)?;
            Some(&mut session.actors.extended.blocks[block][word])
        }
        CinematicArray::ProjectileX => shots.get_mut(index).map(|shot| &mut shot.x),
        CinematicArray::ProjectileType => shots.get_mut(index).map(|shot| &mut shot.kind),
        _ => None,
    }
}

/// The actor block and word of fade plane word `index`: the fade planes are
/// actor blocks 4 and 5 ([`crate::cinematic::FadePlanes`]).
fn fade_plane_word(index: usize) -> Option<(usize, usize)> {
    (index < 2048).then_some((4 + index / 1024, index % 1024))
}

fn fade_plane(blocks: &[[u16; 1024]; 6], index: usize) -> Option<u16> {
    let (block, word) = fade_plane_word(index)?;
    Some(blocks[block][word])
}

/// A group of words laid out in the projection, visited in order.
pub trait Layout: Sized {
    /// Visit each word with where the projection encodes it.
    fn visit(&mut self, visit: impl FnMut(usize, &mut u16));
    /// The group as `session` holds it: each word `visit` visits holds the
    /// Session field the projection encodes there.
    fn session_copy(session: &Session) -> Self;
    /// Store the words `selected` marks, by visit index, to their Session
    /// fields, as plain stores of each word's bytes do; false, storing
    /// nothing, for a group whose words are not all plain Session fields.
    fn store_session(&mut self, session: &mut Session, selected: &[bool]) -> bool {
        let _ = (session, selected);
        false
    }
}

/// Every Session copy is laid out by its words. Their stores go through
/// `session_cohort::store`, which stores a retained Session's fields
/// directly on the compat game's lease.
impl<C: crate::session_cohort::Cohort> Layout for C {
    fn visit(&mut self, visit: impl FnMut(usize, &mut u16)) {
        self.words(visit);
    }
    fn session_copy(session: &Session) -> Self {
        C::session_copy(session)
    }
}

/// Copy into `to` each word of `from` that `selected` accepts, by visit
/// index.
pub fn copy_words<L: Layout>(from: &mut L, to: &mut L, selected: impl Fn(usize) -> bool) {
    let mut values = Vec::new();
    from.visit(|_, word| values.push(*word));
    let mut index = 0;
    to.visit(|_, word| {
        if selected(index) {
            *word = values[index];
        }
        index += 1;
    });
}

/// Load each of `words` from the Session field the projection encodes it
/// from.
pub fn load_session_words<L: Layout>(words: &mut L, session: &Session) {
    copy_words(&mut L::session_copy(session), words, |_| true);
}

/// A value `session_encoding` decodes from the projection, and the Session
/// fields it copies.
pub trait Decode {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self;
    /// The value as `session` holds it; equal to [`Self::decode`] of its
    /// projection.
    fn from_session(session: &Session) -> Self;
}

impl Decode for crate::camera::Camera {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self {
        crate::session_encoding::camera(bytes)
    }
    fn from_session(session: &Session) -> Self {
        session.room.camera
    }
}
impl Decode for crate::flow::Flow {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self {
        crate::session_encoding::flow(bytes)
    }
    fn from_session(session: &Session) -> Self {
        session.flow
    }
}
impl Decode for crate::room_view::RoomGeometry {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self {
        crate::session_encoding::room_geometry(bytes)
    }
    fn from_session(session: &Session) -> Self {
        let room = &session.room;
        Self {
            area: room.area,
            width: room.width,
            height: room.height,
            scroll_width: room.scroll_width,
            scroll_height: room.scroll_height,
        }
    }
}
impl Decode for crate::room_view::RoomEffects {
    fn decode(bytes: &impl Fn(usize) -> u8) -> Self {
        crate::session_encoding::room_effects(bytes)
    }
    fn from_session(session: &Session) -> Self {
        let entry = &session.room.entry;
        Self {
            earthquake_kind: session.feedback.earthquake_kind,
            earthquake_timer: session.feedback.earthquake_timer,
            map_x: entry.map_x,
            map_y: entry.map_y,
            elevator_properties: entry.elevator_properties,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_encoding::{projected_byte, write_projected_byte};
    use crate::session_owners::Presentation;

    fn session() -> Session {
        Session::with_world(crate::room_entry::world::WorldBinding::canonical_legacy())
    }

    fn projected_word(session: &Session, at: usize) -> u16 {
        let byte = |at| projected_byte(session, at).expect("view byte is projected");
        u16::from_le_bytes([byte(at), byte(at + 1)])
    }

    /// Every word a view names, with the table elements on and past each
    /// table's edges.
    fn fields() -> Vec<Field> {
        use crate::cinematic_state::{Array as A, Word as W};
        use crate::flow::Word as FlowWord;
        use crate::inventory::Word as I;
        use crate::menus::options_objects::objects::{Array as OA, Word as OW};
        let mut fields: Vec<Field> = [
            FlowWord::CinematicFunction,
            FlowWord::DemoTimer,
            FlowWord::DemoSet,
            FlowWord::DemoScene,
            FlowWord::NumDemoSets,
        ]
        .into_iter()
        .map(Field::Flow)
        .collect();
        fields.extend(
            [
                I::EquippedItems,
                I::CollectedItems,
                I::EquippedBeams,
                I::CollectedBeams,
                I::ReserveMode,
                I::Health,
                I::MaxHealth,
                I::Missiles,
                I::MaxMissiles,
                I::SuperMissiles,
                I::MaxSuperMissiles,
                I::PowerBombs,
                I::MaxPowerBombs,
                I::MaxReserveHealth,
                I::ReserveHealth,
                I::ReserveMissiles,
                I::SubunitHealth,
                I::HyperBeam,
            ]
            .into_iter()
            .map(Field::Inventory),
        );
        fields.extend(
            [
                W::Mode7Angle,
                W::Mode7Zoom,
                W::XSub,
                W::X,
                W::YSub,
                W::Y,
                W::XSubspeed,
                W::XSpeed,
                W::YSubspeed,
                W::YSpeed,
                W::Mode7ObjectIndex,
                W::Mode7SpawnParameter,
                W::BgObjectIndex,
                W::EnableObjects,
                W::EnableBgTilemap,
                W::BgVramAddress,
                W::CreditsInstructionList,
                W::CreditsInstructionTimer,
                W::CreditsTimer,
                W::CreditsPreInstruction,
                W::CreditsEnabled,
                W::CreditsTilemapRow,
                W::LastCreditsUpdateY,
                W::ShootingStarsEnabled,
                W::TextGlowObjectIndex,
                W::FunctionTimer,
                W::Var13,
                W::Var14,
                W::Var17,
                W::FrameCounter,
                W::SkipToTitleIndex,
                W::IntroSamusDisplay,
                W::ObjectIndex,
                W::SpawnParameter,
                W::IntroFrameCounter,
                W::IntroTextClickFlag,
                W::IntroJapaneseTextTimer,
                W::JapaneseText,
                W::Layer1X,
                W::Layer1Y,
                W::DoorExit,
                W::FxBlending,
                W::SelectedWeapon,
                W::AutomaticWeapon,
                W::PowerBombX,
                W::PowerBombY,
                W::SequenceStage,
                W::SequenceValue,
                W::SamusPose,
                W::KnockbackXDirection,
                W::SamusYSubspeed,
                W::SamusYSpeed,
            ]
            .into_iter()
            .map(Field::Cinematic),
        );
        let tables = [
            (A::Mode7InstructionPointer, 2),
            (A::Mode7PreInstruction, 2),
            (A::Mode7InstructionTimer, 2),
            (A::Mode7GotoTimer, 2),
            (A::BgArr1, 3),
            (A::BgInstructionPointer, 4),
            (A::BgPreInstruction, 4),
            (A::BgInstructionTimer, 4),
            (A::BgArr6, 4),
            (A::TextGlowPointers, 8),
            (A::TextGlowTimers, 8),
            (A::TextGlowXPositions, 8),
            (A::TextGlowYPositions, 8),
            (A::TextGlowPalettes, 8),
            (A::SprWhatToDraw, 16),
            (A::BgArr7, 16),
            (A::BgArr8, 16),
            (A::BgArr9, 16),
            (A::SprArr6, 16),
            (A::SprArr7, 16),
            (A::SprInstructionPointer, 16),
            (A::SprPreInstruction, 16),
            (A::SprInstructionTimer, 16),
            (A::SprGotoTimer, 16),
            (A::ProjectileX, 10),
            (A::ProjectileType, 10),
        ];
        for (array, len) in tables {
            fields.extend((-2..len + 2).map(|index| Field::CinematicElement(array, index)));
        }
        let edges = [(A::PaletteBuffer, 0..4), (A::PaletteBuffer, 252..258)];
        let planes = [(A::TilemapStuff, 0..4), (A::TilemapStuff, 1020..1028)];
        let end = [(A::TilemapStuff, 2044..2050)];
        for (array, indices) in edges.into_iter().chain(planes).chain(end) {
            fields.extend(indices.map(|index| Field::CinematicElement(array, index)));
        }
        fields.extend([OW::Index, OW::InitParameter].map(Field::OptionsObject));
        for array in [
            OA::Spritemaps,
            OA::XPositions,
            OA::YPositions,
            OA::Palettes,
            OA::UnusedA,
            OA::UnusedB,
            OA::InstructionLists,
            OA::PreInstructions,
            OA::InstructionTimers,
            OA::Timers,
            OA::ControllerBindings,
        ] {
            fields.extend((0..10).map(|index| Field::OptionsObjectElement(array, index)));
        }
        {
            use crate::room_view::{EffectWord as E, Word as R};
            fields.extend(
                [
                    R::Area,
                    R::Width,
                    R::Height,
                    R::ScrollWidth,
                    R::ScrollHeight,
                ]
                .map(Field::Room),
            );
            fields.extend(
                [
                    E::EarthquakeKind,
                    E::EarthquakeTimer,
                    E::MapX,
                    E::MapY,
                    E::ElevatorProperties,
                ]
                .map(Field::RoomEffect),
            );
        }
        fields.extend((0..258).map(Field::PaletteColor));
        fields.extend((0..258).map(Field::PaletteTarget));
        fields.extend([
            Field::PreviousCre,
            Field::CurrentCre,
            Field::DoorTransition,
            Field::RoomHeader,
        ]);
        fields
    }

    /// Each layout loaded from the Session is its projected words, and a
    /// layout that stores its Session fields leaves the Session as the
    /// projected byte stores of the words it selects do.
    fn check_layout<L: Layout + Clone + Default + PartialEq + core::fmt::Debug>() {
        let mut session = session();
        let mut addresses = Vec::new();
        L::default().visit(|at, _| addresses.push(at));
        // FX control lanes read and publish through the control topology,
        // which a bare Session lacks; layouts holding them are covered by
        // replays instead.
        if addresses
            .iter()
            .any(|&at| (at..at + 2).any(|byte| crate::fx_encoding::control_lane(byte).is_some()))
        {
            return;
        }
        let value = |index: usize, salt: u16| {
            0x2468u16.wrapping_add(salt.wrapping_mul(0x0705) ^ (index as u16).wrapping_mul(0x0b0b))
        };
        for (index, &at) in addresses.iter().enumerate() {
            for (lane, byte) in value(index, 1).to_le_bytes().into_iter().enumerate() {
                assert!(
                    write_projected_byte(&mut session, at + lane, byte),
                    "{at:#x}"
                );
            }
        }
        let mut direct = L::default();
        load_session_words(&mut direct, &session);
        let mut projected = L::default();
        projected.visit(|at, word| *word = projected_word(&session, at));
        assert_eq!(direct, projected, "loads");

        let mut words = L::default();
        let mut index = 0;
        words.visit(|_, word| {
            *word = value(index, 2);
            index += 1;
        });
        let count = addresses.len();
        let all = vec![true; count];
        let alternate: Vec<bool> = (0..count).map(|index| index % 2 == 0).collect();
        let last: Vec<bool> = (0..count).map(|index| index + 1 == count).collect();
        for selected in [all, alternate, last, vec![false; count]] {
            let mut direct = session.clone();
            if !words.clone().store_session(&mut direct, &selected) {
                assert_eq!(direct, session, "an unsupported store stores nothing");
                continue;
            }
            let mut projected = session.clone();
            let mut stored = words.clone();
            let mut index = 0;
            stored.visit(|at, word| {
                if selected[index] {
                    for (lane, byte) in word.to_le_bytes().into_iter().enumerate() {
                        assert!(write_projected_byte(&mut projected, at + lane, byte));
                    }
                }
                index += 1;
            });
            assert!(direct == projected, "stores {selected:?}");
        }
    }

    #[test]
    fn layouts_are_their_session_fields() {
        check_layout::<crate::liquid::LiquidWords>();
        check_layout::<crate::visual_state::PaletteFade>();
        check_layout::<crate::flow::Flow>();
        check_layout::<crate::inventory::Inventory>();
        check_layout::<crate::menu_state::MenuState>();
        check_layout::<crate::menu_state::MenuScreens>();
        check_layout::<crate::session_words::SessionWords>();
        check_layout::<crate::cinematic_state::Cinematic>();
        check_layout::<crate::cinematic::FadePlanes>();
        check_layout::<crate::cinematic::PaletteBuffer>();
        check_layout::<crate::cinematic::ShotWords>();
        check_layout::<crate::menu_window_state::WindowState>();
        check_layout::<(
            crate::menu_state::OptionsMenu,
            crate::session_words::SessionWords,
        )>();
    }

    /// Each decoded value copied from the Session is its projection's decode.
    fn check_decode<T: Decode + PartialEq + core::fmt::Debug>() {
        let mut session = session();
        let addresses = core::cell::RefCell::new(Vec::new());
        T::decode(&|at| {
            addresses.borrow_mut().push(at);
            0
        });
        for (index, at) in addresses.into_inner().into_iter().enumerate() {
            let value = 0x31u8.wrapping_add((index as u8).wrapping_mul(0x1d));
            assert!(write_projected_byte(&mut session, at, value), "{at:#x}");
        }
        let decoded = T::decode(&|at| projected_byte(&session, at).expect("decoded byte"));
        assert_eq!(T::from_session(&session), decoded);
    }

    #[test]
    fn decoded_views_are_their_session_fields() {
        check_decode::<crate::camera::Camera>();
        check_decode::<crate::flow::Flow>();
        check_decode::<crate::room_view::RoomGeometry>();
        check_decode::<crate::room_view::RoomEffects>();
    }

    /// A Session field read directly is the word the projection decodes at
    /// the field's address, and storing it directly (a word, or its low
    /// byte) leaves the Session exactly as the projected byte stores did.
    #[test]
    fn view_words_are_their_projected_words() {
        let base = session();
        let mut probe = base.clone();
        for (index, field) in fields().into_iter().enumerate() {
            let at = field.address();
            let stores = field.session_word_mut(&mut probe).is_some();
            let always = matches!(
                field,
                Field::Flow(_)
                    | Field::Inventory(_)
                    | Field::Room(_)
                    | Field::RoomEffect(_)
                    | Field::PreviousCre
                    | Field::CurrentCre
                    | Field::DoorTransition
            ) || matches!(field, Field::PaletteColor(i) | Field::PaletteTarget(i) if i < 256);
            assert!(stores || !always, "{field:?} has no Session field");
            let Some(word) = field.session_word(&base) else {
                assert!(!stores, "{field:?} stores without a read");
                continue;
            };
            assert_eq!(word, projected_word(&base, at), "{field:?} reads");
            if !stores {
                continue;
            }
            for lane in [at, at + 1] {
                assert!(
                    !Presentation::requires_semantic_counterpart(lane),
                    "{field:?} marks the presentation"
                );
            }
            let value = 0x1357u16.wrapping_add((index as u16).wrapping_mul(0x0b0d));
            let [low, high] = value.to_le_bytes();
            let mut projected = base.clone();
            assert!(write_projected_byte(&mut projected, at, low), "{field:?}");
            let low_only = projected.clone();
            assert!(
                write_projected_byte(&mut projected, at + 1, high),
                "{field:?}"
            );

            let mut direct = base.clone();
            *field.session_word_mut(&mut direct).expect("stored") = value;
            assert!(direct == projected, "{field:?} stores its word");
            assert_eq!(field.session_word(&direct), Some(value), "{field:?}");
            assert_eq!(projected_word(&direct, at), value, "{field:?}");

            let mut direct = base.clone();
            let word = field.session_word_mut(&mut direct).expect("stored");
            *word = (*word & 0xff00) | u16::from(low);
            assert!(direct == low_only, "{field:?} stores its low byte");
        }
    }
}

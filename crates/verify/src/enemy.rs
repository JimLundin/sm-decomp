//! The enemy slots, decoded from the original's `enemy_data` table (bank $A0 log, `$0F78`).
//!
//! The spritemap and instruction pointers (`+16`, `+1A`) are in the slot's AI bank (`+2E`),
//! which the species header also names; they decode through the content's programs.

use data::enemy::program::{EnemyPrograms, Spritemap, SpritemapId};
use data::enemy::{Behaviour, SpeciesKey, SpeciesMap};
use data::id::SpeciesId;
use data::rom::RomAddress;
use game::enemy::instructions::label;
use game::state::room::{
    AiVariables, ENEMY_SLOTS, Elevator, EnemyBody, EnemyCommon, EyePhase, MorphBallEye,
    PaletteIndex, TilesIndex,
};
use game::state::samus::{Fixed, Point};
use game::state::slots::{Cursor, FlagSet, Frames, Generation, Label, Occupancy, Slot, SlotStore};

use crate::decode::DecodeError;
use crate::source::{self, SlotField, WramTable};

/// The properties word's deleted bit (bank $A0 log): the next determine pass frees the slot.
const DELETED: u16 = 0x0200;

/// Where the original keeps each enemy slot's fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnemySources {
    /// The slot table.
    pub table: WramTable,
    /// `.common.species` and the occupancy.
    pub species: SlotField,
    /// `.common.position.x.pixels`.
    pub x: SlotField,
    /// `.common.position.x.subpixels`.
    pub x_subpixels: SlotField,
    /// `.common.position.y.pixels`.
    pub y: SlotField,
    /// `.common.position.y.subpixels`.
    pub y_subpixels: SlotField,
    /// `.common.radius_x`.
    pub radius_x: SlotField,
    /// `.common.radius_y`.
    pub radius_y: SlotField,
    /// `.common.properties`, and whether the occupant is deleting.
    pub properties: SlotField,
    /// `.common.extra_properties`.
    pub extra_properties: SlotField,
    /// `.common.ai_handlers`.
    pub ai_handlers: SlotField,
    /// `.common.health`.
    pub health: SlotField,
    /// `.common.timer`.
    pub timer: SlotField,
    /// `.common.spritemap`.
    pub spritemap: SlotField,
    /// `.common.cursor.at`.
    pub instruction: SlotField,
    /// `.common.cursor.delay`.
    pub instruction_timer: SlotField,
    /// The AI bank the pointers are in.
    pub bank: SlotField,
    /// `.common.palette`.
    pub palette: SlotField,
    /// `.common.tiles`.
    pub tiles: SlotField,
    /// `.common.layer`.
    pub layer: SlotField,
    /// `.common.flash`.
    pub flash: SlotField,
    /// `.common.frozen`.
    pub frozen: SlotField,
    /// `.common.invincibility`.
    pub invincibility: SlotField,
    /// `.common.shake`.
    pub shake: SlotField,
    /// `.common.frame_counter`.
    pub frame_counter: SlotField,
    /// The body's AI words, in order.
    pub variables: [SlotField; 6],
    /// `.common.parameter_1`.
    pub parameter_1: SlotField,
    /// `.common.parameter_2`.
    pub parameter_2: SlotField,
}

impl EnemySources {
    /// The original's layout.
    pub const ORIGINAL: Self = Self {
        table: source::ENEMY_DATA,
        species: source::ENEMY_SPECIES,
        x: source::ENEMY_X_POS,
        x_subpixels: source::ENEMY_X_SUBPOS,
        y: source::ENEMY_Y_POS,
        y_subpixels: source::ENEMY_Y_SUBPOS,
        radius_x: source::ENEMY_X_RADIUS,
        radius_y: source::ENEMY_Y_RADIUS,
        properties: source::ENEMY_PROPERTIES,
        extra_properties: source::ENEMY_EXTRA_PROPERTIES,
        ai_handlers: source::ENEMY_AI_HANDLER_BITS,
        health: source::ENEMY_HEALTH,
        timer: source::ENEMY_TIMER,
        spritemap: source::ENEMY_SPRITEMAP,
        instruction: source::ENEMY_INSTRUCTION,
        instruction_timer: source::ENEMY_INSTRUCTION_TIMER,
        bank: source::ENEMY_BANK,
        palette: source::ENEMY_PALETTE_INDEX,
        tiles: source::ENEMY_VRAM_TILES_INDEX,
        layer: source::ENEMY_LAYER,
        flash: source::ENEMY_FLASH_TIMER,
        frozen: source::ENEMY_FROZEN_TIMER,
        invincibility: source::ENEMY_INVINCIBILITY_TIMER,
        shake: source::ENEMY_SHAKE_TIMER,
        frame_counter: source::ENEMY_FRAME_COUNTER,
        variables: source::ENEMY_AI_VARIABLES,
        parameter_1: source::ENEMY_PARAMETER_1,
        parameter_2: source::ENEMY_PARAMETER_2,
    };

    /// Every slot in `wram`, its species resolved through `species`.
    ///
    /// The generation stays at its default: the original keeps no fill count.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a species word `species` doesn't hold, or an
    /// occupied slot's instruction or spritemap pointer `programs` doesn't, and
    /// [`DecodeError::WramTooShort`] when a slot lies past the record's work RAM.
    pub fn read(
        &self,
        wram: &[u8],
        species: &SpeciesMap,
        programs: &EnemyPrograms,
    ) -> Result<SlotStore<EnemyCommon, EnemyBody, ENEMY_SLOTS>, DecodeError> {
        let mut store = SlotStore::default();
        for (slot, index) in store.slots.iter_mut().zip(0_u16..) {
            *slot = self.slot(wram, index, species, programs)?;
        }
        Ok(store)
    }

    /// The occupant of slot `index`, a species with `behaviour` and `properties`: its body
    /// from its AI words, deleting if the properties say so.
    fn occupant(
        &self,
        wram: &[u8],
        index: u16,
        behaviour: Behaviour,
        properties: u16,
    ) -> Result<Occupancy<EnemyBody>, DecodeError> {
        let word = |field: SlotField| self.table.word(field, index)?.read(wram);
        let [first, second, third, fourth, fifth, sixth] = self.variables;
        let variables = AiVariables {
            a: word(first)?,
            b: word(second)?,
            c: word(third)?,
            d: word(fourth)?,
            e: word(fifth)?,
            f: word(sixth)?,
        };
        let body = body(behaviour, variables).map_err(|word| DecodeError::Unrepresentable {
            source: self
                .variables
                .get(word.index)
                .map_or(self.species.name, |field| field.name),
            value: word.value,
        })?;
        Ok(if properties & DELETED == 0 {
            Occupancy::Live(body)
        } else {
            Occupancy::Deleting(body)
        })
    }

    /// Slot `index`.
    ///
    /// A free slot's pointers stay at their defaults: they are the last occupant's, which
    /// the next spawn writes over, and its bank may name no program.
    fn slot(
        &self,
        wram: &[u8],
        index: u16,
        species: &SpeciesMap,
        programs: &EnemyPrograms,
    ) -> Result<Slot<EnemyCommon, EnemyBody>, DecodeError> {
        let word = |field: SlotField| self.table.word(field, index)?.read(wram);
        let key = word(self.species)?;
        let occupant = match key {
            0 => None,
            key => Some(
                species
                    .get(SpeciesKey(key))
                    .ok_or(DecodeError::Unrepresentable {
                        source: self.species.name,
                        value: key,
                    })?,
            ),
        };
        let properties = word(self.properties)?;
        let occupancy = match occupant {
            None => Occupancy::Free,
            Some((_, behaviour)) => self.occupant(wram, index, behaviour, properties)?,
        };
        let fixed = |pixels: SlotField, subpixels: SlotField| -> Result<_, DecodeError> {
            Ok(Fixed {
                pixels: word(pixels)?,
                subpixels: word(subpixels)?,
            })
        };
        let frames = |field: SlotField| word(field).map(Frames);
        let [bank, _] = word(self.bank)?.to_le_bytes();
        let pointer = |field: SlotField| -> Result<(RomAddress, SlotField, u16), DecodeError> {
            let value = word(field)?;
            Ok((RomAddress::new(bank, value), field, value))
        };
        let unrepresentable =
            |(_, field, value): (RomAddress, SlotField, u16)| DecodeError::Unrepresentable {
                source: field.name,
                value,
            };
        let (at, spritemap) = if occupant.is_some() {
            let at = pointer(self.instruction)?;
            let shown = pointer(self.spritemap)?;
            (
                label(programs.target(at.0).ok_or_else(|| unrepresentable(at))?),
                programs
                    .spritemap_id(Spritemap(shown.0))
                    .ok_or_else(|| unrepresentable(shown))?,
            )
        } else {
            (Label::default(), SpritemapId::default())
        };
        let cursor = Cursor {
            at,
            delay: frames(self.instruction_timer)?,
            ..Cursor::default()
        };
        Ok(Slot {
            generation: Generation::default(),
            common: EnemyCommon {
                species: occupant.map_or_else(SpeciesId::default, |(id, _)| id),
                position: Point {
                    x: fixed(self.x, self.x_subpixels)?,
                    y: fixed(self.y, self.y_subpixels)?,
                },
                radius_x: word(self.radius_x)?,
                radius_y: word(self.radius_y)?,
                properties: flags(properties),
                extra_properties: flags(word(self.extra_properties)?),
                ai_handlers: flags(word(self.ai_handlers)?),
                health: word(self.health)?,
                timer: word(self.timer)?,
                cursor,
                spritemap,
                palette: PaletteIndex(word(self.palette)?),
                tiles: TilesIndex(word(self.tiles)?),
                layer: word(self.layer)?,
                flash: frames(self.flash)?,
                frozen: frames(self.frozen)?,
                invincibility: frames(self.invincibility)?,
                shake: frames(self.shake)?,
                frame_counter: word(self.frame_counter)?,
                parameter_1: word(self.parameter_1)?,
                parameter_2: word(self.parameter_2)?,
            },
            occupancy,
        })
    }
}

/// The set a word holds, bit `n` the set's `n`th member: the slot's flag words are in
/// their enums' bit order, so every word, unnamed bits included, round-trips.
fn flags<E>(word: u16) -> FlagSet<E> {
    FlagSet {
        bits: u32::from(word),
        ..FlagSet::default()
    }
}

/// An AI word a species' body has no value for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownWord {
    /// Which of the six, from the first.
    pub index: usize,
    /// The word.
    pub value: u16,
}

/// The body a species with `behaviour` runs, holding its AI words.
///
/// # Errors
///
/// [`UnknownWord`] for a word the species' body has no value for.
pub fn body(behaviour: Behaviour, variables: AiVariables) -> Result<EnemyBody, UnknownWord> {
    Ok(match behaviour {
        Behaviour::MorphBallEye => EnemyBody::MorphBallEye(eye(variables)?),
        Behaviour::Elevator => EnemyBody::Elevator(elevator(variables)),
        Behaviour::FaceBlock => EnemyBody::FaceBlock { variables },
    })
}

/// The eye's function pointers in bank $A8 (bank $A8 log, `$A8:90E2`), by phase.
const EYE_PHASES: [(u16, EyePhase); 5] = [
    (0x90f1, EyePhase::WaitingForSamus),
    (0x912e, EyePhase::Activating),
    (0x9160, EyePhase::Active),
    (0x91ce, EyePhase::Deactivating),
    (0x91dc, EyePhase::Mount),
];

/// The eye's AI words (bank $A8 log, `$A8:9058` to `$A8:91DC`): the third its activated
/// flag, the fourth its angle, the fifth its function timer, the sixth its function.
///
/// # Errors
///
/// [`UnknownWord`] for an activated flag other than 0 or 1, or a function that isn't one
/// of the eye's.
pub fn eye(variables: AiVariables) -> Result<MorphBallEye, UnknownWord> {
    let activated = match variables.c {
        0 => false,
        1 => true,
        value => return Err(UnknownWord { index: 2, value }),
    };
    let phase = EYE_PHASES
        .iter()
        .find(|&&(word, _)| word == variables.f)
        .map(|&(_, phase)| phase)
        .ok_or(UnknownWord {
            index: 5,
            value: variables.f,
        })?;
    Ok(MorphBallEye {
        unused: [variables.a, variables.b],
        activated,
        angle: variables.d,
        function_timer: Frames(variables.e),
        phase,
    })
}

/// The words `eye` holds, in slot order: [`eye`]'s inverse.
#[must_use]
pub fn eye_words(eye: MorphBallEye) -> AiVariables {
    let [first, second] = eye.unused;
    let function = EYE_PHASES
        .iter()
        .find(|&&(_, phase)| phase == eye.phase)
        .map_or(0, |&(word, _)| word);
    AiVariables {
        a: first,
        b: second,
        c: u16::from(eye.activated),
        d: eye.angle,
        e: eye.function_timer.0,
        f: function,
    }
}

/// The elevator's AI words (bank $A3 log): the first is its target Y.
#[must_use]
pub const fn elevator(variables: AiVariables) -> Elevator {
    Elevator {
        target_y: variables.a,
        unused: [
            variables.b,
            variables.c,
            variables.d,
            variables.e,
            variables.f,
        ],
    }
}

/// The words `elevator` holds, in slot order: [`elevator`]'s inverse.
#[must_use]
pub const fn elevator_words(elevator: Elevator) -> AiVariables {
    let [second, third, fourth, fifth, sixth] = elevator.unused;
    AiVariables {
        a: elevator.target_y,
        b: second,
        c: third,
        d: fourth,
        e: fifth,
        f: sixth,
    }
}

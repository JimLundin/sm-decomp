//! Enemy instruction lists, decoded into typed programs (D13).
//!
//! Bank $A0 log (<https://patrickjohnston.org/bank/A0>): an enemy's instruction pointer
//! (`$0F92`) names a list of words in its AI bank. A word below `$8000` is a frame: that many
//! frames of the spritemap the next word points to. A word with its high bit set is an
//! instruction routine; each AI bank keeps a copy of the common instructions at the offsets
//! bank $A0 has them (`$A0:807C` to `$A0:817D`), and its operands follow it. Processing the
//! instructions (`$A0:C26A`) runs routines until a frame.
//!
//! The decoder walks every list the loaded species' AI names, follows each go-to, and cuts
//! the words into programs: a program is a run of steps that fall through to each other, so
//! every step is in exactly one program and every address a list reaches has one place.

use std::collections::{BTreeMap, BTreeSet};

use canonical::Canonical;

use crate::id::ProgramId;
use crate::rom::{AddressError, Lorom, RomAddress};
use crate::spritemap;

/// `$A0:807C`: delete the enemy.
const DELETE: u16 = 0x807c;
/// `$A0:808A`: call the function the operand names.
const CALL: u16 = 0x808a;
/// `$A0:80ED`: go to the operand.
const GOTO: u16 = 0x80ed;
/// `$A0:8110`: decrement the timer and go to the operand if it isn't zero.
const DECREMENT_TIMER_GOTO: u16 = 0x8110;
/// `$A0:8123`: the timer is the operand.
const SET_TIMER: u16 = 0x8123;
/// `$A0:812F`: sleep.
const SLEEP: u16 = 0x812f;
/// `$A0:813A`: wait the operand's frames.
const WAIT: u16 = 0x813a;
/// `$A0:8173`: enable off-screen processing.
const ENABLE_OFF_SCREEN: u16 = 0x8173;
/// `$A0:817D`: disable off-screen processing.
const DISABLE_OFF_SCREEN: u16 = 0x817d;

/// A spritemap in an AI bank: its stable key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Spritemap(pub RomAddress);

/// A spritemap in the loaded content: its place in [`EnemyPrograms`]' spritemap table.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpritemapId(pub u16);

/// A step of an enemy program: what a go-to names and an enemy's cursor holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Target {
    /// The program.
    pub program: ProgramId,
    /// The step within it.
    pub step: u16,
}

/// A species function an instruction list calls (`$A0:808A`). None of the lists slice 0's
/// species run calls one; each joins as a variant with the species whose list calls it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Function {}

/// One step of an enemy program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    /// Show the spritemap for `delay` frames: a word below `$8000` and the spritemap's.
    Show {
        /// The instruction timer it sets.
        delay: u16,
        /// The spritemap.
        spritemap: SpritemapId,
    },
    /// `$A0:807C`: mark the enemy deleted and stop.
    Delete,
    /// `$A0:808A`: call the species function, then on.
    Call(Function),
    /// `$A0:80ED`: go to the target.
    Goto(Target),
    /// `$A0:8110`: decrement the timer; go to the target unless it reached zero, else on.
    DecrementTimerGoto(Target),
    /// `$A0:8123`: set the timer, then on.
    SetTimer(u16),
    /// `$A0:812F`: stop here; the next run starts at this step again.
    Sleep,
    /// `$A0:813A`: set the instruction timer and stop; the next run starts at the next step.
    Wait(u16),
    /// `$A0:8173`: set the processed-off-screen property, then on.
    EnableOffScreen,
    /// `$A0:817D`: clear the processed-off-screen property, then on.
    DisableOffScreen,
}

impl Instruction {
    /// Whether processing stops at this step for the frame.
    #[must_use]
    pub const fn stops(self) -> bool {
        match self {
            Self::Show { .. } | Self::Delete | Self::Sleep | Self::Wait(_) => true,
            Self::Call(_)
            | Self::Goto(_)
            | Self::DecrementTimerGoto(_)
            | Self::SetTimer(_)
            | Self::EnableOffScreen
            | Self::DisableOffScreen => false,
        }
    }

    /// Whether the step after it can run next.
    const fn falls_through(self) -> bool {
        !matches!(self, Self::Delete | Self::Goto(_) | Self::Sleep)
    }
}

/// One program: its steps, each with the address the original's pointer holds for it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    steps: Vec<(RomAddress, Instruction)>,
}

impl Program {
    /// The steps, in order.
    pub fn steps(&self) -> impl Iterator<Item = Instruction> + '_ {
        self.steps.iter().map(|&(_, instruction)| instruction)
    }
}

/// Why the instruction lists don't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramError {
    /// A list word is outside the image.
    Address(AddressError),
    /// A list runs past its bank's end.
    BankEnd(RomAddress),
    /// An instruction routine the decoder doesn't know, at the address, with its word.
    UnknownInstruction(RomAddress, u16),
    /// A call to a species function no species names, at the address.
    UnknownFunction(RomAddress),
    /// Steps that can run forever without stopping, through the address.
    Spins(RomAddress),
    /// A list or spritemap a species' AI names that the loaded programs don't hold.
    Unloaded(RomAddress),
}

impl From<AddressError> for ProgramError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// The instruction lists slice 0's species start.
pub const SLICE_ZERO_LISTS: [RomAddress; 15] = [
    // The elevator's (`$A3:94E6`).
    RomAddress::new(0xa3, 0x94d6),
    // The Morph Ball Room eye's: active (`$A8:8FAC`), deactivating and closed facing right
    // and left, activating facing right and left (bank $A8 log, `$A8:8FAC` to `$A8:9026`),
    // and the four its init picks by facing (`$A8:90DA`).
    RomAddress::new(0xa8, 0x8fac),
    RomAddress::new(0xa8, 0x8ff0),
    RomAddress::new(0xa8, 0x8ffc),
    RomAddress::new(0xa8, 0x9002),
    RomAddress::new(0xa8, 0x900e),
    RomAddress::new(0xa8, 0x9014),
    RomAddress::new(0xa8, 0x9026),
    RomAddress::new(0xa8, 0x9038),
    RomAddress::new(0xa8, 0x903e),
    RomAddress::new(0xa8, 0x9044),
    RomAddress::new(0xa8, 0x904a),
    // The face block's: turning left, turning right, facing forward (bank $A8 log,
    // `$A8:E80C`, `$A8:E81A`, `$A8:E828`).
    RomAddress::new(0xa8, 0xe80c),
    RomAddress::new(0xa8, 0xe81a),
    RomAddress::new(0xa8, 0xe828),
];

/// The spritemaps slice 0's species set outside a list: the empty spritemap their inits set
/// (`$A3:804D`, the elevator's init at `$A3:94E6`; `$A8:804D`, the eye's).
pub const SLICE_ZERO_SPRITEMAPS: [Spritemap; 2] = [
    Spritemap(RomAddress::new(0xa3, 0x804d)),
    Spritemap(RomAddress::new(0xa8, 0x804d)),
];

/// The sprites of each of `spritemaps`, in the format bank $81's `$81:8AB8` reads.
// open: the extended format (bank $A0 log, `$A0:944A` with extra property bit 2) joins with
// the first species that has it.
fn pictures(
    rom: Lorom<'_>,
    spritemaps: &[Spritemap],
) -> Result<Vec<spritemap::Spritemap>, AddressError> {
    spritemaps
        .iter()
        .map(|&Spritemap(at)| spritemap::Spritemap::load(rom, at))
        .collect()
}

/// An instruction before its go-to targets and spritemap are resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Raw {
    Show(u16, Spritemap),
    Delete,
    Goto(RomAddress),
    DecrementTimerGoto(RomAddress),
    SetTimer(u16),
    Sleep,
    Wait(u16),
    EnableOffScreen,
    DisableOffScreen,
}

impl Raw {
    /// Bytes it takes, its word and its operands.
    const fn size(self) -> usize {
        match self {
            Self::Delete | Self::Sleep | Self::EnableOffScreen | Self::DisableOffScreen => 2,
            Self::Show(..)
            | Self::Goto(_)
            | Self::DecrementTimerGoto(_)
            | Self::SetTimer(_)
            | Self::Wait(_) => 4,
        }
    }

    const fn falls_through(self) -> bool {
        !matches!(self, Self::Delete | Self::Goto(_) | Self::Sleep)
    }

    const fn target(self) -> Option<RomAddress> {
        match self {
            Self::Goto(target) | Self::DecrementTimerGoto(target) => Some(target),
            Self::Show(..)
            | Self::Delete
            | Self::SetTimer(_)
            | Self::Sleep
            | Self::Wait(_)
            | Self::EnableOffScreen
            | Self::DisableOffScreen => None,
        }
    }
}

/// The word at `at`.
fn word(rom: Lorom<'_>, at: RomAddress) -> Result<u16, ProgramError> {
    let bytes = rom.read(at, 2)?;
    let [low, high] = <[u8; 2]>::try_from(bytes).map_err(|_| ProgramError::BankEnd(at))?;
    Ok(u16::from_le_bytes([low, high]))
}

/// The instruction at `at`.
fn raw(rom: Lorom<'_>, at: RomAddress) -> Result<Raw, ProgramError> {
    let operand_at = at.advance(2).ok_or(ProgramError::BankEnd(at))?;
    let operand = || word(rom, operand_at);
    let in_bank = |offset: u16| RomAddress::new(at.bank, offset);
    let head = word(rom, at)?;
    Ok(match head {
        delay if delay < 0x8000 => Raw::Show(delay, Spritemap(in_bank(operand()?))),
        DELETE => Raw::Delete,
        CALL => return Err(ProgramError::UnknownFunction(at)),
        GOTO => Raw::Goto(in_bank(operand()?)),
        DECREMENT_TIMER_GOTO => Raw::DecrementTimerGoto(in_bank(operand()?)),
        SET_TIMER => Raw::SetTimer(operand()?),
        SLEEP => Raw::Sleep,
        WAIT => Raw::Wait(operand()?),
        ENABLE_OFF_SCREEN => Raw::EnableOffScreen,
        DISABLE_OFF_SCREEN => Raw::DisableOffScreen,
        unknown => return Err(ProgramError::UnknownInstruction(at, unknown)),
    })
}

/// The loaded species' enemy programs and the spritemaps they show.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnemyPrograms {
    programs: Vec<Program>,
    places: BTreeMap<RomAddress, Target>,
    spritemaps: Vec<Spritemap>,
    /// Each of `spritemaps`' sprites, in the same order.
    pictures: Vec<spritemap::Spritemap>,
}

impl EnemyPrograms {
    /// Decode the lists starting at `lists` and every list they go to, with the spritemaps
    /// they show and `spritemaps`.
    ///
    /// # Errors
    ///
    /// [`ProgramError`] if a list word is outside the image, an instruction is unknown, or
    /// steps can run forever without stopping.
    pub fn load(
        rom: Lorom<'_>,
        lists: &[RomAddress],
        spritemaps: &[Spritemap],
    ) -> Result<Self, ProgramError> {
        let mut words: BTreeMap<RomAddress, Raw> = BTreeMap::new();
        let mut pending: Vec<RomAddress> = lists.to_vec();
        while let Some(at) = pending.pop() {
            if words.contains_key(&at) {
                continue;
            }
            let instruction = raw(rom, at)?;
            words.insert(at, instruction);
            if instruction.falls_through() {
                pending.push(
                    at.advance(instruction.size())
                        .ok_or(ProgramError::BankEnd(at))?,
                );
            }
            pending.extend(instruction.target());
        }
        let continued: BTreeSet<RomAddress> = words
            .iter()
            .filter(|(_, instruction)| instruction.falls_through())
            .filter_map(|(&at, instruction)| at.advance(instruction.size()))
            .collect();
        let mut places = BTreeMap::new();
        let mut runs: Vec<Vec<(RomAddress, Raw)>> = Vec::new();
        for (&start, _) in words.iter().filter(|(at, _)| !continued.contains(at)) {
            let program =
                ProgramId(u16::try_from(runs.len()).map_err(|_| ProgramError::BankEnd(start))?);
            let mut run = Vec::new();
            let mut at = Some(start);
            while let Some(here) = at {
                let Some(&instruction) = words.get(&here) else {
                    break;
                };
                let step = u16::try_from(run.len()).map_err(|_| ProgramError::BankEnd(here))?;
                places.insert(here, Target { program, step });
                run.push((here, instruction));
                at = instruction
                    .falls_through()
                    .then(|| here.advance(instruction.size()))
                    .flatten();
            }
            runs.push(run);
        }
        let mut shown: BTreeSet<Spritemap> = spritemaps.iter().copied().collect();
        shown.extend(words.values().filter_map(|instruction| match instruction {
            Raw::Show(_, spritemap) => Some(*spritemap),
            Raw::Delete
            | Raw::Goto(_)
            | Raw::DecrementTimerGoto(_)
            | Raw::SetTimer(_)
            | Raw::Sleep
            | Raw::Wait(_)
            | Raw::EnableOffScreen
            | Raw::DisableOffScreen => None,
        }));
        let spritemaps: Vec<Spritemap> = shown.into_iter().collect();
        let place = |at: RomAddress| places.get(&at).copied().ok_or(ProgramError::BankEnd(at));
        let spritemap_id = |spritemap: Spritemap| {
            spritemaps
                .binary_search(&spritemap)
                .ok()
                .and_then(|index| u16::try_from(index).ok())
                .map(SpritemapId)
                .ok_or(ProgramError::BankEnd(spritemap.0))
        };
        let programs = runs
            .into_iter()
            .map(|run| {
                let steps = run
                    .into_iter()
                    .map(|(at, instruction)| {
                        let typed = match instruction {
                            Raw::Show(delay, spritemap) => Instruction::Show {
                                delay,
                                spritemap: spritemap_id(spritemap)?,
                            },
                            Raw::Delete => Instruction::Delete,
                            Raw::Goto(target) => Instruction::Goto(place(target)?),
                            Raw::DecrementTimerGoto(target) => {
                                Instruction::DecrementTimerGoto(place(target)?)
                            }
                            Raw::SetTimer(value) => Instruction::SetTimer(value),
                            Raw::Sleep => Instruction::Sleep,
                            Raw::Wait(frames) => Instruction::Wait(frames),
                            Raw::EnableOffScreen => Instruction::EnableOffScreen,
                            Raw::DisableOffScreen => Instruction::DisableOffScreen,
                        };
                        Ok((at, typed))
                    })
                    .collect::<Result<_, ProgramError>>()?;
                Ok(Program { steps })
            })
            .collect::<Result<_, ProgramError>>()?;
        let loaded = Self {
            programs,
            places,
            pictures: pictures(rom, &spritemaps)?,
            spritemaps,
        };
        loaded.check_stops()?;
        Ok(loaded)
    }

    /// The programs slice 0's species run.
    ///
    /// # Errors
    ///
    /// As [`Self::load`].
    pub fn slice_zero(rom: Lorom<'_>) -> Result<Self, ProgramError> {
        Self::load(rom, &SLICE_ZERO_LISTS, &SLICE_ZERO_SPRITEMAPS)
    }

    /// Fail if a step can reach itself through steps that don't stop: processing would
    /// never end (D13's step bound).
    fn check_stops(&self) -> Result<(), ProgramError> {
        let next = |target: Target| -> Vec<Target> {
            let Some(instruction) = self.step(target) else {
                return Vec::new();
            };
            if instruction.stops() {
                return Vec::new();
            }
            let mut after = Vec::new();
            if instruction.falls_through() {
                after.push(Target {
                    step: target.step.wrapping_add(1),
                    ..target
                });
            }
            match instruction {
                Instruction::Goto(to) | Instruction::DecrementTimerGoto(to) => after.push(to),
                Instruction::Show { .. }
                | Instruction::Delete
                | Instruction::Call(_)
                | Instruction::SetTimer(_)
                | Instruction::Sleep
                | Instruction::Wait(_)
                | Instruction::EnableOffScreen
                | Instruction::DisableOffScreen => {}
            }
            after
        };
        let bound = self.places.len();
        for (&at, &start) in &self.places {
            // Every path of non-stopping steps from `start` ends within `bound` steps, or
            // it repeats one.
            let mut frontier = vec![start];
            for _ in 0..=bound {
                frontier = frontier.into_iter().flat_map(next).collect();
                frontier.sort_unstable();
                frontier.dedup();
                if frontier.is_empty() {
                    break;
                }
            }
            if !frontier.is_empty() {
                return Err(ProgramError::Spins(at));
            }
        }
        Ok(())
    }

    /// The step at `target`; `None` past the content.
    #[must_use]
    pub fn step(&self, target: Target) -> Option<Instruction> {
        self.programs
            .get(usize::from(target.program.0))?
            .steps
            .get(usize::from(target.step))
            .map(|&(_, instruction)| instruction)
    }

    /// The program with `id`.
    #[must_use]
    pub fn program(&self, id: ProgramId) -> Option<&Program> {
        self.programs.get(usize::from(id.0))
    }

    /// How many programs there are.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.programs.len()
    }

    /// Whether there are none.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }

    /// The step the original's pointer `at` names; `None` for an address no list reaches.
    #[must_use]
    pub fn target(&self, at: RomAddress) -> Option<Target> {
        self.places.get(&at).copied()
    }

    /// The original's pointer for `target`: [`Self::target`]'s inverse.
    #[must_use]
    pub fn address(&self, target: Target) -> Option<RomAddress> {
        self.programs
            .get(usize::from(target.program.0))?
            .steps
            .get(usize::from(target.step))
            .map(|&(at, _)| at)
    }

    /// The ID of `spritemap`; `None` for one the content doesn't hold.
    #[must_use]
    pub fn spritemap_id(&self, spritemap: Spritemap) -> Option<SpritemapId> {
        let index = self.spritemaps.binary_search(&spritemap).ok()?;
        u16::try_from(index).ok().map(SpritemapId)
    }

    /// The spritemap with `id`: [`Self::spritemap_id`]'s inverse.
    #[must_use]
    pub fn spritemap(&self, id: SpritemapId) -> Option<Spritemap> {
        self.spritemaps.get(usize::from(id.0)).copied()
    }

    /// The sprites of the spritemap with `id`; `None` for one the content doesn't hold.
    #[must_use]
    pub fn sprites(&self, id: SpritemapId) -> Option<&spritemap::Spritemap> {
        self.pictures.get(usize::from(id.0))
    }
}

#[cfg(test)]
mod tests;

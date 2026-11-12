//! A Ceres door's slot words as the boss-ceres-ridley model's door.
//!
//! By the decode notes' tables (`docs/behaviours/boss-ceres-ridley.decode.md`, "State"): the
//! phase from the instruction pointer (`+1A`) and the instruction timer (`+1C`), the figure from
//! the spritemap pointer (`+16`).
//!
//! Only the doors this piece models decode as one: parameter 0 (facing right), 1 (facing left)
//! and 4 (the shaft's door); the station's other doors stay unmodelled.

use core::num::Wrapping;

use boss_ceres_ridley::{CeresDoor, DoorKind, Figure, FigureLook, Phase, Quarter, Wait};
use game::boss_ceres_ridley_bridge::{Door, kind};
use types::{Pair, Px, Side};

/// The six AI words of every door this piece models: the AI function (`$A6:F76B`), then the
/// drawn-by-Ridley flag and the rumble's words, which only the later pieces' kinds set.
pub const WORDS: [u16; 6] = [0xF76B, 0, 0, 0, 0, 0];

/// Why a door's words aren't a door's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unknown {
    /// An instruction pointer no phase of its kind starts at.
    Instruction(u16),
    /// An instruction timer of 0 in a phase that waits.
    Timer(u16),
}

/// The phase a door of `facing` and `kind` holds at instruction pointer `at`, with the wait
/// `timer`; `Placed` and `Wall` carry none.
///
/// # Errors
///
/// [`Unknown`] for a pointer no phase of the door's starts at, or a wait of 0.
pub fn phase(kind: DoorKind, facing: Side, at: u16, timer: u16) -> Result<Phase, Unknown> {
    let waits = |make: fn(Wait) -> Phase| Wait::new(timer).map(make).ok_or(Unknown::Timer(timer));
    let quarter = |base: u16| {
        let quarters = [
            Quarter::First,
            Quarter::Second,
            Quarter::Third,
            Quarter::Fourth,
        ];
        quarters
            .into_iter()
            .zip((0_u16..).map(|step| base.wrapping_add(step.wrapping_mul(4))))
            .find_map(|(quarter, start)| (start == at).then_some(quarter))
    };
    let (placed, starting, open, closing, closed, opening) = match facing {
        Side::Right => (0xF56C, 0xF574, 0xF57C, 0xF58C, 0xF5A0, 0xF5AA),
        Side::Left => (
            match kind {
                DoorKind::Door => 0xF5BE,
                DoorKind::ShaftDoor => 0xF61A,
            },
            0xF5C6,
            0xF5CE,
            0xF5DE,
            0xF5F2,
            0xF5FC,
        ),
    };
    if at == placed {
        return Ok(Phase::Placed);
    }
    if matches!(kind, DoorKind::ShaftDoor) && at == 0xF626 {
        return Ok(Phase::Wall);
    }
    if at == starting {
        return waits(|wait| Phase::Starting { wait });
    }
    if at == open {
        return waits(|wait| Phase::Open { wait });
    }
    if at == closed {
        return waits(|wait| Phase::Closed { wait });
    }
    if let Some(quarter) = quarter(closing) {
        let wait = Wait::new(timer).ok_or(Unknown::Timer(timer))?;
        return Ok(Phase::Closing { quarter, wait });
    }
    if let Some(quarter) = quarter(opening) {
        let wait = Wait::new(timer).ok_or(Unknown::Timer(timer))?;
        return Ok(Phase::Opening { quarter, wait });
    }
    Err(Unknown::Instruction(at))
}

/// The figure the spritemap pointer `spritemap` shows: none for the loader's and the init's
/// (a `Placed` door's), `Err` for a pointer no door figure is.
///
/// # Errors
///
/// The pointer, when it is neither a figure nor a placed door's.
pub fn figure(spritemap: u16) -> Result<Option<Figure>, u16> {
    const PLACED: [u16; 2] = [0x804D, 0xFAC7];
    const LOOKS: [FigureLook; 5] = [
        FigureLook::Closed,
        FigureLook::Ajar1,
        FigureLook::Ajar2,
        FigureLook::Ajar3,
        FigureLook::Open,
    ];
    const RIGHT: [u16; 5] = [0xFA13, 0xFA3D, 0xFA67, 0xFA87, 0xFAA7];
    const LEFT: [u16; 5] = [0xF95F, 0xF989, 0xF9B3, 0xF9D3, 0xF9F3];
    if PLACED.contains(&spritemap) {
        return Ok(None);
    }
    [(Side::Right, RIGHT), (Side::Left, LEFT)]
        .into_iter()
        .flat_map(|(facing, pointers)| {
            LOOKS
                .into_iter()
                .zip(pointers)
                .map(move |(look, pointer)| (Figure { look, facing }, pointer))
        })
        .find_map(|(figure, pointer)| (pointer == spritemap).then_some(Some(figure)))
        .ok_or(spritemap)
}

/// The door a slot of parameter `parameter` at `position` holds, from its instruction pointer
/// and timer; `Ok(None)` for a parameter this piece doesn't model.
///
/// # Errors
///
/// As [`phase`].
pub fn door(
    parameter: u16,
    position: Pair<Px>,
    instruction: u16,
    timer: u16,
) -> Result<Option<Door>, Unknown> {
    let Some((kind, facing)) = kind(parameter) else {
        return Ok(None);
    };
    Ok(Some(Door(CeresDoor {
        kind,
        facing,
        position,
        phase: phase(kind, facing, instruction, timer)?,
    })))
}

/// A slot position's whole pixels as the model's position.
#[must_use]
pub const fn position(x: u16, y: u16) -> Pair<Px> {
    Pair {
        x: Wrapping(x),
        y: Wrapping(y),
    }
}

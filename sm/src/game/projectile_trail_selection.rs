//! Select the upcoming or active trail definition at its scheduled phase.
#![forbid(unsafe_code)]
use crate::game::projectile_playback::{Command, Definitions, Frame, Owner};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Source {
    Upcoming,
    Active,
}

pub trait TrailDefinitions<O: Owner>: Definitions<O> {
    type Marker;
    fn sample_trail(&mut self, owner: O, cursor: O::Cursor, source: Source) -> (O, Self::Marker);
}

pub fn sample<O: Owner, D: TrailDefinitions<O>>(
    mut owner: O,
    slot: O::Slot,
    defs: &mut D,
) -> (O, D::Marker) {
    let cursor = owner.cursor(slot);
    let mut upcoming = owner.remaining(slot) == 1;
    if upcoming {
        let (next, frame) = defs.open(owner, cursor);
        owner = defs.validate(
            next,
            &frame,
            crate::game::projectile_playback::Field::Command,
        );
        upcoming = matches!(frame.command(), Command::Show { .. });
    }
    defs.sample_trail(
        owner,
        cursor,
        if upcoming {
            Source::Upcoming
        } else {
            Source::Active
        },
    )
}

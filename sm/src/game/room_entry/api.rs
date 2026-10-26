//! Room arrival requests and explicit suspension outcomes.
#![forbid(unsafe_code)]
use super::world::{ExitId, StartId};

use std::num::NonZeroU8;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactSite {
    CurrentCollision,
    Authored(ExitId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartSite {
    CurrentLoadStation,
    Authored(StartId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrivalRequest {
    Contact(ContactSite),
    Start(StartSite),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryWait {
    Nmi,
    NoUpdate,
    Music,
    Mechanisms,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryPoll {
    Rejected,
    Returned(u8),
    Waiting {
        cause: EntryWait,
        returned: NonZeroU8,
    },
    Complete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntrySignal {
    Initial,
    DoorState9,
    DoorState10,
    DoorState11,
    ScrollIrq,
}
impl EntryPoll {
    pub(crate) fn returned_byte(self) -> u8 {
        match self {
            Self::Returned(value) => value,
            Self::Waiting { returned, .. } => returned.get(),
            Self::Rejected | Self::Complete => 0,
        }
    }
    pub(crate) fn legacy_unknown(value: u8) -> Self {
        Self::Returned(value)
    }
}

//! Complete ordered physical outputs and their independently retained delivery progress.

use crate::production::timing::{self, Event};
use canonical::Canonical;
use data::timing::WRAM_DATA_PORT;

/// Physical effects already produced by hardware, with external delivery continuation.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Publications {
    /// Complete ordered stream, including already delivered effects.
    pub events: Vec<Event>,
    delivered: u64,
    interval: u64,
    previous_interval: u64,
}

/// Invalid imported publication progress or out-of-order physical effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicationFault {
    /// Delivery progress lies outside the retained physical stream.
    Cursor,
    /// A new actual output precedes an already produced output.
    Order,
}

pub(super) fn writes_wram(event: &timing::Event) -> bool {
    match &event.kind {
        timing::EventKind::Publication(write) => write.port == WRAM_DATA_PORT,
        timing::EventKind::Payload(payload) => payload.target.publication_port() == WRAM_DATA_PORT,
        timing::EventKind::Sample(..)
        | timing::EventKind::BulkService(_)
        | timing::EventKind::LineService(_)
        | timing::EventKind::Interrupt(_)
        | timing::EventKind::ControllerStart
        | timing::EventKind::ControllerEnd
        | timing::EventKind::ControllerCommitted
        | timing::EventKind::AudioPortRead { .. } => false,
    }
}

impl Publications {
    /// Retain fresh physical effects in their actual hardware order.
    /// Equal instants preserve the supplied order and multiplicity.
    ///
    /// # Errors
    /// Invalid cursor or reversed physical time rejects the entire batch.
    pub fn record(&mut self, events: Vec<Event>) -> Result<(), PublicationFault> {
        self.validate()?;
        let mut previous = self.events.last().map(|event| event.at);
        for event in &events {
            if previous.is_some_and(|at| at > event.at) {
                return Err(PublicationFault::Order);
            }
            previous = Some(event.at);
        }
        self.events.extend(events);
        Ok(())
    }

    /// Deliver only newly produced outputs from the frame's private candidate.
    /// Commit delivery position with downstream effects and the complete frame state.
    ///
    /// # Errors
    /// Invalid restored delivery positions reject before changing the cursor.
    pub fn deliver(&mut self) -> Result<Vec<Event>, PublicationFault> {
        self.validate()?;
        let position = usize::try_from(self.delivered).map_err(|_| PublicationFault::Cursor)?;
        let delivered = u64::try_from(self.events.len()).map_err(|_| PublicationFault::Cursor)?;
        let outputs = self.events.iter().skip(position).cloned().collect();
        self.delivered = delivered;
        Ok(outputs)
    }

    pub(super) fn begin_interval(&mut self) -> Result<(), PublicationFault> {
        if usize::try_from(self.delivered) != Ok(self.events.len()) {
            return Err(PublicationFault::Cursor);
        }
        self.previous_interval = self.interval;
        self.interval = self.delivered;
        Ok(())
    }

    pub(super) const fn original_interval(&self) -> bool {
        self.interval == 0
    }

    pub(super) const fn interval_start(&self) -> u64 {
        self.interval
    }

    pub(super) const fn previous_was_original(&self) -> bool {
        self.previous_interval == 0
    }

    pub(super) fn preceding(&self) -> Result<&[Event], PublicationFault> {
        usize::try_from(self.interval)
            .ok()
            .and_then(|index| self.events.get(..index))
            .ok_or(PublicationFault::Cursor)
    }

    pub(super) fn previous(&self) -> Result<&[Event], PublicationFault> {
        let start =
            usize::try_from(self.previous_interval).map_err(|_| PublicationFault::Cursor)?;
        let end = usize::try_from(self.interval).map_err(|_| PublicationFault::Cursor)?;
        self.events.get(start..end).ok_or(PublicationFault::Cursor)
    }

    /// Current opportunity's physical events, in reached hardware order.
    ///
    /// # Errors
    /// The retained interval cursor is outside the event stream.
    pub fn active(&self) -> Result<&[Event], PublicationFault> {
        usize::try_from(self.interval)
            .ok()
            .and_then(|index| self.events.get(index..))
            .ok_or(PublicationFault::Cursor)
    }

    /// Validate the complete imported publication continuation.
    ///
    /// # Errors
    /// Cursor outside the stream or nonmonotone retained physical time.
    pub fn validate(&self) -> Result<(), PublicationFault> {
        let position = usize::try_from(self.delivered).map_err(|_| PublicationFault::Cursor)?;
        if self.previous_interval > self.interval
            || self.interval > self.delivered
            || position > self.events.len()
        {
            return Err(PublicationFault::Cursor);
        }
        let mut previous = None;
        for event in &self.events {
            if previous.is_some_and(|at| at > event.at) {
                return Err(PublicationFault::Order);
            }
            previous = Some(event.at);
        }
        Ok(())
    }
}

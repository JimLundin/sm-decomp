//! Complete ordered physical outputs and their independently retained delivery progress.

use crate::production::timing::Event;
use canonical::Canonical;

/// Physical effects already produced by hardware, with external delivery continuation.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Publications {
    /// Complete ordered stream, including already delivered effects.
    pub events: Vec<Event>,
    delivered: u64,
    interval: u64,
}

/// Invalid imported publication progress or out-of-order physical effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicationFault {
    /// Delivery progress lies outside the retained physical stream.
    Cursor,
    /// A new actual output precedes an already produced output.
    Order,
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
        self.interval = self.delivered;
        Ok(())
    }

    pub(super) const fn original_interval(&self) -> bool {
        self.interval == 0
    }

    pub(super) fn active(&self) -> Result<&[Event], PublicationFault> {
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
        if self.interval > self.delivered || position > self.events.len() {
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

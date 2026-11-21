//! Retained liquid history with actual finite-frame preservation receipts.

use super::borrowed_player_preparation::PlayerEntryCompletionsPin;
use super::{interrupts::DrawingRequest, movement_caller::RunCompletion};
use canonical::{Canonical, model::Model};

/// Independently decoded original liquid history.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    /// Complete original image identity, independent of current surfaces.
    pub origin: [u8; 32],
    /// Retained meaning decoded at the original admission boundary.
    pub meaning: Meaning,
}

/// Retained physics meanings; only Air has qualified preservation.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meaning {
    /// No retained liquid transition.
    Air,
    /// Another history requires a qualified producer.
    Other,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Interrupts {
    Unseen,
    BeginAdmitted,
    BeginReturned,
    EndAdmitted,
    Returned,
}

/// Same original authority carried through actual owner completions.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct History {
    origin: Option<Admission>,
    current: Meaning,
    preserved: bool,
    interruption: Option<super::interruption_caller::Admission>,
    environment: Option<Model<PlayerEntryCompletionsPin>>,
    run: Option<RunCompletion>,
    interrupts: Interrupts,
}

/// Missing actual history or an invalidated original authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fault;

impl History {
    /// Admit history once without consulting current liquid surfaces.
    /// # Errors
    /// Other retained meanings have no qualified preservation.
    pub fn from_origin(origin: Admission) -> Result<Self, Fault> {
        if origin.meaning != Meaning::Air {
            return Err(Fault);
        }
        Ok(Self {
            current: origin.meaning,
            origin: Some(origin),
            preserved: true,
            interruption: None,
            environment: None,
            run: None,
            interrupts: Interrupts::Unseen,
        })
    }

    /// Writes to either history byte and unknown activity invalidate even unchanged values.
    pub const fn invalidate(&mut self) {
        self.preserved = false;
    }

    /// Explicit replacement is a writer, including same-value replacement.
    pub const fn write(&mut self, meaning: Meaning) {
        self.current = meaning;
        self.invalidate();
    }

    /// Retain the actual returned Environment owner once.
    /// # Errors
    /// Missing or repeated completion is unsupported.
    pub fn environment(
        &mut self,
        completed: &Model<PlayerEntryCompletionsPin>,
    ) -> Result<(), Fault> {
        self.validate(self.origin.as_ref().ok_or(Fault)?)?;
        if self.environment.is_some() || completed.0.environment.is_none() {
            return Err(Fault);
        }
        self.environment = Some(completed.clone());
        Ok(())
    }

    /// Retain the actual returned Run owner once.
    /// # Errors
    /// Early or repeated Run is unsupported.
    pub fn run(&mut self, completed: &RunCompletion) -> Result<(), Fault> {
        self.validate(self.origin.as_ref().ok_or(Fault)?)?;
        if self.environment.is_none() || self.run.is_some() {
            return Err(Fault);
        }
        self.run = Some(completed.clone());
        Ok(())
    }

    /// Observe only actual successfully admitted HUD interrupts.
    pub const fn hud_irq(&mut self, request: DrawingRequest) {
        self.interrupts = match (self.interrupts, request) {
            (Interrupts::Unseen, DrawingRequest::Begin) => Interrupts::BeginAdmitted,
            (Interrupts::BeginReturned, DrawingRequest::End) => Interrupts::EndAdmitted,
            _ => {
                self.invalidate();
                self.interrupts
            }
        };
    }

    /// Retain the actual matching physical handler return, separately from admission.
    /// # Errors
    /// Wrong, early or repeated returns cannot complete preservation.
    pub fn hud_irq_returned(
        &mut self,
        request: DrawingRequest,
        work: crate::production::timing::WorkId,
    ) -> Result<(), Fault> {
        if work != crate::production::timing::WorkId(100) {
            return Err(Fault);
        }
        self.interrupts = match (self.interrupts, request) {
            (Interrupts::BeginAdmitted, DrawingRequest::Begin) => Interrupts::BeginReturned,
            (Interrupts::EndAdmitted, DrawingRequest::End) => Interrupts::Returned,
            _ => return Err(Fault),
        };
        Ok(())
    }

    /// Complete restored authority preserves original meaning and writer status.
    /// # Errors
    /// Invalidated history cannot regain freshness by restoration.
    pub fn validate_current(&self) -> Result<(), Fault> {
        self.origin.as_ref().map_or_else(
            || if self.preserved { Ok(()) } else { Err(Fault) },
            |origin| self.validate(origin),
        )
    }

    pub(super) fn retire_opportunity(&mut self) {
        // These observations belong to the completed opportunity. Only a writer
        // or unqualified activity invalidates the live retained physics history.
        self.interruption = None;
        self.environment = None;
        self.run = None;
        self.interrupts = Interrupts::Unseen;
    }

    /// Attach independently decoded interruption authority at origin admission.
    /// # Errors
    /// Rejects repeated or late admission and mismatched original images.
    pub fn admit_interruption(
        &mut self,
        admission: super::interruption_caller::Admission,
    ) -> Result<(), Fault> {
        self.validate_current()?;
        if self.interruption.is_some()
            || self
                .origin
                .as_ref()
                .is_none_or(|origin| origin.origin != admission.origin())
            || self.environment.is_some()
            || self.run.is_some()
            || self.interrupts != Interrupts::Unseen
        {
            return Err(Fault);
        }
        self.interruption = Some(admission);
        Ok(())
    }

    /// Invalidate the current interruption authority after a write or unknown activity.
    pub const fn invalidate_interruption(&mut self) {
        self.interruption = None;
    }

    pub(super) fn interruption_holds(
        &self,
        admission: &super::interruption_caller::Admission,
        entry: &Model<PlayerEntryCompletionsPin>,
        run: &RunCompletion,
    ) -> bool {
        self.validate_current().is_ok()
            && self.interruption.as_ref() == Some(admission)
            && self
                .origin
                .as_ref()
                .is_some_and(|origin| admission.origin() == origin.origin)
            && self.environment.as_ref() == Some(entry)
            && entry.0.pose_input.as_ref().is_some_and(|pose| {
                pose.terminal.bomb_jump
                    == super::borrowed_player_preparation::BombJumpDirection::None
            })
            && self.run.as_ref() == Some(run)
            && self.interrupts == Interrupts::Returned
    }

    /// Restore and use preserve complete provenance and writer status.
    /// # Errors
    /// A changed origin or invalidated authority is unsupported.
    pub fn validate(&self, origin: &Admission) -> Result<(), Fault> {
        if !self.preserved || self.origin.as_ref() != Some(origin) || self.current != Meaning::Air {
            return Err(Fault);
        }
        Ok(())
    }

    /// Join separately qualified current effects to the same actual history receipts.
    /// # Errors
    /// Missing receipts, substituted history or unsupported equipment cannot supply Animation.
    pub fn at_animation(
        &self,
        origin: &Admission,
        completed: &Model<PlayerEntryCompletionsPin>,
        run: &RunCompletion,
    ) -> Result<samus_appearance::Liquid, Fault> {
        self.validate(origin)?;
        let current = completed.0.environment.as_ref().ok_or(Fault)?;
        if self
            .environment
            .as_ref()
            .and_then(|entry| entry.0.environment.as_ref())
            != Some(current)
            || self.run.as_ref() != Some(run)
            || self.interrupts != Interrupts::Returned
            || current.dependency.equipment[types::Item::GravitySuit]
            || current.dependency.equipment[types::Item::VariaSuit]
            || current.dependency.water
                != super::borrowed_player_preparation::LiquidSurface::Inactive
            || current.dependency.lava_acid
                != super::borrowed_player_preparation::LiquidSurface::Inactive
        {
            return Err(Fault);
        }
        Ok(samus_appearance::Liquid::NONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retiring_consumers_preserves_air_authority_and_its_invalidation() {
        let origin = Admission {
            origin: [23; 32],
            meaning: Meaning::Air,
        };
        let mut history = History::from_origin(origin.clone()).unwrap();
        history.retire_opportunity();
        assert_eq!(history.validate(&origin), Ok(()));
        history.write(Meaning::Air);
        history.retire_opportunity();
        assert_eq!(history.validate(&origin), Err(Fault));
    }
}

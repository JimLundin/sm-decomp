//! Complete following-camera work from actual completed movement and current room inputs.

use super::movement::Movement;
use crate::{
    production::timing::WorkId,
    state::owned::{CameraPin, MovementPin},
};
use canonical::{Canonical, model::Model};
use core::num::Wrapping;
use types::Parts;

mod contract;
mod physical;
pub(super) use contract::Horizontal;
pub use contract::{
    PhysicalContract, RequiredCentre, RequiredDependencies, RequiredScreens, RequiredView,
};
#[cfg(test)]
mod tests;
pub use physical::{Requests, Samples};

/// Actual movement quantities captured before this ordinary update begins.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Previous(pub Model<MovementPin>);

impl Previous {
    /// Preserve the actual current movement centre before its next update.
    #[must_use]
    pub const fn capture(movement: &Movement) -> Self {
        Self(Model(*movement.state()))
    }
}

/// Current screen permission supplied by the room/scroll owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Following pushes the view out of this screen.
    Closed,
    /// The view may show the complete screen.
    Open,
    /// The lowest view starts at this screen's top.
    Short,
}

impl Screen {
    const fn model(self) -> types::ScreenKind {
        match self {
            Self::Closed => types::ScreenKind::Closed,
            Self::Open => types::ScreenKind::Open,
            Self::Short => types::ScreenKind::Short,
        }
    }
}

/// Complete current room projection, including the run past the room's final screen.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// Actual current room's importer-assigned identity.
    pub identity: data::id::RoomId,
    /// Room rows followed by the room loader's complete remaining screen run.
    pub screens: Vec<Screen>,
    /// Current width and height in screens.
    pub size: [u16; 2],
    /// Aim heights while not rising and while rising, respectively.
    pub heights: [u16; 2],
}

impl Room {
    fn model(&self) -> Result<world_camera::Room, Fault> {
        let [width, height] = self.size;
        let [normal, rising] = self.heights;
        world_camera::Room::new(
            self.screens.iter().map(|screen| screen.model()).collect(),
            types::Pair {
                x: usize::from(width),
                y: usize::from(height),
            },
            world_camera::Heights {
                while_not_rising: Wrapping(normal),
                while_rising: Wrapping(rising),
            },
        )
        .map_err(Fault::Room)
    }
}

/// Current boss/room lead selection.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lead {
    /// Show more of the ordinary direction of travel.
    Ahead,
    /// Keep Samus near the left edge.
    KeepLeft,
    /// Keep Samus near the right edge.
    KeepRight,
    /// Use the room's closer lead.
    Near,
}

impl Lead {
    const fn model(self) -> world_camera::LeadSetting {
        match self {
            Self::Ahead => world_camera::LeadSetting::Ahead,
            Self::KeepLeft => world_camera::LeadSetting::KeepLeft,
            Self::KeepRight => world_camera::LeadSetting::KeepRight,
            Self::Near => world_camera::LeadSetting::Near,
        }
    }
}

/// Complete tuning read by this actual follow, retained across physical suspension.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tuning {
    /// Extra following distance in wrapping fixed-point units.
    pub extra: u32,
    /// Right/left leads for ordinary travel.
    pub ahead: [u16; 2],
    /// Right/left leads keeping Samus near the left edge.
    pub keep_left: [u16; 2],
    /// Right/left leads keeping Samus near the right edge.
    pub keep_right: [u16; 2],
    /// Right/left leads for the closer view.
    pub near: [u16; 2],
    /// Additional whole-pixel push distance.
    pub push: u16,
    /// Bottom margin of an ordinary open screen.
    pub margin: u16,
}

impl Tuning {
    /// Capture every current rule, including adjusted profiles.
    #[must_use]
    pub const fn capture(rules: &world_camera::Rules) -> Self {
        Self {
            extra: rules.extra.0,
            ahead: [rules.ahead.right.0, rules.ahead.left.0],
            keep_left: [rules.keep_left.right.0, rules.keep_left.left.0],
            keep_right: [rules.keep_right.right.0, rules.keep_right.left.0],
            near: [rules.near.right.0, rules.near.left.0],
            push: rules.push.0,
            margin: rules.margin.0,
        }
    }

    const fn model(self) -> world_camera::Rules {
        const fn leads([right, left]: [u16; 2]) -> world_camera::Leads {
            world_camera::Leads {
                right: Wrapping(right),
                left: Wrapping(left),
            }
        }
        world_camera::Rules {
            extra: Wrapping(self.extra),
            ahead: leads(self.ahead),
            keep_left: leads(self.keep_left),
            keep_right: leads(self.keep_right),
            near: leads(self.near),
            push: Wrapping(self.push),
            margin: Wrapping(self.margin),
        }
    }
}

/// Activity reported by an independently current external Camera producer.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    /// The external producer is inactive at this actual opportunity.
    Inactive,
    /// The producer needs a broader Camera adapter.
    Active,
}

/// Current neighbour quantities captured at the actual Camera opportunity.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Inputs {
    /// Current room and complete scroll permissions.
    pub room: Room,
    /// Actual current lead setting.
    pub lead: Lead,
    /// Current time-stop owner output.
    pub time_stopped: bool,
    /// Complete rules current when following is chosen.
    pub tuning: Tuning,
    /// Actual current grapple scroll activity.
    pub grapple: Activity,
    /// Actual current knockback activity.
    pub knockback: Activity,
    /// Actual current room scroll-completion hook activity.
    pub hook: Activity,
}

/// Actual complete operands used to choose the camera's candidate result.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Complete authoritative camera before this follow, including retained fractions.
    pub initial: Model<CameraPin>,
    /// Actual pre-movement state, joined to the camera's remembered centre.
    pub previous: Previous,
    /// Actual finished movement and its complete travel/skid/rise/placement publications.
    pub movement: Movement,
    /// Complete current neighbouring operands and tuning.
    pub inputs: Inputs,
}

/// Candidate and physical metadata; persistent camera authority remains with its owner.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// No camera follow has been chosen in this frame.
    #[default]
    Idle,
    /// A complete chosen follow requires actual-current source qualification.
    Offered {
        /// Exact current operands offered for complete physical qualification.
        receipt: Box<Receipt>,
        /// Private candidate, unavailable to drawing until completion.
        next: Model<CameraPin>,
    },
    /// A qualified whole physical follow retains its exact result until completion.
    Charging {
        /// Actual complete physical operation identity.
        work: WorkId,
        /// Qualified actual operands retained without recapture during a hold.
        receipt: Box<Receipt>,
        /// Result of the original chosen clean follow.
        next: Model<CameraPin>,
        /// Actual hardware products retained through holds and interrupt suspension.
        samples: Samples,
    },
    /// This frame's actual follow completed; its viewport may be consumed by drawing.
    Completed(Box<Receipt>),
}

/// An unsupported dependency or invalid order leaves the persistent camera unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A follow was already chosen, was not bound, or completed twice.
    Order,
    /// Movement has not finished publishing this update's camera inputs.
    Movement,
    /// Actual pre-movement centre differs from the camera's remembered centre.
    Previous,
    /// The supplied current room relation is not checked content.
    Room(world_camera::RoomError),
    /// The physical work or persistent camera changed while the follow was pending.
    Completion,
    /// Missing, extra or reordered Camera multiplication publications/samples.
    Profile,
    /// Actual complete operands do not meet the selected semantic physical dependencies.
    Dependencies,
    /// Actual finished movement diverges from the qualified current centre.
    CurrentCentre,
    /// Actual prior centre and placement diverge from the qualified operands.
    PreviousCentre,
    /// Actual hardware products are incomplete or disagree with the actual current operands.
    Samples,
    /// Grapple, knockback or a room hook requires a different producer adapter.
    Unsupported,
}

impl Receipt {
    pub(super) fn follow(&self) -> Result<Model<CameraPin>, Fault> {
        let Self {
            previous,
            movement,
            inputs,
            ..
        } = self;
        if inputs.grapple == Activity::Active
            || inputs.knockback == Activity::Active
            || inputs.hook == Activity::Active
        {
            return Err(Fault::Unsupported);
        }
        let (samus, outputs) = movement.finished().ok_or(Fault::Movement)?;
        let Movement::Work(current) = movement else {
            return Err(Fault::Movement);
        };
        if current.model.0.start != previous.0.0 {
            return Err(Fault::Previous);
        }
        let room = inputs.room.model()?;
        let frame = world_camera::Frame {
            samus: world_camera::Samus {
                position: samus.position,
                placed: outputs
                    .placed
                    .map(|placed| types::PlacementY(placed.pixel().0)),
                facing: samus.pose.facing,
                backwards: outputs.skidding,
                rising: outputs.rising,
            },
            room: &room,
            lead: inputs.lead.model(),
            time_stopped: inputs.time_stopped,
        };
        Ok(Model(self.initial.0.step(&inputs.tuning.model(), &frame)))
    }
}

impl Stage {
    /// Run the actual clean follow from completed movement into a retained candidate.
    /// No candidate viewport is exposed to drawing before actual physical completion.
    /// # Errors
    /// Repeated choice, unfinished movement, wrong previous centre or invalid room.
    pub fn choose(
        &mut self,
        camera: &Model<CameraPin>,
        previous: &Previous,
        movement: &Movement,
        inputs: &Inputs,
    ) -> Result<&Receipt, Fault> {
        if camera.0.seen() != previous.0.0.position {
            return Err(Fault::Previous);
        }
        self.choose_from_current_history(camera, previous, movement, inputs)
    }

    pub(super) fn choose_from_current_history(
        &mut self,
        camera: &Model<CameraPin>,
        previous: &Previous,
        movement: &Movement,
        inputs: &Inputs,
    ) -> Result<&Receipt, Fault> {
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        let receipt = Receipt {
            initial: *camera,
            previous: previous.clone(),
            movement: movement.clone(),
            inputs: inputs.clone(),
        };
        let next = receipt.follow()?;
        *self = Self::Offered {
            receipt: Box::new(receipt),
            next,
        };
        match self {
            Self::Offered { receipt, .. } => Ok(receipt),
            Self::Idle | Self::Charging { .. } | Self::Completed(_) => Err(Fault::Order),
        }
    }

    /// Bind only after the caller joins this complete receipt to actual-current source predicates.
    /// A Camera family name alone never authorizes the full physical profile.
    /// # Errors
    /// No offered follow or a follow already charged/completed.
    pub fn bind(&mut self, work: WorkId) -> Result<(), Fault> {
        let Self::Offered { receipt, next } = self else {
            return Err(Fault::Order);
        };
        *self = Self::Charging {
            work,
            receipt: receipt.clone(),
            next: *next,
            samples: Samples::Empty,
        };
        Ok(())
    }

    /// Bind the actual current multiplication operands to the qualified whole template.
    /// # Errors
    /// No offered Camera operation or incompatible arithmetic publication/sample ordering.
    pub fn physical(&self, contract: &PhysicalContract) -> Result<data::timing::Profile, Fault> {
        let (Self::Offered { receipt, next } | Self::Charging { receipt, next, .. }) = self else {
            return Err(Fault::Order);
        };
        let template = contract.select(receipt, next)?;
        Requests::from_receipt(receipt)?.bind(template)
    }

    /// Retain one actual hardware result from this pending Camera operation.
    /// # Errors
    /// No pending charged operation, a different active work or a duplicate/reordered sample.
    pub fn sample(
        &mut self,
        expected: WorkId,
        device: data::timing::Sample,
        value: u8,
    ) -> Result<(), Fault> {
        let Self::Charging { work, samples, .. } = self else {
            return Err(Fault::Order);
        };
        if *work != expected {
            return Err(Fault::Completion);
        }
        samples.accept(device, value)
    }

    /// Publish the retained clean result once at the actual whole follow's completion.
    /// # Errors
    /// Wrong work identity, changed persistent camera or repeated completion.
    pub fn complete(
        &mut self,
        camera: &mut Model<CameraPin>,
        completed: WorkId,
    ) -> Result<&Receipt, Fault> {
        let Self::Charging {
            work,
            receipt,
            next,
            samples,
        } = self
        else {
            return Err(Fault::Order);
        };
        if *work != completed || camera != &receipt.initial {
            return Err(Fault::Completion);
        }
        let product = Requests::from_receipt(receipt)?.product();
        if samples.completed()? != [product, product] {
            return Err(Fault::Samples);
        }
        *camera = *next;
        *self = Self::Completed(receipt.clone());
        match self {
            Self::Completed(receipt) => Ok(receipt),
            Self::Idle | Self::Offered { .. } | Self::Charging { .. } => Err(Fault::Order),
        }
    }

    /// Actual completed viewport pixels, ready for drawing from the sole current camera.
    #[must_use]
    pub fn viewport(&self, camera: &Model<CameraPin>) -> Option<[u16; 2]> {
        if !matches!(self, Self::Completed(_)) {
            return None;
        }
        let position = camera.0.position();
        Some([position.x.pixel().0, position.y.pixel().0])
    }
}

//! Ordered gameplay requests and fresh movement receipts before downstream consumption.
use samus_appearance::{GraphicalContact, OpportunityOutput, SoundRequest};
use types::AnimationSound;

/// Fresh semantic work and accepted contact from the actual movement call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementOutputs {
    /// Accepted response with the speed before that response.
    pub floor_response: Option<types::FloorResponse>,
    /// Independent permission for appearance to evaluate landing graphics.
    pub touchdown_graphics: bool,
    /// Ordered evaluated leading-edge cells, with direction and caller purpose.
    pub collision_tests: types::CollisionTests,
    /// Actual attempts, considered candidates, support and body-fit decisions in order.
    pub work: Vec<types::MovementWork>,
    /// Ordinary admission, distinct from a request of zero travel.
    pub skip: Option<types::MotionSkipReason>,
    /// Caller roster at construction; absent for callers supplying only model outcomes.
    pub roster: Option<crate::samus_movement_bridge::roster::Roster>,
    /// Per-request joins; these carry no source timing admission or costs.
    pub searches: Vec<crate::samus_movement_bridge::roster::Search>,
}

/// Each distinct source request survives, in source order, including animation requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Accepted touchdown or wall kick, preserving transport metadata.
    Movement(types::SoundRequest),
    /// `WallJumpBranch` or `BoostingLap`.
    Animation(AnimationSound),
    /// Contact, selection, drawing or first passage.
    Appearance(SoundRequest),
}
/// Transient producer outputs; never an audio queue, retry or acknowledgement.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outputs {
    /// Actual movement publications for timing and graphical admission.
    pub movement: Option<MovementOutputs>,
    /// Ordered semantic requests for the later sound owner.
    pub requests: Vec<Request>,
    /// Graphical contacts for part 2, independent of sound suppression.
    pub contacts: Vec<GraphicalContact>,
}
impl Outputs {
    /// Publish the actual clean call without rebuilding work or suppressing repeated sounds.
    pub fn moved(&mut self, outcome: &samus_movement::Outcome) {
        self.movement_receipt(outcome);
        self.movement_requests(outcome);
    }

    /// Preserve source-admitted movement sounds at their phase, independently of work.
    pub fn movement_requests(&mut self, outcome: &samus_movement::Outcome) {
        self.requests.extend(
            outcome
                .touchdown_requests
                .iter()
                .copied()
                .map(Request::Movement),
        );
    }

    fn movement_receipt(&mut self, outcome: &samus_movement::Outcome) {
        self.movement = Some(MovementOutputs {
            floor_response: outcome.floor_response,
            touchdown_graphics: outcome.touchdown_graphics,
            collision_tests: outcome.collision_tests.clone(),
            work: outcome.movement_work.clone(),
            skip: outcome.movement_skip,
            roster: None,
            searches: Vec::new(),
        });
    }

    /// Deliver actual model publications alongside their owned caller population.
    pub fn moved_with_roster(
        &mut self,
        outcome: &samus_movement::Outcome,
        roster: &crate::samus_movement_bridge::roster::Roster,
    ) {
        self.work_with_roster(outcome, roster);
        self.movement_requests(outcome);
    }

    /// Publish every actual movement call even when appearance cannot admit its sounds.
    pub fn work_with_roster(
        &mut self,
        outcome: &samus_movement::Outcome,
        roster: &crate::samus_movement_bridge::roster::Roster,
    ) {
        self.movement_receipt(outcome);
        if let Some(movement) = &mut self.movement {
            movement.searches = roster.searches(&movement.work);
            movement.roster = Some(roster.clone());
        }
    }

    /// Preserve both independent outputs of an opportunity.
    pub fn retain(&mut self, output: OpportunityOutput) {
        self.requests.extend(output.sound.map(Request::Appearance));
        self.contacts.extend(output.contact);
    }
    /// Preserve an animation request at its own phase.
    pub fn animation(&mut self, sound: Option<AnimationSound>) {
        self.requests.extend(sound.map(Request::Animation));
    }
}

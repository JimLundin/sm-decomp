//! Current Health, Drawing history and ordinary HUD in an admitted tail-contact opportunity.

use super::{
    AdmissionFault, ProductionFault, Unsupported,
    frame::{self, frame_history::EnergyRead, hud::Hud},
};
use canonical::{Canonical, model::Model};
use interface_hud::{
    Ammunition, Dropped, Energy, FrameCount, HudOpportunity, HudPictures, HudState,
    ItemControlInputs, MiniMapPower, MovementInputs, Reserve, Room, StatusInputs,
};
use samus_health::{ProtectionFrames, Rules, TailImpact, WarningSound};

const CERES_TAIL_OFFER: u16 = 15;

/// Checked HUD content for the scoped caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Identity of the independently authenticated initial quantities.
    pub origin_identity: [u8; 32],
    /// Imported HUD pictures.
    pub pictures: HudPictures,
    /// Imported map for the current area.
    pub map: interface_hud::HerMap,
}

/// The independently observed running clock and equipment for Maintenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    /// The current clock is stopped.
    pub time_stopped: bool,
    /// Current Varia equipment flag.
    pub varia: bool,
    /// Current Gravity equipment flag.
    pub gravity: bool,
}

/// The current room facts at the powered-off map opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapInput {
    /// Current player position after movement.
    pub position: types::Pair<u16>,
    /// Current room on the map, with its bounds.
    pub room: Room,
    /// Current map collection fact.
    pub map_collected: bool,
}

/// The next causal action at a supported scoped gameplay opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// Open the opportunity at actual HUD controls.
    HudControls {
        /// Number of completed opportunities since admission.
        opportunity: u64,
        /// Independently observed current HUD clock.
        frame: FrameCount,
        /// Actual independent current control inputs.
        controls: ItemControlInputs,
    },
    /// Answer the actor's current query and admit the actual Maintenance context.
    Protection(Context),
    /// Deliver the external actor's actual whole-energy argument.
    Impact(u16),
    /// Apply ordinary map work in the independently qualified powered-off scope.
    HudMap(MapInput),
    /// Complete the admitted Health Maintenance and Warning opportunity.
    Maintenance,
    /// First Drawing energy consumer, with prior Drawing history.
    FirstDrawingRead,
    /// Second Drawing energy consumer commits the new history.
    SecondDrawingRead,
    /// The late status opportunity reads current Health.
    Status {
        /// Capacity observed at its actual status read.
        capacity: u16,
        /// Actual reserve status.
        reserve: Reserve,
        /// Actual status ammunition.
        ammunition: Ammunition,
    },
    /// Independently qualified active protection ageing after status.
    ActiveClose,
}

/// One output at each actual read or owner transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    /// Early HUD controls completed without changing the selection.
    HudControls,
    /// Current complete Health at the actor's query.
    Protection(samus_health::State),
    /// Current complete Health after a delivered impact.
    Impact(samus_health::State),
    /// Early map work completed without enabling the map.
    HudMap,
    /// Current Health and its request to the Warning receiver.
    Maintenance(samus_health::State, WarningSound),
    /// The live energy comparison and protection at the first Drawing read.
    FirstDrawingRead(EnergyRead, ProtectionFrames),
    /// Whole energy, newly committed Drawing history and protection.
    SecondDrawingRead(u16, u16, ProtectionFrames),
    /// Live energy, previous cache and whether the shown energy cache refreshed.
    Status(u16, u16, bool),
    /// Protection after the single qualified close.
    ActiveClose(ProtectionFrames),
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    HudControls,
    ActorQuery,
    ActorDelivery,
    ActorDelivered,
    Maintenance,
    FirstDrawingRead,
    SecondDrawingRead,
    Status,
    ActiveClose,
}

/// Complete carried owners and the next opportunity, including suspended actor delivery.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    origin_identity: [u8; 32],
    health: Model<frame::health::HealthPin>,
    hud: Hud,
    previous_draw_energy: u16,
    completed: u64,
    phase: Phase,
}

impl State {
    /// Admit one origin before any selected work.
    #[must_use]
    pub fn new(
        origin_identity: [u8; 32],
        health: samus_health::State,
        hud: HudState,
        previous_draw_energy: u16,
    ) -> Self {
        Self {
            origin_identity,
            health: Model(health),
            hud: Hud::new(hud),
            previous_draw_energy,
            completed: 0,
            phase: Phase::HudControls,
        }
    }

    pub(super) fn validate(&self, content: &Content) -> Result<(), AdmissionFault> {
        if self.origin_identity != content.origin_identity {
            return Err(AdmissionFault::HealthTailOrigin);
        }
        let expected = match self.phase {
            Phase::HudControls | Phase::ActiveClose => None,
            Phase::ActorQuery | Phase::ActorDelivery | Phase::ActorDelivered => {
                Some(HudOpportunity::MiniMapUpdate)
            }
            Phase::Maintenance
            | Phase::FirstDrawingRead
            | Phase::SecondDrawingRead
            | Phase::Status => Some(HudOpportunity::StatusRefresh),
        };
        let joined = match &self.hud {
            Hud::Initial(_) => expected.is_none(),
            Hud::Work(_) => expected.is_some() && self.hud.opportunity() == expected,
            Hud::Countdown { .. } => false,
        };
        if !joined {
            return Err(AdmissionFault::HealthTailPhase);
        }
        Ok(())
    }

    /// Complete carried HUD including gauge publication and cache.
    #[must_use]
    pub fn hud(&self) -> &HudState {
        self.hud.state()
    }

    /// Complete carried Health including its current protection.
    #[must_use]
    pub const fn health(&self) -> samus_health::State {
        self.health.0
    }

    /// Most recent committed Drawing energy.
    #[must_use]
    pub const fn previous_draw_energy(&self) -> u16 {
        self.previous_draw_energy
    }

    fn hud_controls(
        &mut self,
        frame: FrameCount,
        inputs: ItemControlInputs,
    ) -> Result<(), ProductionFault> {
        let state = self.hud.state();
        if state.selection != interface_hud::Selection::None
            || state.shown.highlighted.is_some()
            || state.mini_map != MiniMapPower::Off
            || inputs.select != interface_hud::SelectButton::NotPressed
            || inputs.cancel != interface_hud::CancelButton::NotPressed
            || !self.hud.begin_staged(frame)
            || !self.hud.choose_controls(inputs)
            || !self.hud.apply()
            || !self.hud.events().is_empty()
        {
            return Err(missing());
        }
        Ok(())
    }

    fn hud_map(&mut self, content: &Content, input: MapInput) -> Result<(), ProductionFault> {
        let inputs = MovementInputs {
            runs: true,
            turned_on: false,
            mother_brain_falls: false,
            position: input.position,
            room: input.room,
            map: content.map.clone(),
            map_collected: input.map_collected,
        };
        if self.hud.state().mini_map != MiniMapPower::Off
            || !self
                .hud
                .choose_map(&interface_hud::Rules::default(), &content.pictures, inputs)
            || !self.hud.apply()
        {
            return Err(missing());
        }
        Ok(())
    }

    fn hud_status(
        &mut self,
        content: &Content,
        inputs: StatusInputs,
    ) -> Result<Output, ProductionFault> {
        let prior = self.hud.state().shown.energy;
        if !self.hud.choose_ordinary_status(
            &interface_hud::Rules::default(),
            &content.pictures,
            inputs,
        ) || !self.hud.apply()
        {
            return Err(missing());
        }
        let (hud, events) = self.hud.publication().ok_or_else(missing)?;
        if !events.is_empty() {
            return Err(missing());
        }
        let redrawn = hud.shown.energy != prior;
        self.hud = Hud::new(hud.clone());
        Ok(Output::Status(inputs.energy.now, prior, redrawn))
    }

    pub(super) fn advance(
        &mut self,
        content: &Content,
        rules: Rules,
        inputs: &[Input],
    ) -> Result<Vec<Output>, ProductionFault> {
        inputs
            .iter()
            .map(|step| self.step(content, rules, *step))
            .collect()
    }

    fn step(
        &mut self,
        content: &Content,
        rules: Rules,
        input: Input,
    ) -> Result<Output, ProductionFault> {
        let (next, output) = match (self.phase, input) {
            (
                Phase::HudControls,
                Input::HudControls {
                    opportunity,
                    frame,
                    controls,
                },
            ) if opportunity == self.completed => {
                self.hud_controls(frame, controls)?;
                (Phase::ActorQuery, Output::HudControls)
            }
            (Phase::ActorQuery, Input::Protection(clock_and_suits))
                if !clock_and_suits.time_stopped
                    && !clock_and_suits.varia
                    && !clock_and_suits.gravity =>
            {
                (Phase::ActorDelivery, Output::Protection(self.health.0))
            }
            (Phase::ActorDelivery, Input::Impact(CERES_TAIL_OFFER)) => {
                self.health.0.tail_contact(TailImpact::EligibleCeresTail);
                (Phase::ActorDelivered, Output::Impact(self.health.0))
            }
            (Phase::ActorDelivery | Phase::ActorDelivered, Input::HudMap(input)) => {
                self.hud_map(content, input)?;
                (Phase::Maintenance, Output::HudMap)
            }
            (Phase::Maintenance, Input::Maintenance) => {
                let warning = self.health.0.health_opportunity(&rules);
                (
                    Phase::FirstDrawingRead,
                    Output::Maintenance(self.health.0, warning),
                )
            }
            (Phase::FirstDrawingRead, Input::FirstDrawingRead) => {
                let read = EnergyRead::compare(&self.health.0, self.previous_draw_energy);
                (
                    Phase::SecondDrawingRead,
                    Output::FirstDrawingRead(read, self.health.0.protection),
                )
            }
            (Phase::SecondDrawingRead, Input::SecondDrawingRead) => {
                let energy =
                    EnergyRead::read_and_commit(&self.health.0, &mut self.previous_draw_energy);
                (
                    Phase::Status,
                    Output::SecondDrawingRead(
                        energy,
                        self.previous_draw_energy,
                        self.health.0.protection,
                    ),
                )
            }
            (
                Phase::Status,
                Input::Status {
                    capacity,
                    reserve,
                    ammunition,
                },
            ) => {
                let output = self.hud_status(
                    content,
                    StatusInputs {
                        energy: Energy {
                            now: self.health.0.energy.whole.get(),
                            capacity,
                        },
                        reserve,
                        ammunition,
                        muted_by: None,
                        collected: None,
                        dropped_before_status_bar: Dropped::Nothing,
                        dropped_after_status_bar: Dropped::Nothing,
                    },
                )?;
                (Phase::ActiveClose, output)
            }
            (Phase::ActiveClose, Input::ActiveClose) => {
                self.completed = self.completed.checked_add(1).ok_or_else(missing)?;
                self.health.0.close_active_opportunity();
                (
                    Phase::HudControls,
                    Output::ActiveClose(self.health.0.protection),
                )
            }
            _ => return Err(missing()),
        };
        self.phase = next;
        Ok(output)
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::HealthTailOpportunity)
}

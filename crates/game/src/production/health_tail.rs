//! Current Health, Drawing history and ordinary HUD in an admitted tail-contact opportunity.

use super::{AdmissionFault, ProductionFault, Unsupported, hud::HudPin};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use interface_hud::{
    Ammunition, CancelButton, Controls, CountdownRuns, Dropped, Energy, FrameCount, FrameInputs,
    HudPictures, HudState, MiniMapInputs, MiniMapPower, Reserve, Room, SelectButton, Selection,
    Status, StockAtStart,
};
use samus_health::{ProtectionFrames, Rules, TailImpact, WarningSound};

const CERES_TAIL_OFFER: u16 = 15;

/// Complete health quantities; delivery and neighbouring qualification stay outside this pin.
pub struct HealthPin;
impl Pinned for HealthPin {
    type State = samus_health::State;
    const NAME: &'static str = "production-current-samus-health";
    const SHAPE: u64 = 0x2FD6_C95A_B0A2_9544;
}

/// Drawing's signed comparison of current and previously drawn energy.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnergyChange {
    /// Current whole energy is lower at the Drawing read.
    Decreased,
    /// The current whole energy equals the previous Drawing history.
    Held,
    /// The signed comparison admits an increase.
    Increased,
}

/// Current whole Health energy and the Drawing history compared with it.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnergyRead {
    /// Current whole energy read from Health.
    pub current: u16,
    /// Prior Drawing energy before the second read commits it.
    pub previous: u16,
    /// The comparison at the first Drawing read.
    pub change: EnergyChange,
}

impl EnergyRead {
    pub(in crate::production) const fn compare(
        health: &samus_health::State,
        previous: u16,
    ) -> Self {
        let current = health.energy.whole.get();
        let change = if current.wrapping_sub(previous) & (1 << 15) != 0 {
            EnergyChange::Decreased
        } else if current == previous {
            EnergyChange::Held
        } else {
            EnergyChange::Increased
        };
        Self {
            current,
            previous,
            change,
        }
    }

    pub(in crate::production) const fn read_and_commit(
        health: &samus_health::State,
        previous: &mut u16,
    ) -> u16 {
        let current = health.energy.whole.get();
        *previous = current;
        current
    }
}

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

/// The current room facts the powered-off mini-map reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapInput {
    /// Her position after her moves.
    pub position: types::Pair<u16>,
    /// Current room on the map, with its bounds.
    pub room: Room,
    /// Current map collection fact.
    pub map_collected: bool,
}

/// The HUD's neighbours' facts for its whole frame, read once the frame's Health is known.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudFrame {
    /// Number of completed opportunities since admission.
    pub opportunity: u64,
    /// The frame count as the frame begins.
    pub frame: FrameCount,
    /// Her stock as the frame begins.
    pub stock_at_start: StockAtStart,
    /// The item controls; this scope admits them only released.
    pub controls: Controls,
    /// What the powered-off mini-map reads.
    pub map: MapInput,
    /// Her energy capacity as the frame ends.
    pub capacity: u16,
    /// Her reserve as the frame ends.
    pub reserve: Reserve,
    /// Her ammunition as the frame ends.
    pub ammunition: Ammunition,
    /// Whether the scene runs the escape countdown this frame.
    pub countdown: CountdownRuns,
}

/// The next causal action at a supported scoped gameplay opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// Answer the actor's current query and admit the actual Maintenance context.
    Protection(Context),
    /// Deliver the external actor's actual whole-energy argument.
    Impact(u16),
    /// Complete the admitted Health Maintenance and Warning opportunity.
    Maintenance,
    /// First Drawing energy consumer, with prior Drawing history.
    FirstDrawingRead,
    /// Second Drawing energy consumer commits the new history.
    SecondDrawingRead,
    /// The HUD plays its whole frame, its status bar reading current Health.
    Status(HudFrame),
    /// Independently qualified active protection ageing after status.
    ActiveClose,
}

/// One output at each actual read or owner transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    /// Current complete Health at the actor's query.
    Protection(samus_health::State),
    /// Current complete Health after a delivered impact.
    Impact(samus_health::State),
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
    ActorQuery,
    ActorDelivery,
    ActorDelivered,
    FirstDrawingRead,
    SecondDrawingRead,
    Status,
    ActiveClose,
}

/// Complete carried owners and the next opportunity, including suspended actor delivery.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    origin_identity: [u8; 32],
    health: Model<HealthPin>,
    hud: Model<HudPin>,
    previous_draw_energy: u16,
    completed: u64,
    phase: Phase,
}

impl State {
    /// Admit one origin before any selected work.
    #[must_use]
    pub const fn new(
        origin_identity: [u8; 32],
        health: samus_health::State,
        hud: HudState,
        previous_draw_energy: u16,
    ) -> Self {
        Self {
            origin_identity,
            health: Model(health),
            hud: Model(hud),
            previous_draw_energy,
            completed: 0,
            phase: Phase::ActorQuery,
        }
    }

    pub(super) fn validate(&self, content: &Content) -> Result<(), AdmissionFault> {
        if self.origin_identity != content.origin_identity {
            return Err(AdmissionFault::HealthTailOrigin);
        }
        Ok(())
    }

    /// Complete carried HUD including gauge publication and cache.
    #[must_use]
    pub const fn hud(&self) -> &HudState {
        &self.hud.0
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

    /// The HUD's whole frame. The scope holds a quiet HUD: nothing selected or highlighted,
    /// the mini-map off, the item controls released and no event raised.
    fn hud_frame(
        &mut self,
        content: &Content,
        input: &HudFrame,
    ) -> Result<Output, ProductionFault> {
        let prior = &self.hud.0;
        if input.opportunity != self.completed
            || prior.selection != Selection::None
            || prior.shown.highlighted.is_some()
            || prior.mini_map != MiniMapPower::Off
            || input.controls.select != SelectButton::NotPressed
            || input.controls.cancel != CancelButton::NotPressed
        {
            return Err(missing());
        }
        let energy = Energy {
            now: self.health.0.energy.whole.get(),
            capacity: input.capacity,
        };
        let mut hud = prior.clone();
        let outputs = hud
            .frame_of_play(
                &interface_hud::Rules::default(),
                &content.pictures,
                &FrameInputs {
                    frame: input.frame,
                    stock_at_start: input.stock_at_start,
                    controls: input.controls,
                    mini_map: MiniMapInputs {
                        runs: true,
                        turned_on: false,
                        position: input.map.position,
                        room: input.map.room,
                        map: &content.map,
                        map_collected: input.map.map_collected,
                    },
                    mother_brain_falls: false,
                    status: Status {
                        energy,
                        reserve: input.reserve,
                        ammunition: input.ammunition,
                    },
                    collected: None,
                    muted_by: None,
                    dropped_before_status_bar: Dropped::Nothing,
                    dropped_after_status_bar: Dropped::Nothing,
                    countdown: input.countdown,
                },
            )
            .map_err(|_| missing())?;
        if !outputs.events.is_empty() {
            return Err(missing());
        }
        let shown_before = prior.shown.energy;
        let redrawn = hud.shown.energy != shown_before;
        self.hud = Model(hud);
        Ok(Output::Status(energy.now, shown_before, redrawn))
    }

    pub(super) fn advance(
        &mut self,
        content: &Content,
        rules: Rules,
        inputs: &[Input],
    ) -> Result<Vec<Output>, ProductionFault> {
        inputs
            .iter()
            .map(|step| self.step(content, rules, step))
            .collect()
    }

    fn step(
        &mut self,
        content: &Content,
        rules: Rules,
        input: &Input,
    ) -> Result<Output, ProductionFault> {
        let (next, output) = match (self.phase, input) {
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
            (Phase::ActorDelivery | Phase::ActorDelivered, Input::Maintenance) => {
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
            (Phase::Status, Input::Status(hud)) => {
                (Phase::ActiveClose, self.hud_frame(content, hud)?)
            }
            (Phase::ActiveClose, Input::ActiveClose) => {
                self.completed = self.completed.checked_add(1).ok_or_else(missing)?;
                self.health.0.close_active_opportunity();
                (
                    Phase::ActorQuery,
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

//! Current Health, Drawing's energy history and the ordinary HUD in the scoped Ceres tail, played
//! one whole frame at a time.

use super::{AdmissionFault, ProductionFault, Unsupported, hud::HudPin};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use interface_hud::{
    Ammunition, CancelButton, Controls, CountdownInputs, Dropped, Energy, FrameCount, FrameInputs,
    HudPictures, HudState, MiniMapInputs, MiniMapPower, Reserve, Room, SelectButton, Selection,
    Status, StockAtStart,
};
use samus_health::{ProtectionFrames, Rules, TailImpact, WarningSound};

/// Complete health quantities; neighbouring qualification stays outside this pin.
pub struct HealthPin;
impl Pinned for HealthPin {
    type State = samus_health::State;
    const NAME: &'static str = "production-current-samus-health";
    const SHAPE: u64 = 0x2FD6_C95A_B0A2_9544;
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

/// The independently observed clock and equipment that admit a frame to Health.
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

/// The HUD's neighbours' facts for its whole frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudFrame {
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
    /// What the escape countdown reads this frame: whether the scene runs it, its start and the
    /// count of finished frames.
    pub countdown: CountdownInputs,
}

/// One whole frame's independently observed inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Frames played since admission, which must match the carried count.
    pub played: u64,
    /// The clock and equipment that admit the frame.
    pub context: Context,
    /// Ridley's tail impact this frame.
    pub tail: TailImpact,
    /// The HUD's neighbours this frame.
    pub hud: HudFrame,
}

/// One played frame's outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Output {
    /// Health's request to the Warning receiver.
    pub warning_sound: WarningSound,
    /// The protection Drawing reads for the flash after damage.
    pub protection_drawn: ProtectionFrames,
    /// Whether the HUD redrew its energy gauge.
    pub gauge_redrawn: bool,
}

/// Complete carried owners from one authenticated origin.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    origin_identity: [u8; 32],
    health: Model<HealthPin>,
    hud: Model<HudPin>,
    drawn_energy: u16,
    played: u64,
}

impl State {
    /// Admit one origin before any frame.
    #[must_use]
    pub const fn new(
        origin_identity: [u8; 32],
        health: samus_health::State,
        hud: HudState,
        drawn_energy: u16,
    ) -> Self {
        Self {
            origin_identity,
            health: Model(health),
            hud: Model(hud),
            drawn_energy,
            played: 0,
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

    /// The whole energy Drawing last drew, which it compares with the next frame's.
    #[must_use]
    pub const fn drawn_energy(&self) -> u16 {
        self.drawn_energy
    }

    /// Plays one whole frame: Health, then Drawing's energy history, then the HUD's frame of
    /// play reading the frame's Health. Nothing is committed unless the whole frame is
    /// supported.
    pub(super) fn advance(
        &mut self,
        content: &Content,
        rules: Rules,
        frame: &Frame,
    ) -> Result<Output, ProductionFault> {
        let Context {
            time_stopped,
            varia,
            gravity,
        } = frame.context;
        if frame.played != self.played || time_stopped || varia || gravity {
            return Err(missing());
        }
        let health = self.health.0.play_frame(frame.tail, &rules);
        let energy = health.state.energy.whole.get();
        let hud = self.hud_frame(content, &frame.hud, energy)?;
        let played = self.played.checked_add(1).ok_or_else(missing)?;
        let gauge_redrawn = hud.shown.energy != self.hud.0.shown.energy;
        self.health = Model(health.state);
        self.hud = Model(hud);
        self.drawn_energy = energy;
        self.played = played;
        Ok(Output {
            warning_sound: health.warning_sound,
            protection_drawn: health.protection_drawn,
            gauge_redrawn,
        })
    }

    /// The HUD's whole frame. The scope holds a quiet HUD: nothing selected or highlighted,
    /// the mini-map off, the item controls released and no event raised.
    fn hud_frame(
        &self,
        content: &Content,
        input: &HudFrame,
        energy: u16,
    ) -> Result<HudState, ProductionFault> {
        let prior = &self.hud.0;
        if prior.selection != Selection::None
            || prior.shown.highlighted.is_some()
            || prior.mini_map != MiniMapPower::Off
            || input.controls.select != SelectButton::NotPressed
            || input.controls.cancel != CancelButton::NotPressed
        {
            return Err(missing());
        }
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
                        energy: Energy {
                            now: energy,
                            capacity: input.capacity,
                        },
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
        Ok(hud)
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::HealthTailFrame)
}

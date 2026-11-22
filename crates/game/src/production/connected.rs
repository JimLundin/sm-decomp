//! One carried ordinary lifecycle through movement, appearance and camera.

mod appearance;
pub mod early_map;
pub mod inputs;
mod transition;
mod world;

use crate::production::{AppearanceContent, CameraRules, MovementContent, movement_work::Movement};
use crate::state::owned::{CameraPin, MovementPin};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use samus_appearance::{AnimationDecision, Figure, WorkStep};
use samus_movement::suspended::{LocalRule, MovementDecision};

/// Complete content for the connected selected owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Checked body/cannon content.
    pub movement: MovementContent,
    /// Checked clips and running paces.
    pub appearance: AppearanceContent,
}

/// Complete selected tuning bound to the saved context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Movement parameters.
    pub movement: samus_movement::Rules,
    /// Appearance parameters.
    pub appearance: samus_appearance::Rules,
    /// Camera parameters.
    pub camera: CameraRules,
}

/// Complete current playing Figure, independent of unmade selection.
pub struct PlayingPin;
impl Pinned for PlayingPin {
    type State = Figure;
    const NAME: &'static str = "production-connected-playing";
    const SHAPE: u64 = 0x4301_5F68_4F3E_58BA;
}

/// Complete actual Ahead query retained before movement.
pub struct AheadPin;
impl Pinned for AheadPin {
    type State = samus_appearance::Ahead;
    const NAME: &'static str = "production-connected-ahead";
    const SHAPE: u64 = 0xc3a4_0ebc_338e_3734;
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct Context {
    inputs: Model<inputs::InputsPin>,
    previous: Model<MovementPin>,
    playing: Model<PlayingPin>,
    ahead: Model<AheadPin>,
}

#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Ready,
    Motion(Box<Context>),
    MotionComplete(Box<Context>),
    Contact(Box<Context>),
    Animation(Box<Context>),
    AnimationComplete(Box<Context>),
    Pose(Box<Context>),
    PoseComplete(Box<Context>),
    Selection(Box<Context>),
    SelectedClip(Box<Context>),
    SelectedClipComplete(Box<Context>),
    RemainingMotion(Box<Context>),
    RemainingMotionComplete(Box<Context>),
    DashLoss(Box<Context>),
    DashLossComplete(Box<Context>),
    Camera(Box<Context>),
    EarlyMap(Box<Context>),
}

/// Complete selected owners and actual unfinished ordinary opportunity.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Sole movement owner, with all current request/answer publications.
    pub movement: Movement,
    /// Sole appearance owner and intent-delivery positions.
    pub appearance: crate::production::appearance::Work,
    /// Actually playing Figure; accepted selection remains distinct until published.
    pub playing: Model<PlayingPin>,
    /// Complete retained camera.
    pub camera: Model<CameraPin>,
    phase: Phase,
    completed: u64,
    receipts_delivered: u64,
    early_map: Option<Box<early_map::State>>,
}

/// The caller may yield after actual semantic work, without inventing hardware time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Advance {
    /// Preserve the next pending operation/answer after one semantic transition.
    Yield,
    /// Complete this actual ordinary opportunity.
    Opportunity,
    /// Stop after actual Motion and Contact, before Animation and pose placement.
    EarlyMap,
}

/// Actual ordinary progress is independent of record indices and comparison horizons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    /// Complete owner State retains unfinished work.
    Pending,
    /// One admitted ordinary opportunity completed.
    Complete,
}

/// Complete actual outputs newly published by this call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Actual opportunity progress.
    pub progress: Progress,
    /// Ordered Appearance/animation sound requests.
    pub sounds: Vec<samus_appearance::PublishedSound>,
    /// Ordered graphical Contact intents, even with receivers unselected.
    pub graphics: Vec<samus_appearance::GraphicalContact>,
    /// Actual model receipts in opportunity order, preserving each read's full output.
    pub appearance_receipts: Vec<samus_appearance::Receipt>,
    /// Complete movement outputs when this opportunity completed.
    pub movement: Option<samus_movement::Outcome>,
    /// Complete actual camera when this opportunity completed.
    pub camera: Option<world_camera::Camera>,
}

/// Unsupported current family, missing producer or invalid retained operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Wrong/missing ordinary progress or completion.
    Order,
    /// Changed/missing independently current stimuli.
    Inputs,
    /// Unqualified action, activity, liquid, booster or reaction family.
    Family,
    /// Movement did not provide the actual required current publication.
    Movement,
    /// Appearance work or intent-delivery progress is invalid.
    Appearance,
    /// Current or accepted Figure lacks the qualified content relation.
    Figure,
    /// Actual consumed pose initialization is unavailable or replaced.
    Publication,
    /// Final/prior movement or camera room/history is invalid.
    Camera,
    /// Missing or incompatible early map owner.
    HudMap,
}

impl State {
    /// Admit selected initial quantities without completing any operation.
    /// # Errors
    /// An incompatible playing/current pose has no qualified current owner.
    pub fn new(
        movement: samus_movement::Samus,
        appearance: samus_appearance::State,
        playing: Figure,
        camera: world_camera::Camera,
    ) -> Result<Self, Fault> {
        if playing.pose != movement.pose
            || playing.sprite != types::Sprite::Plain
            || movement.pose.action != types::Action::Running
        {
            return Err(Fault::Figure);
        }
        Ok(Self {
            movement: Movement::new(movement),
            appearance: crate::production::appearance::Work::new(
                samus_appearance::SuspendedAppearance::new(appearance, None, Vec::new()),
            ),
            playing: Model(playing),
            camera: Model(camera),
            phase: Phase::Ready,
            completed: 0,
            receipts_delivered: 0,
            early_map: None,
        })
    }

    /// Select early map ownership at admission before any ordinary work begins.
    /// # Errors
    /// An active or previously configured owner rejects replacement.
    pub fn with_early_map(mut self, hud: interface_hud::HudState) -> Result<Self, Fault> {
        if self.pending() || self.completed() != 0 || self.early_map.is_some() {
            return Err(Fault::HudMap);
        }
        self.early_map = Some(Box::new(early_map::State {
            hud: crate::production::frame::hud::Hud::new(hud),
            position: None,
        }));
        Ok(self)
    }

    /// Current scoped HUD and the latest early Movement coordinates when produced.
    #[must_use]
    pub fn early_map(&self) -> Option<(&interface_hud::HudState, Option<[u32; 2]>)> {
        self.early_map
            .as_ref()
            .map(|map| (map.hud.state(), map.position))
    }

    /// Number of genuinely completed ordinary opportunities.
    #[must_use]
    pub const fn completed(&self) -> u64 {
        self.completed
    }

    /// An actual ordinary opportunity remains unfinished.
    #[must_use]
    pub const fn pending(&self) -> bool {
        !matches!(self.phase, Phase::Ready)
    }

    /// Validate complete current owners and retained constructor state.
    /// # Errors
    /// Changed current owner, stale pending work or unsupported inputs reject.
    pub fn validate(&self, content: &Content) -> Result<(), Fault> {
        self.appearance.validate().map_err(|_| Fault::Appearance)?;
        if usize::try_from(self.receipts_delivered)
            .is_ok_and(|at| at > self.appearance.model.0.completed.len())
        {
            return Err(Fault::Appearance);
        }
        appearance::running(self.playing.0, &content.appearance)?;
        let at_map = matches!(self.phase, Phase::EarlyMap(_));
        if let Some(map) = &self.early_map {
            let position = self.movement.state().position;
            if map.position.is_some() != (at_map || self.completed > 0)
                || map.position.is_some() != map.hud.map_ready()
                || (at_map && map.position != Some([position.x.0, position.y.0]))
                || (self.pending() && !at_map)
            {
                return Err(Fault::HudMap);
            }
        } else if at_map {
            return Err(Fault::HudMap);
        }
        match &self.phase {
            Phase::Ready => {
                if self.appearance.model.0.pending.is_some()
                    || self.appearance.model.0.consumed != self.appearance.model.0.steps.len()
                {
                    return Err(Fault::Appearance);
                }
            }
            Phase::Motion(ctx)
            | Phase::MotionComplete(ctx)
            | Phase::Contact(ctx)
            | Phase::EarlyMap(ctx)
            | Phase::Animation(ctx)
            | Phase::AnimationComplete(ctx)
            | Phase::Pose(ctx)
            | Phase::PoseComplete(ctx)
            | Phase::Selection(ctx)
            | Phase::SelectedClip(ctx)
            | Phase::SelectedClipComplete(ctx)
            | Phase::RemainingMotion(ctx)
            | Phase::RemainingMotionComplete(ctx)
            | Phase::DashLoss(ctx)
            | Phase::DashLossComplete(ctx)
            | Phase::Camera(ctx) => {
                if ctx.inputs.0.activity != inputs::Activity::Inactive
                    || self.movement.state().pose.action != types::Action::Running
                {
                    return Err(Fault::Family);
                }
                let Movement::Work(work) = &self.movement else {
                    return Err(Fault::Movement);
                };
                if work.model.0.start != ctx.previous.0 {
                    return Err(Fault::Movement);
                }
            }
        }
        Ok(())
    }

    fn begin(&mut self, input: &inputs::Inputs, content: &Content) -> Result<(), Fault> {
        if input.activity != inputs::Activity::Inactive
            || input.items[types::Item::SpeedBooster]
            || input.liquid != samus_appearance::Liquid::NONE
        {
            return Err(Fault::Family);
        }
        appearance::running(self.playing.0, &content.appearance)?;
        input.geometry.validate().map_err(|_| Fault::Inputs)?;
        if usize::try_from(self.receipts_delivered).ok()
            != Some(self.appearance.model.0.completed.len())
        {
            return Err(Fault::Appearance);
        }
        self.appearance.retire().map_err(|_| Fault::Appearance)?;
        self.receipts_delivered = 0;
        let ctx = Context {
            inputs: Model(input.clone()),
            previous: Model(*self.movement.state()),
            playing: self.playing,
            ahead: Model(appearance::ahead(
                &self.appearance,
                self.playing.0,
                &content.appearance,
            )?),
        };
        self.movement.begin().map_err(|_| Fault::Movement)?;
        self.phase = Phase::Motion(Box::new(ctx));
        Ok(())
    }

    fn movement_operation(
        &mut self,
        ctx: &Context,
        content: &Content,
        rules: &Rules,
        complete: bool,
    ) -> Result<MovementDecision, Fault> {
        let guard = world::Guard {
            world: &ctx.inputs.0.geometry,
            rejected: std::cell::Cell::new(false),
        };
        let world: &dyn types::World = &guard;
        let radius = content
            .movement
            .radius(self.movement.state().pose)
            .ok_or(Fault::Figure)?;
        let body = |_| radius;
        let heading = |pose| content.movement.cannon_heading(pose);
        let frame = samus_movement::Frame {
            sprite_before_movement: ctx.playing.0.sprite,
            cinematic_active: matches!(
                ctx.inputs.0.cinematic,
                samus_appearance::CinematicActivity::Active
            ),
            pad: ctx.inputs.0.held,
            pressed: ctx.inputs.0.pressed,
            items: ctx.inputs.0.items,
            spin_turned_far: ctx.ahead.0.turned_far,
            elevator_moving: false,
            parity: ctx.inputs.0.parity,
            animation_ends: ctx.ahead.0.end,
            body: &body,
            cannon_heading: &heading,
        };
        if !complete {
            let offered = self
                .movement
                .offer(&content.movement, &rules.movement, &frame, world)
                .map_err(|_| Fault::Movement)?
                .ok_or(Fault::Movement)?;
            if guard.rejected.get() {
                return Err(Fault::Family);
            }
            return Ok(offered);
        }
        let Movement::Work(work) = &self.movement else {
            return Err(Fault::Movement);
        };
        let decision = work.model.0.offered.clone().ok_or(Fault::Movement)?;
        self.movement
            .complete(&content.movement, &rules.movement, &frame, world)
            .map_err(|_| Fault::Movement)?;
        if guard.rejected.get() {
            return Err(Fault::Family);
        }
        Ok(decision)
    }

    fn animation_data(
        &self,
        ctx: &Context,
        from: Figure,
        selected: Figure,
        content: &Content,
        rules: &Rules,
    ) -> Result<samus_appearance::AnimationInputs, Fault> {
        let input = appearance::animation_inputs(
            &self.movement,
            &ctx.previous.0,
            ctx.playing.0,
            selected,
            &ctx.inputs.0,
        )?;
        content
            .appearance
            .animation_inputs(input, from, rules.appearance.clone())
            .map_err(|_| Fault::Figure)
    }

    fn transition(&mut self, content: &Content, rules: &Rules) -> Result<(), Fault> {
        let phase = core::mem::take(&mut self.phase);
        self.phase = match phase {
            phase @ (Phase::Motion(_)
            | Phase::MotionComplete(_)
            | Phase::Pose(_)
            | Phase::PoseComplete(_)
            | Phase::RemainingMotion(_)
            | Phase::RemainingMotionComplete(_)) => self.movement_phase(phase, content, rules)?,
            phase @ (Phase::Contact(_)
            | Phase::EarlyMap(_)
            | Phase::Animation(_)
            | Phase::AnimationComplete(_)
            | Phase::Selection(_)
            | Phase::SelectedClip(_)
            | Phase::SelectedClipComplete(_)) => self.appearance_phase(phase, content, rules)?,
            phase @ (Phase::DashLoss(_) | Phase::DashLossComplete(_)) => self.dash_phase(phase)?,
            phase @ Phase::Camera(_) => self.camera_phase(phase, rules)?,
            Phase::Ready => return Err(Fault::Order),
        };
        Ok(())
    }

    /// Advance actual selected semantic work on a complete private candidate.
    /// # Errors
    /// Missing/current inputs, unsupported families or stale owners reject atomically.
    pub fn advance(
        &mut self,
        input: &inputs::Inputs,
        content: &Content,
        rules: &Rules,
        until: Advance,
    ) -> Result<Outputs, Fault> {
        self.advance_with_map(input, content, rules, until, None)
    }

    pub(super) fn advance_with_map(
        &mut self,
        input: &inputs::Inputs,
        content: &Content,
        rules: &Rules,
        until: Advance,
        map: Option<early_map::Inputs<'_>>,
    ) -> Result<Outputs, Fault> {
        let mut candidate = self.clone();
        candidate.validate(content)?;
        if candidate.early_map.is_some() != map.is_some()
            || until == Advance::EarlyMap && (map.is_none() || candidate.pending())
            || until == Advance::Yield && map.is_some()
        {
            return Err(Fault::HudMap);
        }
        if matches!(candidate.phase, Phase::Ready) {
            if let (Some(owner), Some(premises)) = (&mut candidate.early_map, map) {
                owner.begin(premises)?;
            }
            candidate.begin(input, content)?;
        } else {
            if candidate.context_inputs() != Some(input) {
                return Err(Fault::Inputs);
            }
            if let (Some(owner), Some(premises)) = (&candidate.early_map, map)
                && !owner.holds_applied(premises)
            {
                return Err(Fault::HudMap);
            }
        }
        let mut sounds = Vec::new();
        let mut graphics = Vec::new();
        let mut appearance_receipts = Vec::new();
        loop {
            candidate.transition(content, rules)?;
            if let (Phase::EarlyMap(_), Some(owner), Some(premises)) =
                (&candidate.phase, &mut candidate.early_map, map)
            {
                owner.publish(candidate.movement.state(), premises)?;
            }
            let delivered = candidate
                .appearance
                .deliver()
                .map_err(|_| Fault::Appearance)?;
            sounds.extend(delivered.sounds);
            graphics.extend(delivered.graphics);
            let at =
                usize::try_from(candidate.receipts_delivered).map_err(|_| Fault::Appearance)?;
            appearance_receipts.extend(
                candidate
                    .appearance
                    .model
                    .0
                    .completed
                    .iter()
                    .skip(at)
                    .cloned(),
            );
            candidate.receipts_delivered =
                u64::try_from(candidate.appearance.model.0.completed.len())
                    .map_err(|_| Fault::Appearance)?;
            if !candidate.pending()
                || until == Advance::Yield
                || until == Advance::EarlyMap && matches!(candidate.phase, Phase::EarlyMap(_))
            {
                break;
            }
        }
        candidate.validate(content)?;
        let ready = !candidate.pending();
        let outputs = Outputs {
            progress: if ready {
                Progress::Complete
            } else {
                Progress::Pending
            },
            sounds,
            graphics,
            appearance_receipts,
            movement: if ready {
                candidate.movement.finished().map(|(_, out)| out.clone())
            } else {
                None
            },
            camera: ready.then_some(candidate.camera.0),
        };
        *self = candidate;
        Ok(outputs)
    }

    fn context_inputs(&self) -> Option<&inputs::Inputs> {
        match &self.phase {
            Phase::Ready => None,
            Phase::Motion(ctx)
            | Phase::MotionComplete(ctx)
            | Phase::Contact(ctx)
            | Phase::EarlyMap(ctx)
            | Phase::Animation(ctx)
            | Phase::AnimationComplete(ctx)
            | Phase::Pose(ctx)
            | Phase::PoseComplete(ctx)
            | Phase::Selection(ctx)
            | Phase::SelectedClip(ctx)
            | Phase::SelectedClipComplete(ctx)
            | Phase::RemainingMotion(ctx)
            | Phase::RemainingMotionComplete(ctx)
            | Phase::DashLoss(ctx)
            | Phase::DashLossComplete(ctx)
            | Phase::Camera(ctx) => Some(&ctx.inputs.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pins_keep_complete_inputs_query_and_work() {
        use canonical::model::{pin_holds, shape};
        assert_eq!(
            (
                shape::<inputs::Inputs>(),
                shape::<samus_appearance::Ahead>(),
                pin_holds::<crate::production::appearance::WorkPin>(),
                pin_holds::<crate::production::movement_work::WorkPin>(),
                pin_holds::<crate::production::movement_work::ReceiptsPin>()
            ),
            (
                Ok(inputs::InputsPin::SHAPE),
                Ok(AheadPin::SHAPE),
                Ok(()),
                Ok(()),
                Ok(())
            )
        );
    }
}

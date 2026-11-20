//! Actual refresh readiness and complete Camera work, independent of caller scheduling.

use super::{camera_stage, hud::Hud, hud_stage, movement::Movement};
use crate::{production::timing::WorkId, state::owned::CameraPin};
use canonical::{Canonical, model::Model};
use data::timing::{Profile, Sample, Template, TemplateEffect};
use types::Parts;

/// Actual refresh producer identity, preceding its physical return tail.
pub const HUD_REFRESH: WorkId = WorkId(1200);
/// Remaining physical return exposure after actual HUD readiness.
pub const HUD_RETURN: WorkId = WorkId(1201);
/// Whole actual Camera follow including both arithmetic requests.
pub const CAMERA_FOLLOW: WorkId = WorkId(1202);

/// Complete immutable refresh source bill split at actual producer readiness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshPhysical {
    /// All refresh work through the actual terminal state publication.
    pub producer: Template,
    /// Remaining return work after full HUD state and tilemap are ready.
    pub return_tail: Template,
}

/// Refresh progress; the actual HUD owner retains all chosen inputs and answers.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refresh {
    /// No current display inputs have been observed.
    Pending,
    /// Chosen actual refresh awaits its full producer bill.
    Charging,
    /// Complete HUD state and tilemap are published; return is not yet charged.
    Ready,
    /// The completed producer's physical return remains active.
    Returning,
    /// Complete refresh and return tail have both finished.
    Complete,
}

/// Rejected order, current producer join or physical effect contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A chosen or completed owner cannot be repeated or completed under another identity.
    Order,
    /// Actual early controls or current movement dependency is missing or changed.
    Producers,
    /// The refresh bill contains hardware effects or cannot be bound.
    Profile,
    /// Actual fresh display admission or terminal publication failed.
    Hud(hud_stage::StageFault),
    /// Actual Camera admission, arithmetic continuation or completion failed.
    Camera(camera_stage::Fault),
}

impl RefreshPhysical {
    /// Bind complete effect-free refresh work without default publications.
    /// # Errors
    /// Either source segment contains a sample/publication or has an invalid shape.
    pub fn profiles(&self) -> Result<[Profile; 2], Fault> {
        Ok([quiet(&self.producer)?, quiet(&self.return_tail)?])
    }
}

fn quiet(template: &Template) -> Result<Profile, Fault> {
    if template
        .runs
        .iter()
        .any(|run| run.effect != TemplateEffect::None)
    {
        return Err(Fault::Profile);
    }
    template.bind(&[]).map_err(|_| Fault::Profile)
}

impl Refresh {
    /// Observe current inventory, events and actual movement only at the refresh opportunity.
    /// The enclosing caller qualifies its causal movement prefix before this entry.
    /// # Errors
    /// Wrong order, altered controls, stale current owners or incompatible source bill.
    pub fn choose(
        &mut self,
        hud: &mut Hud,
        movement: &Movement,
        controls: &hud_stage::ControlsCompletion,
        current: hud_stage::CurrentDisplay<'_>,
        content: (&interface_hud::Rules, &interface_hud::HudPictures),
        admitted: (&hud_stage::DisplayReceipt, &RefreshPhysical),
    ) -> Result<Profile, Fault> {
        if *self != Self::Pending {
            return Err(Fault::Order);
        }
        let Movement::Work(_) = movement else {
            return Err(Fault::Producers);
        };
        let samus = movement.state();
        if controls.work != WorkId(16)
            || !matches!(hud.controls(), Some(interface_hud::Observation::Completed(answer)) if answer.eq(&controls.answer.0))
            || current.position
                != (types::Pair {
                    x: samus.position.x.pixel().0,
                    y: samus.position.y.pixel().0,
                })
            || current.action != samus.pose.action
        {
            return Err(Fault::Producers);
        }
        let [producer, _] = admitted.1.profiles()?;
        let mut candidate = hud.clone();
        if candidate.opportunity() == Some(interface_hud::HudOpportunity::MiniMapUpdate) {
            if current.premises != &admitted.0.premises
                || current.event_owner != Some(&current.premises.events)
                || hud.state().selection != interface_hud::Selection::None
                || hud.state().mini_map != interface_hud::MiniMapPower::On
            {
                return Err(Fault::Producers);
            }
            let inputs = current
                .premises
                .inputs(current.status, current.position, current.action);
            if !candidate.choose_map(
                content.0,
                content.1,
                interface_hud::MovementInputs::capture(
                    &inputs.mini_map(),
                    inputs.mother_brain_falls,
                ),
            ) {
                return Err(Fault::Order);
            }
        } else {
            hud_stage::choose_admitted_display(
                &mut candidate,
                content.0,
                content.1,
                current,
                admitted.0,
            )
            .map_err(Fault::Hud)?;
        }
        *hud = candidate;
        *self = Self::Charging;
        Ok(producer)
    }

    /// Materialize the complete tilemap once, at the actual producer's returned identity.
    /// # Errors
    /// Wrong identity/order or incomplete actual chosen refresh.
    pub fn complete_producer(
        &mut self,
        hud: &mut Hud,
        work: WorkId,
    ) -> Result<hud_stage::DisplayPublication, Fault> {
        if *self != Self::Charging || work != HUD_REFRESH {
            return Err(Fault::Order);
        }
        let publication = if hud.opportunity() == Some(interface_hud::HudOpportunity::MiniMapUpdate)
        {
            if !hud.apply() || !hud.map_ready() {
                return Err(Fault::Order);
            }
            hud_stage::DisplayPublication {
                tilemap: super::hud::tilemap(hud.state()).map_err(|_| Fault::Order)?,
                events: hud.events().to_vec(),
            }
        } else {
            hud_stage::complete_display(hud).map_err(Fault::Hud)?
        };
        *self = Self::Ready;
        Ok(publication)
    }

    /// Charge the retained return only after actual full HUD producer readiness.
    /// # Errors
    /// Wrong order, missing actual publication or incompatible complete source bill.
    pub fn begin_return(
        &mut self,
        hud: &Hud,
        physical: &RefreshPhysical,
    ) -> Result<Profile, Fault> {
        if *self != Self::Ready || (hud.publication().is_none() && !hud.map_ready()) {
            return Err(Fault::Order);
        }
        let [_, tail] = physical.profiles()?;
        *self = Self::Returning;
        Ok(tail)
    }

    /// Complete the actual physical tail without repeating the HUD producer.
    /// # Errors
    /// Wrong returned identity or repeated/out-of-order completion.
    pub fn complete_return(&mut self, work: WorkId) -> Result<(), Fault> {
        if *self != Self::Returning || work != HUD_RETURN {
            return Err(Fault::Order);
        }
        *self = Self::Complete;
        Ok(())
    }
}

/// Choose and qualify the actual clean follow atomically before the caller starts its bill.
/// # Errors
/// Unfinished actual movement, incompatible current viewport or unsupported full physical contract.
pub fn choose_camera(
    stage: &mut camera_stage::Stage,
    camera: &Model<CameraPin>,
    previous: &camera_stage::Previous,
    movement: &Movement,
    inputs: &camera_stage::Inputs,
    contract: &camera_stage::PhysicalContract,
) -> Result<Profile, Fault> {
    let mut candidate = stage.clone();
    candidate
        .choose(camera, previous, movement, inputs)
        .map_err(Fault::Camera)?;
    let profile = candidate.physical(contract).map_err(Fault::Camera)?;
    candidate.bind(CAMERA_FOLLOW).map_err(Fault::Camera)?;
    *stage = candidate;
    Ok(profile)
}

/// Route an actual arithmetic result only to the active whole Camera owner.
/// # Errors
/// Another physical owner is active or the sample is duplicated/reordered.
pub fn sample_camera(
    stage: &mut camera_stage::Stage,
    active: WorkId,
    sample: Sample,
    value: u8,
) -> Result<(), Fault> {
    if active != CAMERA_FOLLOW {
        return Err(Fault::Order);
    }
    stage.sample(active, sample, value).map_err(Fault::Camera)
}

/// Commit the sole current viewport only at actual whole Camera completion.
/// # Errors
/// Wrong owner, changed viewport authority or incomplete actual hardware products.
pub fn complete_camera(
    stage: &mut camera_stage::Stage,
    camera: &mut Model<CameraPin>,
    work: WorkId,
) -> Result<(), Fault> {
    if work != CAMERA_FOLLOW {
        return Err(Fault::Order);
    }
    stage.complete(camera, work).map_err(Fault::Camera)?;
    Ok(())
}

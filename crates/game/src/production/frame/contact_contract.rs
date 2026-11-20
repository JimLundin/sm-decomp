//! Fresh stride qualification and independent graphical-contact lifetime.

use super::{
    State,
    appearance_caller::{Kind, Receipt, Stage},
    sender_contract::BusRegime,
};
use crate::production::AppearanceContent;
use canonical::model::Pinned;
use samus_appearance::{AppearanceRead, ContactSuppressor, GraphicalContact, SoundRequest};

/// Current liquid surface supplied by its owner.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    /// No surface is active.
    Absent,
    /// An active surface needs a separately qualified graphical effect.
    Present,
}

/// Current room and liquid owners, independent of the sound suppressors.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    /// Actual imported room identity.
    #[serde(with = "RoomId")]
    pub room: data::id::RoomId,
    /// Actual current area.
    #[serde(with = "Area")]
    pub area: types::Area,
    /// Current water surface activity.
    pub water: Surface,
    /// Current lava or acid surface activity.
    pub lava: Surface,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "data::id::RoomId", transparent)]
struct RoomId(u16);

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "types::Area")]
enum Area {
    Brinstar,
    Ceres,
    Crateria,
    Maridia,
    Norfair,
    Tourian,
    WreckedShip,
}

/// Explicit current dependencies at the stride opportunity.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Current {
    /// Cinematic, boss and special-colour owners suppress sound only.
    pub suppressors: types::Set<ContactSuppressor>,
    /// Current graphical context.
    pub context: Context,
    /// Current instruction-fetch regime.
    pub bus: BusRegime,
}

/// Complete current Contact dependencies carried by the engine checkpoint.
pub struct CurrentPin;
impl Pinned for CurrentPin {
    type State = Current;
    const NAME: &'static str = "production-contact-current";
    const SHAPE: u64 = 0x3BA5_6ADF_9877_BCFA;
}

#[cfg(test)]
mod pin_tests {
    #[test]
    fn current_contact_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<super::CurrentPin>(), Ok(()));
    }
}

/// Unsupported current family, stale receipt or incomplete physical envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Missing or stale actual Contact read.
    Receipt,
    /// An actual current dependency is outside the qualified arm.
    Current,
    /// A physical effect or intrinsic duration differs.
    Profile,
    /// Actual completed output identity differs.
    Output,
}

/// Independently accepted whole Contact envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContract {
    template: data::timing::Template,
}

/// Whole-owner qualification, retaining actual inputs and outputs through charging.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qualified {
    /// One whole Contact charge, never an additional appearance update.
    pub profile: data::timing::Profile,
    /// Actual retained Contact receipt.
    pub receipt: Receipt,
    /// Actual graphical output available to the later Part2 owner.
    pub graphic: GraphicalContact,
    /// Actual semantic sound request; transport owns admission.
    pub sound: SoundRequest,
    current: Current,
}

impl PhysicalContract {
    /// Retain an independently qualified complete envelope.
    #[must_use]
    pub const fn new(template: data::timing::Template) -> Self {
        Self { template }
    }

    /// Qualify the actual pre-animation Contact read and its complete clean result.
    /// # Errors
    /// Unsupported current arm, changed checked clip or incomplete physical envelope.
    pub fn qualify(
        &self,
        stage: &Stage,
        frame: &State,
        content: &AppearanceContent,
        current: Current,
    ) -> Result<Qualified, Fault> {
        stage.validate(frame).map_err(|_| Fault::Receipt)?;
        let receipt = stage.receipt().ok_or(Fault::Receipt)?;
        let graphic = contact(receipt, content, current)?;
        if frame.movement.state().pose != graphic.figure.pose {
            return Err(Fault::Current);
        }
        let mut ticks = 0_u64;
        for run in &self.template.runs {
            if !matches!(run.effect, data::timing::TemplateEffect::None) {
                return Err(Fault::Profile);
            }
            ticks = ticks
                .checked_add(u64::from(run.count).saturating_mul(u64::from(run.cycle.ticks())))
                .ok_or(Fault::Profile)?;
        }
        if ticks != 840 {
            return Err(Fault::Profile);
        }
        let profile = self.template.bind(&[]).map_err(|_| Fault::Profile)?;
        Ok(Qualified {
            profile,
            receipt: receipt.clone(),
            graphic,
            sound: SoundRequest::Footstep,
            current,
        })
    }
}

fn contact(
    receipt: &Receipt,
    content: &AppearanceContent,
    current: Current,
) -> Result<GraphicalContact, Fault> {
    receipt.validate().map_err(|_| Fault::Receipt)?;
    let work = &receipt.chosen.model.0;
    let Some(samus_appearance::WorkStep::Appearance(AppearanceRead::Contact {
        figure,
        playhead,
        stage,
        clip,
        suppressors,
    })) = work.steps.get(work.consumed)
    else {
        return Err(Fault::Receipt);
    };
    if receipt.kind != Kind::Contact
        || *figure != receipt.playing.0
        || figure.pose.action != types::Action::Running
        || *playhead != receipt.before.model.0.state.playhead
        || playhead.time_left != samus_appearance::Frames::LAST
        || *stage != types::BoostStage::Stage0
        || *stage != receipt.before.model.0.state.stage
        || content.get(*figure) != Some(clip)
        || clip.hold(playhead.step).is_none()
        || !clip.contacts.contains(&playhead.step)
        || *suppressors != current.suppressors
        || suppressors.values().any(|active| *active)
        || current.context.area != types::Area::Ceres
        || current.context.water != Surface::Absent
        || current.context.lava != Surface::Absent
        || current.bus != BusRegime::Fast
    {
        return Err(Fault::Current);
    }
    let graphic = GraphicalContact {
        figure: *figure,
        playhead: *playhead,
    };
    let Some(samus_appearance::Receipt::Appearance { output, .. }) =
        receipt.result.model.0.completed.last()
    else {
        return Err(Fault::Output);
    };
    if output.contact != Some(graphic) || output.sound != Some(SoundRequest::Footstep) {
        return Err(Fault::Output);
    }
    Ok(graphic)
}

impl Qualified {
    /// Revalidate a held answer immediately before owner completion, without repeating it.
    /// # Errors
    /// Replaced actual owners, content or receipt.
    pub fn validate(
        &self,
        stage: &Stage,
        frame: &State,
        content: &AppearanceContent,
        current: Current,
    ) -> Result<(), Fault> {
        stage.validate(frame).map_err(|_| Fault::Receipt)?;
        if current != self.current
            || stage.receipt() != Some(&self.receipt)
            || contact(&self.receipt, content, current)? != self.graphic
            || frame.movement.state().pose != self.graphic.figure.pose
        {
            return Err(Fault::Current);
        }
        Ok(())
    }

    /// Confirm the actual completed Contact publication before retaining its graphic.
    /// Sound suppressors are not consulted by the later graphical consumer.
    /// # Errors
    /// A different receipt, physical identity or actual output was returned.
    pub fn completed(
        &self,
        completion: &super::appearance_caller::Completion,
        output: &crate::production::appearance::Outputs,
    ) -> Result<(), Fault> {
        if completion.work != Kind::Contact.identity()
            || completion.receipt != self.receipt
            || output.graphics.as_slice() != [self.graphic]
            || output.sounds.as_slice()
                != [samus_appearance::PublishedSound::Appearance(self.sound)]
        {
            return Err(Fault::Output);
        }
        Ok(())
    }

    /// Revalidate graphical dependencies at the actual Part2 consuming opportunity.
    /// # Errors
    /// Replaced graphic, room/liquid context or active boosting requires another arm.
    pub fn graphical(
        &self,
        graphic: GraphicalContact,
        context: Context,
        stage: types::BoostStage,
    ) -> Result<(), Fault> {
        if graphic != self.graphic
            || context != self.current.context
            || stage == types::BoostStage::Boosting
        {
            return Err(Fault::Current);
        }
        Ok(())
    }
}

//! Physical background arithmetic from the actual completed Camera.

use super::{Content, Dependency, State, suffix_caller};
use crate::production::{ProductionFault, Unsupported, timing};
use data::timing::Template;

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::BackgroundPreparation))
}

/// Independently admitted scroll context and its complete physical envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    /// Current room-owned offsets and enabled scroll dispositions.
    pub inputs: suffix_caller::BackgroundScrollInputs,
    /// All sampled arithmetic work through physical return.
    pub whole: Template,
}

impl State {
    pub(super) fn advance_background(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let contract = content.background.as_ref().ok_or_else(missing)?;
        let camera = self.s6_camera.following().ok_or_else(missing)?;
        match &self.background {
            suffix_caller::BackgroundScroll::Pending => {
                let profile = self
                    .background
                    .begin(camera, &self.camera, &contract.inputs, &contract.whole)
                    .map_err(|_| missing())?;
                self.timing
                    .begin(suffix_caller::BACKGROUND_SCROLL, profile)
                    .map_err(ProductionFault::FrameHardware)?;
            }
            suffix_caller::BackgroundScroll::Charging {
                camera: before,
                inputs,
                ..
            } => {
                if before != &self.camera
                    || inputs != &contract.inputs
                    || self
                        .timing
                        .work
                        .iter()
                        .filter(|work| {
                            work.kind
                                == timing::WorkKind::Semantic(suffix_caller::BACKGROUND_SCROLL)
                        })
                        .count()
                        != 1
                {
                    return Err(missing());
                }
            }
            suffix_caller::BackgroundScroll::Complete(_) => return Err(missing()),
        }
        Ok(())
    }

    pub(super) fn sample_background(
        &mut self,
        events: &[timing::Event],
    ) -> Result<(), ProductionFault> {
        for event in events {
            if let timing::EventKind::Sample(sample, timing::SampleValue::Byte(byte)) = event.kind {
                self.background
                    .sample(suffix_caller::BACKGROUND_SCROLL, sample, byte)
                    .map_err(|_| missing())?;
            }
        }
        Ok(())
    }

    pub(super) fn complete_background(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let contract = content.background.as_ref().ok_or_else(missing)?;
        self.background
            .complete(
                suffix_caller::BACKGROUND_SCROLL,
                &self.camera,
                &contract.inputs,
            )
            .map_err(|_| missing())?;
        Ok(())
    }
}

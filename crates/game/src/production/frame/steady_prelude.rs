//! Qualified quiet Frame continuation through the live Steady Haze and `LayerBlend` owners.

use super::{
    TransferContent,
    caller::{Caller, HAZE, HAZE_TAIL, LAYERS},
    haze, haze_caller,
    layer_blend::{self, LayerBlendDefaults},
};
use crate::production::{ProductionFault, timing};
use canonical::Canonical;
use data::{
    dma::BusAddress,
    timing::{MasterTicks, Template},
};
use world_haze::HazeState;

/// Current independently imported state and checked work from the same Frame origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qualification {
    /// Clean current Steady model input.
    pub model: HazeState,
    /// Current software countdowns for slots zero and five.
    pub countdowns: [u16; 2],
    /// Original band bytes read from the retained transfer backing.
    pub backing: [u8; 16],
    /// Checked address of the owned retained buffer.
    pub bands_address: BusAddress,
    /// Physical progress of Channels Three and Eight at the same origin.
    pub physical_channels: [timing::Channel; 2],
    /// Software scheduler mask before finishing this opportunity.
    pub software_mask: u8,
    /// Active second window edges and logic.
    pub window_two: [u8; 4],
    /// Actual immutable layer settings.
    pub layer_defaults: LayerBlendDefaults,
    /// Quiet current Steady handler work.
    pub retain: Template,
    /// Quiet remaining scheduler work.
    pub finish: Template,
    /// Quiet current `LayerBlend` work.
    pub layers: Template,
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub(super) struct State {
    haze: haze::Haze,
    backing: TransferContent,
    software_mask: u8,
    window_two: [u8; 4],
}

/// Live quantities owned by the Steady continuation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    /// Retained band bytes available to the actual transfer consumer.
    pub backing: [u8; 16],
    /// Scheduler countdowns in object order.
    pub countdowns: [u16; 2],
    /// Software scheduling mask.
    pub software_mask: u8,
    /// Physical line transfer enables.
    pub physical_mask: u8,
    /// Active second window edges and logic.
    pub window_two: [u8; 4],
}

fn mask(timing: &timing::State) -> u8 {
    timing
        .hardware
        .transfers
        .channels
        .iter()
        .enumerate()
        .fold(0, |value, (index, channel)| {
            value | (u8::from(channel.line.enabled) << index)
        })
}

impl Qualification {
    fn validate_channels(&self, timing: &timing::State) -> Result<(), ProductionFault> {
        let actual =
            haze::CHANNELS.map(|slot| timing.hardware.transfers.channels.get(slot.ordinal()));
        if mask(timing) != haze::active_slot_mask()
            || !actual
                .into_iter()
                .eq(self.physical_channels.iter().map(Some))
        {
            return Err(haze::missing());
        }
        Ok(())
    }

    pub(super) fn initial(&self, timing: &timing::State) -> Result<State, ProductionFault> {
        self.validate_channels(timing)?;
        for template in [&self.retain, &self.finish, &self.layers] {
            haze::quiet_profile(template)?;
        }
        let haze = haze::Haze::steady(self.model, self.countdowns)?;
        let bands = haze::materialization(&haze::bands(self.model), self.bands_address)?;
        if bands.region.bytes != self.backing {
            return Err(haze::missing());
        }
        let backing = TransferContent::from_retained(bands).map_err(|_| haze::missing())?;
        Ok(State {
            haze,
            backing,
            software_mask: self.software_mask,
            window_two: self.window_two,
        })
    }

    pub(super) const fn validation_phase(
        &self,
        caller: &Caller,
    ) -> Option<(&Template, timing::WorkId)> {
        if matches!(caller, Caller::HazeRunning { .. }) {
            Some((&self.retain, HAZE))
        } else if matches!(caller, Caller::HazeTail { .. }) {
            Some((&self.finish, HAZE_TAIL))
        } else if matches!(caller, Caller::LayerBlendRunning { .. }) {
            Some((&self.layers, LAYERS))
        } else {
            None
        }
    }

    pub(super) fn finished<'a>(&'a self, caller: &Caller) -> impl Iterator<Item = &'a Template> {
        let count = if matches!(caller, Caller::Random { .. }) {
            3
        } else if matches!(
            caller,
            Caller::LayerBlend { .. } | Caller::LayerBlendRunning { .. }
        ) {
            2
        } else {
            usize::from(matches!(caller, Caller::HazeTail { .. }))
        };
        [&self.retain, &self.finish, &self.layers]
            .into_iter()
            .take(count)
    }
}

impl State {
    pub(super) fn observation(&self, timing: &timing::State) -> Option<Observation> {
        let region = self.backing.regions().into_iter().next()?;
        Some(Observation {
            backing: region.bytes.try_into().ok()?,
            countdowns: self.haze.countdowns(),
            software_mask: self.software_mask,
            physical_mask: mask(timing),
            window_two: self.window_two,
        })
    }

    pub(super) fn validate(
        &self,
        qualification: &Qualification,
        caller: &Caller,
        timing: &timing::State,
    ) -> Result<(), ProductionFault> {
        let mut initial = qualification.initial(timing)?;
        let (work, _) = initial
            .haze
            .choose(None, None, Some(&qualification.retain))?;
        if let Caller::HazeRunning { work: held, .. } = caller
            && *held != work
        {
            return Err(haze::missing());
        }
        if let Caller::HazeTail { prepared, .. }
        | Caller::LayerBlend { haze: prepared, .. }
        | Caller::LayerBlendRunning { haze: prepared, .. }
        | Caller::Random { haze: prepared, .. } = caller
        {
            let result = initial.haze.complete_head(
                &work,
                &mut initial.backing,
                qualification.bands_address,
            )?;
            if *prepared != result {
                return Err(haze::missing());
            }
        }
        if matches!(
            caller,
            Caller::LayerBlend { .. } | Caller::LayerBlendRunning { .. } | Caller::Random { .. }
        ) {
            initial.software_mask = initial.haze.complete_tail()?;
        }
        if let Caller::LayerBlendRunning { haze, layers, .. }
        | Caller::Random { haze, layers, .. } = caller
        {
            if *layers != qualification.layer_defaults.prepare(haze.blend) {
                return Err(haze::missing());
            }
            layer_blend::encode(layers.terminal).map_err(|_| haze::missing())?;
        }
        if *self != initial {
            return Err(haze::missing());
        }
        Ok(())
    }

    pub(super) fn advance(
        &mut self,
        qualification: &Qualification,
        caller: &mut Caller,
        timing: &mut timing::State,
        budget: MasterTicks,
    ) -> Result<(timing::Progress, Vec<timing::Event>), ProductionFault> {
        match caller {
            Caller::Haze { .. } => haze_caller::begin_head(
                caller,
                timing,
                &self.haze,
                None,
                None,
                Some(&qualification.retain),
            )?,
            Caller::LayerBlend { .. } => haze_caller::begin_layers(
                caller,
                timing,
                qualification.layer_defaults,
                &qualification.layers,
            )?,
            Caller::Random { .. } => {
                return Err(ProductionFault::Unsupported(
                    crate::production::Unsupported::Frame(
                        super::caller::Dependency::RandomPreparation,
                    ),
                ));
            }
            Caller::HazeRunning { .. }
            | Caller::HazeTail { .. }
            | Caller::LayerBlendRunning { .. } => {}
            Caller::AwaitingAcknowledgment { .. }
            | Caller::Acknowledgment { .. }
            | Caller::Music { .. }
            | Caller::MusicRunning { .. }
            | Caller::RandomRunning { .. }
            | Caller::Preparation { .. }
            | Caller::Player { .. }
            | Caller::PlayerContinuation { .. } => {
                return Err(ProductionFault::WrongScope);
            }
        }
        let (progress, events) = timing
            .step(budget, &self.backing.regions())
            .map_err(ProductionFault::FrameHardware)?;
        match progress {
            timing::Progress::Completed(HAZE) => haze_caller::complete_head(
                caller,
                timing,
                &mut self.haze,
                &mut self.backing,
                qualification.bands_address,
                &qualification.finish,
            )?,
            timing::Progress::Completed(HAZE_TAIL) => {
                self.software_mask = haze_caller::complete_finish(caller, &mut self.haze)?;
            }
            timing::Progress::Completed(LAYERS) => haze_caller::complete_layers(caller)?,
            timing::Progress::Advanced => {}
            timing::Progress::Completed(_)
            | timing::Progress::Interrupt(_)
            | timing::Progress::HandlerEntered(_)
            | timing::Progress::HandlerReturned(_)
            | timing::Progress::HandlerPortionCompleted(_)
            | timing::Progress::HandlerNeedsWork(_)
            | timing::Progress::Idle => {
                return Err(ProductionFault::FrameHardware(
                    timing::Fault::InterruptBoundary,
                ));
            }
        }
        if !events.is_empty() {
            return Err(haze::missing());
        }
        qualification.validate_channels(timing)?;
        Ok((progress, events))
    }
}

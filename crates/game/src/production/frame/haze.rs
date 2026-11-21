//! Bind the live haze decision to its backdrop buffer and physical transport.

use super::{
    Buffer, Dependency, Materialization, TransferContent,
    borrowed::{HazeOrigin, HazeSlot},
    borrowed_prelude_receipts::{
        BackdropBand, BandIntensity, LayerBlendDisposition, PreparedFixedColour, encode_bands,
    },
};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::{
    dma::BusAddress,
    timing::{ChannelSlot, ContentRegion, Profile, Template, TemplateEffect},
};
use world_haze::{BandIntent, BehaviourId, HazeColour, HazeState, ThirdBackground};

pub(super) struct HazePin;

impl Pinned for HazePin {
    type State = HazeState;
    const NAME: &'static str = "world-haze";
    const SHAPE: u64 = 0x8985_e1ba_46d4_f7c4;
}

/// Current portable haze state and the actual active transport's two countdowns.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Haze {
    model: Model<HazePin>,
    countdowns: [u16; 2],
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum BandPublication {
    Prepare,
    Retain,
}

/// Selected model result, held privately while the physical head charges.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Work {
    before: Model<HazePin>,
    result: Model<HazePin>,
    publication: BandPublication,
    fixed_colour: PreparedFixedColour,
    blend: LayerBlendDisposition,
}

/// Prepared display contribution after the completed physical head.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Prepared {
    pub(super) bands: [BackdropBand; 16],
    pub(super) fixed_colour: PreparedFixedColour,
    pub(super) blend: LayerBlendDisposition,
}

pub(super) const CHANNELS: [ChannelSlot; 2] = [ChannelSlot::Three, ChannelSlot::Eight];

pub(super) const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::HazePreparation))
}

pub(super) fn quiet_profile(template: &Template) -> Result<Profile, ProductionFault> {
    if template
        .runs
        .iter()
        .any(|run| run.effect != TemplateEffect::None)
    {
        return Err(missing());
    }
    template
        .bind(&[])
        .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))
}

impl Haze {
    /// Retain the current model and the qualified active transport from the same origin.
    ///
    /// # Errors
    /// Unsupported Haze or line-transfer ownership is refused at admission.
    pub fn from_origin(model: HazeState, origin: &HazeOrigin) -> Result<Self, ProductionFault> {
        let mut active = origin.slots.iter().filter(|slot| **slot != HazeSlot::Empty);
        let (
            Some(HazeSlot::BackdropRamp {
                channel: ChannelSlot::Three,
                countdown: ramp,
                ..
            }),
            Some(HazeSlot::KeepTransfer {
                channel: ChannelSlot::Eight,
                countdown: hold,
            }),
            None,
        ) = (active.next(), active.next(), active.next())
        else {
            return Err(missing());
        };
        if !origin.running || !origin.power_bomb_inactive || !origin.enabled {
            return Err(missing());
        }
        Ok(Self {
            model: Model(model),
            countdowns: [*ramp, *hold],
        })
    }

    /// Read the current committed haze quantities.
    #[must_use]
    pub const fn current(&self) -> HazeState {
        self.model.0
    }

    pub(super) fn choose(
        &self,
        brightening: &Template,
        settling: Option<&Template>,
    ) -> Result<(Work, Profile), ProductionFault> {
        if self.countdowns.iter().any(|countdown| *countdown <= 1) {
            return Err(missing());
        }
        let step = world_haze::ordinary_opportunity(self.model.0);
        let template = match step.behaviour {
            BehaviourId::HazeBrightens => brightening,
            BehaviourId::HazeSettles => settling.ok_or_else(missing)?,
            BehaviourId::HazeHolds => return Err(missing()),
        };
        let profile = quiet_profile(template)?;
        let publication = match step.display.bands {
            BandIntent::Prepare => BandPublication::Prepare,
            BandIntent::Retain => BandPublication::Retain,
        };
        let blend = match step.display.third_background {
            ThirdBackground::Excluded => LayerBlendDisposition::NoBackgroundThree,
        };
        if step.display.backdrop != types::Colour::BLACK {
            return Err(missing());
        }
        Ok((
            Work {
                before: self.model,
                result: Model(step.state),
                publication,
                fixed_colour: PreparedFixedColour::Black,
                blend,
            },
            profile,
        ))
    }

    pub(super) fn complete_head(
        &mut self,
        work: &Work,
        backing: &mut TransferContent,
        address: BusAddress,
    ) -> Result<Prepared, ProductionFault> {
        if self.model != work.before {
            return Err(missing());
        }
        let component = match work.result.0.colour {
            HazeColour::Red => super::borrowed::ColourComponent::Red,
            HazeColour::Green => super::borrowed::ColourComponent::Green,
            HazeColour::Blue => super::borrowed::ColourComponent::Blue,
        };
        let bands = work.result.0.bands.0.map(|level| BackdropBand {
            component,
            intensity: BandIntensity(level.get()),
        });
        let bytes = encode_bands(&bands).map_err(|_| missing())?;
        let materialization = Materialization {
            buffer: Buffer::HazeBackdropBands,
            region: ContentRegion {
                start: address,
                bytes: bytes.to_vec(),
            },
        };
        match work.publication {
            BandPublication::Prepare => backing.complete(materialization),
            BandPublication::Retain => backing.retain(&materialization),
        }
        .map_err(|_| missing())?;
        self.model = work.result;
        Ok(Prepared {
            bands,
            fixed_colour: work.fixed_colour,
            blend: work.blend,
        })
    }

    pub(super) const fn complete_tail(&mut self) -> Result<(), ProductionFault> {
        let [ramp, hold] = self.countdowns;
        if ramp <= 1 || hold <= 1 {
            return Err(missing());
        }
        self.countdowns = [ramp.wrapping_sub(1), hold.wrapping_sub(1)];
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data::timing::{Cycle, ExposureKind, TemplateRun};
    use world_haze::{HazeBands, HazeFade, HazeLevel, HazePeak};

    fn template(count: u32) -> Template {
        Template {
            runs: vec![TemplateRun {
                count,
                cycle: Cycle::Six,
                kind: ExposureKind::Internal,
                poll_before: false,
                effect: TemplateEffect::None,
            }],
        }
    }

    #[test]
    fn equal_and_changed_bands_publish_before_settling_reuses_backing_after_restore() {
        assert_eq!(canonical::model::pin_holds::<HazePin>(), Ok(()));
        let bands = HazeBands(std::array::from_fn(|level| {
            HazeLevel::new(u8::try_from(level).unwrap()).unwrap()
        }));
        let zeros = HazeBands([HazeLevel::new(0).unwrap(); 16]);
        let bright = template(3);
        let settle = template(2);
        let address = BusAddress { bank: 1, offset: 1 };
        for current in [bands, zeros] {
            let mut owner = Haze {
                model: Model(HazeState {
                    colour: HazeColour::Green,
                    fade: HazeFade::Brightening(HazePeak::new(15).unwrap()),
                    bands: current,
                }),
                countdowns: [7, 8],
            };
            let mut backing = TransferContent::new(Vec::new()).unwrap();
            let (work, physical) = owner.choose(&bright, Some(&settle)).unwrap();
            assert_eq!(physical, bright.bind(&[]).unwrap());
            owner.complete_head(&work, &mut backing, address).unwrap();
            assert_eq!(owner.current().bands, bands);
            assert!(backing.ready_buffer(Buffer::HazeBackdropBands).is_some());
            owner.complete_tail().unwrap();

            backing.begin_opportunity();
            let regions = backing.regions();
            let mut resumed: Haze = canonical::from_bytes(&canonical::to_bytes(&owner)).unwrap();
            let mut restored: TransferContent =
                canonical::from_bytes(&canonical::to_bytes(&backing)).unwrap();
            let (work, physical) = resumed.choose(&bright, Some(&settle)).unwrap();
            assert_eq!(physical, settle.bind(&[]).unwrap());
            assert_eq!(
                resumed.complete_head(
                    &work,
                    &mut TransferContent::new(Vec::new()).unwrap(),
                    address
                ),
                Err(missing())
            );
            assert_eq!(resumed, owner);
            let live_work = owner.choose(&bright, Some(&settle)).unwrap().0;
            assert_eq!(
                owner.complete_head(&live_work, &mut backing, address),
                resumed.complete_head(&work, &mut restored, address)
            );
            owner.complete_tail().unwrap();
            resumed.complete_tail().unwrap();
            assert_eq!(resumed, owner);
            assert_eq!(resumed.current().fade, HazeFade::Steady);
            assert_eq!(restored, backing);
            assert_eq!(restored.regions(), regions);
        }
    }
}

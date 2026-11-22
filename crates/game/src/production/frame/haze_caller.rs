//! Shared Frame transitions for Haze and `LayerBlend`.

use super::caller::{Caller, Dependency, HAZE, HAZE_TAIL, LAYERS};
use super::{TransferContent, haze};
use crate::production::{ProductionFault, Unsupported, timing};
use data::{dma::BusAddress, timing::Template};

pub(super) fn begin_head(
    caller: &mut Caller,
    timing: &mut timing::State,
    haze: &haze::Haze,
    brightening: Option<&Template>,
    settling: Option<&Template>,
    steady: Option<&Template>,
) -> Result<(), ProductionFault> {
    let Caller::Haze {
        prelude,
        publication,
        music,
    } = &*caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    let (work, profile) = haze.choose(brightening, settling, steady)?;
    timing
        .begin(HAZE, profile)
        .map_err(ProductionFault::FrameHardware)?;
    *caller = Caller::HazeRunning {
        prelude: prelude.clone(),
        publication: *publication,
        music: music.clone(),
        work,
    };
    Ok(())
}

pub(super) fn complete_head(
    caller: &mut Caller,
    timing: &mut timing::State,
    haze: &mut haze::Haze,
    backing: &mut TransferContent,
    address: BusAddress,
    finish: &Template,
) -> Result<(), ProductionFault> {
    let Caller::HazeRunning {
        prelude,
        publication,
        music,
        work,
    } = &*caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    let prepared = haze.complete_head(work, backing, address)?;
    let profile = haze::quiet_profile(finish)?;
    timing
        .begin(HAZE_TAIL, profile)
        .map_err(ProductionFault::FrameHardware)?;
    *caller = Caller::HazeTail {
        prelude: prelude.clone(),
        publication: *publication,
        music: music.clone(),
        prepared,
    };
    Ok(())
}

pub(super) fn complete_finish(
    caller: &mut Caller,
    haze: &mut haze::Haze,
) -> Result<u8, ProductionFault> {
    let Caller::HazeTail {
        prelude,
        publication,
        music,
        prepared,
    } = &*caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    let mask = haze.complete_tail()?;
    *caller = Caller::LayerBlend {
        prelude: prelude.clone(),
        publication: *publication,
        music: music.clone(),
        haze: prepared.clone(),
    };
    Ok(mask)
}

pub(super) fn complete_layers(caller: &mut Caller) -> Result<(), ProductionFault> {
    let Caller::LayerBlendRunning {
        prelude,
        publication,
        music,
        haze,
        layers,
    } = &*caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    *caller = Caller::Random {
        prelude: prelude.clone(),
        publication: *publication,
        music: music.clone(),
        haze: haze.clone(),
        layers: layers.clone(),
    };
    Ok(())
}

pub(super) fn begin_layers(
    caller: &mut Caller,
    timing: &mut timing::State,
    defaults: super::layer_blend::LayerBlendDefaults,
    template: &Template,
) -> Result<(), ProductionFault> {
    let Caller::LayerBlend {
        prelude,
        publication,
        music,
        haze,
    } = &*caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    let layers = defaults.prepare(haze.blend);
    super::layer_blend::encode(layers.terminal).map_err(|_| {
        ProductionFault::Unsupported(Unsupported::Frame(Dependency::LayerBlendPreparation))
    })?;
    let profile = template
        .bind(&[])
        .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
    timing
        .begin(LAYERS, profile)
        .map_err(ProductionFault::FrameHardware)?;
    *caller = Caller::LayerBlendRunning {
        prelude: prelude.clone(),
        publication: *publication,
        music: music.clone(),
        haze: haze.clone(),
        layers,
    };
    Ok(())
}

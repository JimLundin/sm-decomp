//! Publish the object strips selected by the ordinary Ridley fight.

use super::Fault;
use crate::{exact::ridley_ribs, state::video::NmiBatch};
use boss_ceres_ridley::{RibLook, RidleyRun};
use data::assets::ridley_ribs::Art;

pub(super) fn queue(
    batch: &mut NmiBatch,
    ridley: Option<&RidleyRun>,
    art: Option<&Art>,
) -> Result<(), Fault> {
    let Some(reveal) = ridley.and_then(|run| run.rib_reveal) else {
        return Ok(());
    };
    let art = art.ok_or(Fault::RidleyRibs)?;
    let strips = match reveal {
        RibLook::Closed => &art.closed,
        RibLook::PartlyOpen => &art.partly_open,
        RibLook::Wide => &art.wide,
    };
    ridley_ribs::queue(batch, strips);
    Ok(())
}

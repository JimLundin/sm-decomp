//! Spec: `Dissolve`. The screen breaks into growing blocks as it darkens, then into shrinking
//! blocks as it brightens into the cursor's screen.

use types::options::{
    BlockSize, DissolveCursor, Dissolving, OptionsItem, OptionsStage, SpecialItem, WayOut,
};
use types::power_on::ScreenLight;

use crate::fade::{self, Fade};

/// Spec: `Dissolve`. A dissolve towards the cursor's item starts at block size 0; the screen is
/// left as it was until the next finished frame.
pub const fn start(cursor: DissolveCursor) -> OptionsStage {
    OptionsStage::DissolvingOut(Dissolving {
        cursor,
        blocks: BlockSize::SMALLEST,
    })
}

/// Spec: `Dissolve`. One level darker and the blocks one size larger; at dark it turns to
/// brighten.
pub fn darken(screen: &mut ScreenLight, dissolving: Dissolving) -> OptionsStage {
    let grown = Dissolving {
        blocks: dissolving.blocks.grown(),
        ..dissolving
    };
    match fade::darken(screen) {
        Fade::Partway => OptionsStage::DissolvingOut(grown),
        Fade::Ended => OptionsStage::DissolvingIn(grown),
    }
}

/// Spec: `Dissolve`. One level brighter and the blocks one size smaller; at full the blocks are
/// gone and the cursor's screen begins.
pub fn brighten(screen: &mut ScreenLight, dissolving: Dissolving) -> OptionsStage {
    match fade::brighten(screen) {
        Fade::Partway => OptionsStage::DissolvingIn(Dissolving {
            blocks: dissolving.blocks.shrunk(),
            ..dissolving
        }),
        Fade::Ended => arrival(dissolving.cursor),
    }
}

/// Spec: `Dissolve`. The screen a dissolve ends on, by the cursor's item: the options on start
/// game, the special settings on weapon-reset, or the controller settings, a later piece.
const fn arrival(cursor: DissolveCursor) -> OptionsStage {
    match cursor {
        DissolveCursor::StartGame => OptionsStage::Choosing(OptionsItem::StartGame),
        DissolveCursor::SpecialSettings => OptionsStage::Special(SpecialItem::WeaponReset),
        DissolveCursor::ControllerSettings => OptionsStage::HandedOn(WayOut::ControlsBegin),
    }
}

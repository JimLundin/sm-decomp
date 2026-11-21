//! Current equipment and room surfaces read by ordinary player entry.

use super::borrowed_gameplay::GameplayDispatchReceipt;
use super::borrowed_player_preparation::{
    EnvironmentCompletion, EnvironmentOrigin, LiquidSurface, PlayerPreparationFault,
    PreparedEnvironment, SuitPaletteCompletion, SuitPaletteDisposition, SuitPaletteOrigin,
    gameplay, ordinary_equipment,
};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use types::{Fixed, Item, Pose, Set};

pub(super) struct EquipmentPin;

impl Pinned for EquipmentPin {
    type State = Set<Item>;
    const NAME: &'static str = "production-frame-entry-equipment";
    const SHAPE: u64 = 0x178e_a93e_6a32_264e;
}

/// Equipped items and live water and lava/acid surface dispositions.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct EntryEnvironment {
    equipment: Model<EquipmentPin>,
    /// The room currently has no active water surface.
    pub water_inactive: bool,
    /// The room currently has no active lava or acid surface.
    pub lava_acid_inactive: bool,
}

impl EntryEnvironment {
    /// Initialize from current independently decoded equipment and surface owners.
    #[must_use]
    pub fn new(origin: &EnvironmentOrigin) -> Self {
        Self {
            equipment: Model(origin.equipment),
            water_inactive: origin.water == LiquidSurface::Inactive,
            lava_acid_inactive: origin.lava_acid == LiquidSurface::Inactive,
        }
    }

    pub(super) fn supports_power_suit(&self) -> bool {
        ordinary_equipment(&self.equipment.0).is_ok()
    }

    pub(super) fn supports_dry_environment(&self) -> bool {
        self.supports_power_suit() && self.water_inactive && self.lava_acid_inactive
    }

    pub(super) fn suit(
        &self,
        route: &GameplayDispatchReceipt,
    ) -> Result<SuitPaletteCompletion, PlayerPreparationFault> {
        gameplay(route)?;
        ordinary_equipment(&self.equipment.0)?;
        Ok(SuitPaletteCompletion {
            dependency: SuitPaletteOrigin {
                equipment: self.equipment.0,
            },
            terminal: SuitPaletteDisposition::PowerSuit,
        })
    }

    pub(super) fn environment(
        &self,
        pose: Pose,
        centre_y: u16,
        acceleration: Fixed,
        route: &GameplayDispatchReceipt,
    ) -> Result<EnvironmentCompletion, PlayerPreparationFault> {
        gameplay(route)?;
        if !self.supports_dry_environment() {
            return Err(PlayerPreparationFault::Unsupported);
        }
        Ok(EnvironmentCompletion {
            dependency: EnvironmentOrigin {
                equipment: self.equipment.0,
                pose,
                centre_y,
                water: LiquidSurface::Inactive,
                lava_acid: LiquidSurface::Inactive,
            },
            terminal: PreparedEnvironment { acceleration },
        })
    }
}

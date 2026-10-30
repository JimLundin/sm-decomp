//! Magdollite runs in `sm_game::actors::norfair_lava_man`; its tables come from here.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, load};
use crate::game_state::Game;
use sm_data::magdollite::{
    ARM_BYTES, ARM_GROWTH_ADDRESS, ARM_INSTRUCTION_ADDRESS, ArmSteps, PALETTE_ADDRESS,
    PALETTE_BYTES, Palettes, import_palettes,
};
pub(crate) use sm_game::actors::norfair_lava_man::*;

const _: () = assert!(core::mem::size_of::<crate::types::EnemySpawnData>() == SPAWN_DATA_BYTES);

#[derive(Default)]
pub(crate) struct ImportedTables {
    palettes: Option<Imported<Palettes>>,
    arm_steps: Option<Imported<ArmSteps>>,
}

pub(crate) fn import(image: &crate::rom::RomImage) -> ImportedTables {
    ImportedTables {
        palettes: Imported::import(image, PALETTE_ADDRESS, PALETTE_BYTES, import_palettes),
        arm_steps: Imported::import(image, ARM_GROWTH_ADDRESS, ARM_BYTES, ArmSteps::import),
    }
}

impl Host for Game {
    /// Colours 9-12 of the glow palette whose colour 0 is word `source`.
    fn magdollite_glow_colors(&self, source: isize) -> [u16; 4] {
        Imported::read(
            &self.magdollite.palettes,
            self,
            |table| {
                let palette = table.row(crate::game::imported::row_of(source, 16)?)?;
                Some(core::array::from_fn(|i| palette[9 + i]))
            },
            |rom| core::array::from_fn(|i| load(rom, PALETTE_ADDRESS, source + 9 + i as isize)),
        )
    }

    /// Arm step `index`'s field; other images and indices run the original load
    /// at `address`.
    fn magdollite_arm(&self, address: u32, index: isize) -> u16 {
        Imported::read(
            &self.magdollite.arm_steps,
            self,
            |table| {
                let step = table.step(usize::try_from(index).ok()?)?;
                Some(match address {
                    ARM_GROWTH_ADDRESS => step.growth,
                    ARM_INSTRUCTION_ADDRESS => step.instruction_list,
                    _ => step.y_offset,
                })
            },
            |rom| load(rom, address, index),
        )
    }
}

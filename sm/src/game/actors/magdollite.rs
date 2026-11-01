//! Magdollite runs in `sm_game::actors::magdollite`; its tables come from here.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use sm_data::magdollite::{
    ARM_BYTES, ARM_GROWTH_ADDRESS, ARM_INSTRUCTION_ADDRESS, ArmSteps, PALETTE_ADDRESS,
    PALETTE_BYTES, Palettes, import_palettes,
};
pub(crate) use sm_game::actors::magdollite::*;
use sm_game::imported::{Imported, load};

// The named bank $A8 addresses are the ones `session_runtime` dispatches on.
const _: () = {
    use sm_game::actors::magdollite::{instruction as i, reaction as r, state as s};
    const fn at(word: u16) -> u32 {
        0xa8_0000 | word as u32
    }
    assert!(i::QUEUE_SOUND_IF_ON_SCREEN == 11054610 && i::HEAD_DOWN_2PX == 11054630);
    assert!(i::HEAD_UP_2PX == 11054640 && i::SET_ANIMATION_ACTIVE == 11054650);
    assert!(i::CLEAR_ANIMATION_ACTIVE == 11054661 && i::HEAD_AND_ARM_UP_1PX == 11054672);
    assert!(i::HEAD_AND_ARM_DOWN_1PX == 11054682 && i::SUBMERGED_POSITION_SHOW_ARM == 11054692);
    assert!(i::RESET_POSITION == 11054728 && i::EMERGED_POSITION_HIDE_ARM == 11054742);
    assert!(i::SPAWN_LAVA == 11054778 && i::OPEN_HAND_RIGHT == 11054794);
    assert!(i::OPEN_HAND_LEFT == 11054820 && i::CLOSED_HAND_RIGHT == 11054846);
    assert!(i::CLOSED_HAND_LEFT == 11054872 && i::RESET_COOLDOWN == 11054916);
    assert!(r::POWER_BOMB == 11056128 && r::CONTACT == 11056134 && r::SHOT == 11056140);
    assert!(at(s::HEAD_IDLE) == 11055386 && at(s::HEAD_WAIT_SUBMERGED) == 11055477);
    assert!(at(s::HEAD_EMERGE_WHEN_READY) == 11055507 && at(s::HEAD_WAIT_EMERGED) == 11055544);
    assert!(at(s::ARM_WAIT_HEAD_SUBMERGED) == 11055581 && at(s::ARM_GROWING) == 11055620);
    assert!(at(s::ARM_GROWN) == 11055761 && at(s::ARM_SHRINKING) == 11055765);
    assert!(at(s::HAND_IDLE) == 11055885 && at(s::HAND_THROW_AND_SHRINK) == 11055903);
    assert!(at(s::HAND_GROW_AND_THROW) == 11055958 && at(s::HAND_FOLLOW_SHRINKING_ARM) == 11056039);
};

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
                let palette = table.row(sm_game::imported::row_of(source, 16)?)?;
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

    fn warn_lava_height(&mut self, value: i32) {
        crate::game::session_runtime::warn_lava_height(value);
    }
}

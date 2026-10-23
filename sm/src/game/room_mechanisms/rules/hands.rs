//! Chozo hand contact guards, activation and nested mechanism creation.
#![forbid(unsafe_code)]
use super::super::creation::Source;
use super::super::{
    fields::{Fields, Global},
    state::Column,
};
use crate::game::mechanism_displacement::Displacement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    LowerNorfair,
    WreckedShip,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundedPose {
    MorphRight,
    SpringRight,
    SpringLeft,
}

pub fn setup(
    c: &mut impl Fields,
    slot: usize,
    hand: Hand,
    displacement: &mut Displacement<'_>,
) -> u8 {
    let enabled = match hand {
        Hand::LowerNorfair => c.global(Global::CollectedItems) & 0x200 != 0,
        Hand::WreckedShip => c.boss(1) & 1 != 0,
    };
    if enabled
        && c.global(Global::CollisionDirection) & 15 == 3
        && (pose_matches(c, GroundedPose::MorphRight)
            || pose_matches(c, GroundedPose::SpringRight)
            || pose_matches(c, GroundedPose::SpringLeft))
    {
        if hand == Hand::LowerNorfair {
            c.mark_event(0xc);
        }
        c.statue_signal();
        if hand == Hand::WreckedShip {
            c.native_scroll_pair(7, 0x202);
            c.native_scroll_pair(13, 0x101);
        }
        let block = c.slot(Column::Block, slot);
        let level = c.level(block);
        c.set_level(block, level & 0xfff);
        c.player_control(0);
        c.spawn(
            Source::Hardcoded {
                x: if hand == Hand::LowerNorfair { 12 } else { 23 },
                y: 29,
                header: if hand == Hand::LowerNorfair {
                    0xd113
                } else {
                    0xd6f8
                },
            },
            displacement,
        );
    }
    // The child or another callback can reuse this captured slot.
    c.release(slot);
    1
}

pub fn prepare_ship_trigger(c: &mut impl Fields, slot: usize) -> u8 {
    c.install(c.slot(Column::Block, slot), 0xb080);
    0
}
pub fn monitor_norfair(c: &mut impl Fields) {
    let block = (8 * u16::from(c.global(Global::RoomWidth) as u8) + 4).wrapping_mul(2);
    if c.level(block) == 0xff {
        c.install(block, 0xb083);
        // The running PLM selection is read after installing the trigger.
        c.release(usize::from(c.global(Global::Selected) >> 1));
    }
}
pub fn lower_acid(c: &mut impl Fields) {
    c.set_global(Global::LiquidHeight, 722);
}

fn pose_matches(c: &impl Fields, pose: GroundedPose) -> bool {
    c.global(Global::Pose)
        == match pose {
            GroundedPose::MorphRight => 0x1d,
            GroundedPose::SpringRight => 0x79,
            GroundedPose::SpringLeft => 0x7a,
        }
}

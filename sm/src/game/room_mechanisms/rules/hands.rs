//! Chozo hand contact guards, activation and nested mechanism creation.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::creation;
use super::super::host::Contact as Displacement;
use super::super::host::SpawnSource as Source;
use super::super::host::*;
use super::terrain_setup as terrain;
use crate::game::progress_flags as flags;

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
    c: &mut impl Host,
    slot: usize,
    hand: Hand,
    displacement: &mut Displacement<'_>,
) -> u8 {
    let enabled = match hand {
        Hand::LowerNorfair => c.session().inventory.equipment.owned & 0x200 != 0,
        Hand::WreckedShip => flags::boss_defeated(c, 1),
    };
    if enabled
        && c.session().player.contact.collision_direction & 15 == 3
        && (pose_matches(c, GroundedPose::MorphRight)
            || pose_matches(c, GroundedPose::SpringRight)
            || pose_matches(c, GroundedPose::SpringLeft))
    {
        if hand == Hand::LowerNorfair {
            flags::set_event(c, 0xc);
        }
        c.statue_signal();
        if hand == Hand::WreckedShip {
            c.scroll_pair(7, 0x202);
            c.scroll_pair(13, 0x101);
        }
        let block = mechanism!(c, slot, block);
        let level = terrain::level(c, block);
        terrain::set_level(c, block, level & 0xfff);
        c.player_control(0);
        creation::spawn(
            c,
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
    mechanism!(c, slot, header = 0);
    1
}

pub fn prepare_ship_trigger(c: &mut impl Host, slot: usize) -> u8 {
    let block = mechanism!(c, slot, block);
    terrain::install(c, block, 0xb080);
    0
}
pub fn monitor_norfair(c: &mut impl Host) {
    let block = (8 * u16::from(c.session().room.width as u8) + 4).wrapping_mul(2);
    if terrain::level(c, block) == 0xff {
        terrain::install(c, block, 0xb083);
        // The running PLM selection is read after installing the trigger.
        mechanism!(
            c,
            usize::from(c.session().mechanisms.selected >> 1),
            header = 0
        );
    }
}
pub fn lower_acid(c: &mut impl Host) {
    c.session_mut().room.liquid_height = 722;
}

fn pose_matches(c: &impl Host, pose: GroundedPose) -> bool {
    c.session().player.pose
        == match pose {
            GroundedPose::MorphRight => 0x1d,
            GroundedPose::SpringRight => 0x79,
            GroundedPose::SpringLeft => 0x7a,
        }
}

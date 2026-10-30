//! Breakable terrain setup, including reveal, retained blocks and grapple flags.
#![forbid(unsafe_code)]

use super::super::access::*;
use super::super::host::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    ScrewAttack,
    SpeedBoost,
    Crumble,
    Shot,
    BombContact,
    BombShot,
    PowerBombShot,
    SuperMissileShot,
    PowerBombTrigger,
    CrittersEscape,
    Grapple,
    Deactivate,
    EnemyBreakable,
    Blank,
    ShotBlock,
    GrappleBlock,
}
#[derive(Clone, Copy)]
pub enum BlockReaction {
    Shot,
    Grapple,
}
#[derive(Clone, Copy)]
pub enum ProjectileWord {
    Native,
    LittleEndian,
}
#[derive(Clone, Copy)]
pub enum RejectionProgram {
    PowerBomb,
    SuperMissile,
}

fn shot(c: &impl Host, order: ProjectileWord) -> u16 {
    match order {
        ProjectileWord::Native => projectile_kind(
            c,
            (c.session().projectiles.selected >> 1) as isize,
            ByteOrder::Native,
        ),
        ProjectileWord::LittleEndian => projectile_kind(
            c,
            (c.session().projectiles.selected >> 1) as isize,
            ByteOrder::Little,
        ),
    }
}
fn reveal(c: &mut impl Host, slot: usize, tile: Option<u16>, mask: u16) {
    let block = mechanism!(c, slot, block);
    let original = level(c, block);
    let saved = tile.map_or(original, |tile| original & 0xf000 | tile);
    mechanism!(c, slot, work0 = saved);
    set_level(c, block, saved & mask);
}
fn speed_breaks(c: &impl Host, screw_attack: bool) -> bool {
    if c.session().player.contact.speed_boost & 0xf00 == 0x400 {
        return true;
    }
    let pose = c.session().player.pose;
    (screw_attack && matches!(pose, 0x81 | 0x82)) || (0xc9..=0xce).contains(&pose)
}

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::ScrewAttack | Setup::SpeedBoost | Setup::BombContact => {
            let accepted = match kind {
                Setup::ScrewAttack => matches!(c.session().player.pose, 0x81 | 0x82),
                Setup::SpeedBoost => speed_breaks(c, false),
                _ => speed_breaks(c, true),
            };
            if !accepted {
                mechanism!(c, slot, header = 0);
                return 1;
            }
            let tile = match kind {
                Setup::SpeedBoost => Some(0xb6),
                Setup::BombContact => Some(0x58),
                _ => None,
            };
            reveal(c, slot, tile, 0xfff);
        }
        Setup::Crumble => {
            if c.session().player.contact.collision_direction & 15 == 3 {
                reveal(c, slot, Some(0xbc), 0x8fff);
                mechanism!(c, slot, instruction_delay = 4);
            } else {
                mechanism!(c, slot, header = 0);
            }
            return 1;
        }
        Setup::Shot => reveal(c, slot, Some(0x52), 0x8fff),
        Setup::BombShot => match shot(c, ProjectileWord::Native) & 0xf00 {
            0x500 => {
                let instruction = mechanism!(c, slot, program).wrapping_add(3);
                mechanism!(c, slot, program = instruction);
                reveal(c, slot, Some(0x58), 0x8fff);
            }
            0x300 => reveal(c, slot, Some(0x58), 0x8fff),
            _ => mechanism!(c, slot, header = 0),
        },
        Setup::PowerBombShot | Setup::SuperMissileShot => {
            let (order, accepted, tile, program) = if kind == Setup::PowerBombShot {
                (
                    ProjectileWord::LittleEndian,
                    0x300,
                    0x57,
                    RejectionProgram::PowerBomb,
                )
            } else {
                (
                    ProjectileWord::Native,
                    0x200,
                    0x9f,
                    RejectionProgram::SuperMissile,
                )
            };
            let projectile = shot(c, order) & 0xf00;
            if projectile == 0x500 {
                mechanism!(
                    c,
                    slot,
                    program = match program {
                        RejectionProgram::PowerBomb => 0xc91c,
                        RejectionProgram::SuperMissile => 0xc922,
                    }
                );
            } else if projectile == accepted {
                reveal(c, slot, Some(tile), 0x8fff);
            } else {
                mechanism!(c, slot, header = 0);
            }
        }
        Setup::PowerBombTrigger => {
            if shot(c, ProjectileWord::Native) & 0xf00 != 0x500 {
                mechanism!(c, slot, header = 0);
            }
        }
        Setup::CrittersEscape => {
            if shot(c, ProjectileWord::Native) != 0 {
                reveal(c, slot, Some(0x9f), 0x8fff);
            } else {
                mechanism!(c, slot, header = 0);
            }
        }
        Setup::Grapple => {
            let block = mechanism!(c, slot, block);
            let level = level(c, block);
            mechanism!(c, slot, work0 = level);
            let next = behavior_byte(c, usize::from((block) >> 1) + 1);
            set_behavior_pair(c, usize::from((block) >> 1), u16::from(next) << 8);
            return 0x41;
        }
        Setup::Blank => {
            let block = mechanism!(c, slot, block);
            set_level(c, block, 0);
        }
        Setup::ShotBlock | Setup::GrappleBlock => {
            let block = mechanism!(c, slot, block);
            let reaction = if kind == Setup::ShotBlock {
                BlockReaction::Shot
            } else {
                BlockReaction::Grapple
            };
            install(
                c,
                block,
                match reaction {
                    BlockReaction::Shot => 0xc000,
                    BlockReaction::Grapple => 0xe000,
                },
            );
        }
        Setup::Deactivate | Setup::EnemyBreakable => {
            let block = mechanism!(c, slot, block);
            let mask = if kind == Setup::Deactivate {
                0x8fff
            } else {
                0xfff
            };
            let level = level(c, block) & mask;
            set_level(c, block, level);
        }
    }
    0
}

/// A block selector rounds down for ordinary level-word accesses.
pub fn level(c: &impl Host, block: u16) -> u16 {
    terrain_word(c, usize::from(block & !1), ByteOrder::Little)
}
pub fn set_level(c: &mut impl Host, block: u16, value: u16) {
    set_terrain_word(c, usize::from(block & !1), value, ByteOrder::Little);
}
/// Preserve unrounded high-byte placement before the behavior update.
pub fn install(c: &mut impl Host, block: u16, value: u16) {
    let high = terrain_high(c, block) & 15 | (value >> 8) as u8;
    set_terrain_high(c, block, high);
    set_behavior_byte(c, usize::from(block >> 1), value as u8);
}
pub fn below(c: &impl Host, block: u16) -> u16 {
    block.wrapping_add(c.session().room.width.wrapping_mul(2))
}

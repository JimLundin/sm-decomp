//! Breakable terrain setup, including reveal, retained blocks and grapple flags.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};

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

fn shot(c: &impl Fields, order: ProjectileWord) -> u16 {
    match order {
        ProjectileWord::Native => {
            c.projectile_word((c.global(Global::ProjectileSelection) >> 1) as isize)
        }
        ProjectileWord::LittleEndian => {
            c.projectile_little_word((c.global(Global::ProjectileSelection) >> 1) as isize)
        }
    }
}
fn reveal(c: &mut impl Fields, slot: usize, tile: Option<u16>, mask: u16) {
    let block = c.slot(Column::Block, slot);
    let original = c.level(block);
    let saved = tile.map_or(original, |tile| original & 0xf000 | tile);
    c.set_slot(Column::Work0, slot, saved);
    c.set_level(block, saved & mask);
}
fn speed_breaks(c: &impl Fields, screw_attack: bool) -> bool {
    if c.global(Global::SpeedBoost) & 0xf00 == 0x400 {
        return true;
    }
    let pose = c.global(Global::Pose);
    (screw_attack && matches!(pose, 0x81 | 0x82)) || (0xc9..=0xce).contains(&pose)
}

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::ScrewAttack | Setup::SpeedBoost | Setup::BombContact => {
            let accepted = match kind {
                Setup::ScrewAttack => matches!(c.global(Global::Pose), 0x81 | 0x82),
                Setup::SpeedBoost => speed_breaks(c, false),
                _ => speed_breaks(c, true),
            };
            if !accepted {
                c.release(slot);
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
            if c.global(Global::CollisionDirection) & 15 == 3 {
                reveal(c, slot, Some(0xbc), 0x8fff);
                c.set_slot(Column::InstructionDelay, slot, 4);
            } else {
                c.release(slot);
            }
            return 1;
        }
        Setup::Shot => reveal(c, slot, Some(0x52), 0x8fff),
        Setup::BombShot => match shot(c, ProjectileWord::Native) & 0xf00 {
            0x500 => {
                let instruction = c.slot(Column::Program, slot).wrapping_add(3);
                c.set_slot(Column::Program, slot, instruction);
                reveal(c, slot, Some(0x58), 0x8fff);
            }
            0x300 => reveal(c, slot, Some(0x58), 0x8fff),
            _ => c.release(slot),
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
                c.set_slot(
                    Column::Program,
                    slot,
                    match program {
                        RejectionProgram::PowerBomb => 0xc91c,
                        RejectionProgram::SuperMissile => 0xc922,
                    },
                );
            } else if projectile == accepted {
                reveal(c, slot, Some(tile), 0x8fff);
            } else {
                c.release(slot);
            }
        }
        Setup::PowerBombTrigger => {
            if shot(c, ProjectileWord::Native) & 0xf00 != 0x500 {
                c.release(slot);
            }
        }
        Setup::CrittersEscape => {
            if shot(c, ProjectileWord::Native) != 0 {
                reveal(c, slot, Some(0x9f), 0x8fff);
            } else {
                c.release(slot);
            }
        }
        Setup::Grapple => {
            let block = c.slot(Column::Block, slot);
            let level = c.level(block);
            c.set_slot(Column::Work0, slot, level);
            let next = c.behavior(block, 1);
            c.set_behavior_word(block, u16::from(next) << 8);
            return 0x41;
        }
        Setup::Blank => c.set_level(c.slot(Column::Block, slot), 0),
        Setup::ShotBlock | Setup::GrappleBlock => {
            let block = c.slot(Column::Block, slot);
            let reaction = if kind == Setup::ShotBlock {
                BlockReaction::Shot
            } else {
                BlockReaction::Grapple
            };
            c.install(
                block,
                match reaction {
                    BlockReaction::Shot => 0xc000,
                    BlockReaction::Grapple => 0xe000,
                },
            );
        }
        Setup::Deactivate | Setup::EnemyBreakable => {
            let block = c.slot(Column::Block, slot);
            let mask = if kind == Setup::Deactivate {
                0x8fff
            } else {
                0xfff
            };
            let level = c.level(block) & mask;
            c.set_level(block, level);
        }
    }
    0
}

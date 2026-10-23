//! Breakable terrain setup, including reveal, retained blocks and grapple flags.
#![forbid(unsafe_code)]

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

pub trait Context {
    type Block: Copy;
    fn block(&self, slot: usize) -> Self::Block;
    fn release(&mut self, slot: usize);
    fn level(&self, block: Self::Block) -> u16;
    fn set_level(&mut self, block: Self::Block, value: u16);
    fn install(&mut self, block: Self::Block, reaction: BlockReaction);
    fn set_saved(&mut self, slot: usize, value: u16);
    fn instruction(&self, slot: usize) -> u16;
    fn set_instruction(&mut self, slot: usize, value: u16);
    fn reject_program(&mut self, slot: usize, program: RejectionProgram);
    fn set_delay(&mut self, slot: usize, value: u16);
    fn speed_boost(&self) -> u16;
    fn pose(&self) -> u16;
    fn direction(&self) -> u16;
    fn projectile_selection(&self) -> u16;
    fn projectile_word(&self, index: usize, order: ProjectileWord) -> u16;
    fn next_behavior(&self, block: Self::Block) -> u8;
    fn set_behavior_word(&mut self, block: Self::Block, value: u16);
}

fn shot(c: &impl Context, order: ProjectileWord) -> u16 {
    c.projectile_word(usize::from(c.projectile_selection() >> 1), order)
}
fn reveal(c: &mut impl Context, slot: usize, tile: Option<u16>, mask: u16) {
    let block = c.block(slot);
    let original = c.level(block);
    let saved = tile.map_or(original, |tile| original & 0xf000 | tile);
    c.set_saved(slot, saved);
    c.set_level(block, saved & mask);
}
fn speed_breaks(c: &impl Context, screw_attack: bool) -> bool {
    if c.speed_boost() & 0xf00 == 0x400 {
        return true;
    }
    let pose = c.pose();
    (screw_attack && matches!(pose, 0x81 | 0x82)) || (0xc9..=0xce).contains(&pose)
}

pub fn setup(c: &mut impl Context, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::ScrewAttack | Setup::SpeedBoost | Setup::BombContact => {
            let accepted = match kind {
                Setup::ScrewAttack => matches!(c.pose(), 0x81 | 0x82),
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
            if c.direction() & 15 == 3 {
                reveal(c, slot, Some(0xbc), 0x8fff);
                c.set_delay(slot, 4);
            } else {
                c.release(slot);
            }
            return 1;
        }
        Setup::Shot => reveal(c, slot, Some(0x52), 0x8fff),
        Setup::BombShot => match shot(c, ProjectileWord::Native) & 0xf00 {
            0x500 => {
                let instruction = c.instruction(slot).wrapping_add(3);
                c.set_instruction(slot, instruction);
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
                c.reject_program(slot, program);
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
            let block = c.block(slot);
            let level = c.level(block);
            c.set_saved(slot, level);
            let next = c.next_behavior(block);
            c.set_behavior_word(block, u16::from(next) << 8);
            return 0x41;
        }
        Setup::Blank => c.set_level(c.block(slot), 0),
        Setup::ShotBlock | Setup::GrappleBlock => {
            let block = c.block(slot);
            let reaction = if kind == Setup::ShotBlock {
                BlockReaction::Shot
            } else {
                BlockReaction::Grapple
            };
            c.install(block, reaction);
        }
        Setup::Deactivate | Setup::EnemyBreakable => {
            let block = c.block(slot);
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

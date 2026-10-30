//! Tube rupture: terrain installation, input suspension and ordered effects.
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Setup {
    Air,
    Solid,
    FourBlocks,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Block {
    Air,
    Solid,
    RightExtension,
    BelowExtension,
    BelowRightExtension,
}

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup) -> u8 {
    install(
        slot,
        if kind == Setup::Air {
            Block::Air
        } else {
            Block::Solid
        },
        c,
    );
    if kind == Setup::FourBlocks {
        for block in [
            Block::RightExtension,
            Block::BelowExtension,
            Block::BelowRightExtension,
        ] {
            install(slot, block, c);
        }
    }
    0
}

pub fn poll(c: &mut impl Host, slot: usize) {
    if c.inputs().pressed & 0xc3c0 != 0 {
        mechanism!(c, slot, instruction_delay = 1);
        {
            let index = slot;
            let value = mechanism!(c, index, program).wrapping_add(1);
            mechanism!(c, index, program = value);
        };
        {
            let index = slot;
            let value = mechanism!(c, index, program).wrapping_add(1);
            mechanism!(c, index, program = value);
        };
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    Water,
    Crack,
    Quake,
    Shatter,
    Lock,
    Unlock,
}

pub fn command(c: &mut impl Host, kind: Command) {
    match kind {
        Command::Water => {
            c.session_mut().room.liquid_options = c.session().room.liquid_options & !4
        }
        Command::Crack => c.enemy_projectile(0xd904, 0),
        Command::Quake => {
            c.session_mut().feedback.earthquake_kind = 11;
            c.session_mut().feedback.earthquake_timer = 64;
        }
        Command::Shatter => {
            for shard in 0u8..10 {
                c.enemy_projectile(0xd912, u16::from(shard) * 2);
            }
            for bubble in 0u8..6 {
                c.enemy_projectile(0xd920, u16::from(bubble) * 2);
            }
        }
        Command::Lock => c.player_control(0),
        Command::Unlock => c.player_control(1),
    }
}

fn install(slot: usize, block: Block, c: &mut impl Host) {
    let index = match block {
        Block::Air | Block::Solid => mechanism!(c, slot, block),
        Block::RightExtension => mechanism!(c, slot, block).wrapping_add(2),
        Block::BelowExtension | Block::BelowRightExtension => {
            let width = c.session().room.width;
            let second = c.session().room.width;
            width
                .wrapping_add(second)
                .wrapping_add(mechanism!(c, slot, block))
                .wrapping_add(if block == Block::BelowRightExtension {
                    2
                } else {
                    0
                })
        }
    };
    terrain::install(
        c,
        index,
        match block {
            Block::Air => 0x44,
            Block::Solid => 0x8044,
            Block::RightExtension => 0x50ff,
            Block::BelowExtension | Block::BelowRightExtension => 0xd0ff,
        },
    );
}

//! Tube rupture: terrain installation, input suspension and ordered effects.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};

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

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup) -> u8 {
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

pub fn poll(c: &mut impl Fields, slot: usize) {
    if c.global(Global::NewKeys) & 0xc3c0 != 0 {
        c.set_slot(Column::InstructionDelay, slot, 1);
        c.advance(slot, 1);
        c.advance(slot, 1);
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

pub fn command(c: &mut impl Fields, kind: Command) {
    match kind {
        Command::Water => c.set_global(Global::LiquidOptions, c.global(Global::LiquidOptions) & !4),
        Command::Crack => c.eproj(0xd904, 0),
        Command::Quake => {
            c.set_global(Global::QuakeType, 11);
            c.set_global(Global::QuakeTimer, 64);
        }
        Command::Shatter => {
            for shard in 0u8..10 {
                c.eproj(0xd912, u16::from(shard) * 2);
            }
            for bubble in 0u8..6 {
                c.eproj(0xd920, u16::from(bubble) * 2);
            }
        }
        Command::Lock => c.player_control(0),
        Command::Unlock => c.player_control(1),
    }
}

fn install(slot: usize, block: Block, c: &mut impl Fields) {
    let index = match block {
        Block::Air | Block::Solid => c.slot(Column::Block, slot),
        Block::RightExtension => c.slot(Column::Block, slot).wrapping_add(2),
        Block::BelowExtension | Block::BelowRightExtension => {
            let width = c.global(Global::RoomWidth);
            let second = c.global(Global::RoomWidth);
            width
                .wrapping_add(second)
                .wrapping_add(c.slot(Column::Block, slot))
                .wrapping_add(if block == Block::BelowRightExtension {
                    2
                } else {
                    0
                })
        }
    };
    c.install(
        index,
        match block {
            Block::Air => 0x44,
            Block::Solid => 0x8044,
            Block::RightExtension => 0x50ff,
            Block::BelowExtension | Block::BelowRightExtension => 0xd0ff,
        },
    );
}

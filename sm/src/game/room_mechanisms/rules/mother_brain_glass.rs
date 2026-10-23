//! Mother Brain glass hit counting, threshold progression and ordered shard bursts.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, SoundChannel},
    state::Column,
};
use crate::game::enemy_script::EnemyScript;

pub fn setup(c: &mut impl Fields, slot: usize) -> u8 {
    c.set_slot(Column::RoomArgument, slot, 0);
    let block = c.slot(Column::Block, slot);
    c.install(block, 0x8044);
    0
}
pub fn add_hits(c: &mut impl Fields, slot: usize, amount: u16) {
    let hits = c.slot(Column::RoomArgument, slot).wrapping_add(amount);
    c.set_slot(Column::RoomArgument, slot, hits);
}
pub fn poll(c: &mut impl Fields, slot: usize) {
    if matches!(c.slot(Column::Counter, slot) & 0xf00, 0x100 | 0x200) {
        add_hits(c, slot, 1);
    }
    c.set_slot(Column::Counter, slot, 0);
}
#[derive(Clone, Copy)]
pub enum Command {
    Threshold,
    Shatter,
}

pub fn command(
    c: &mut impl Fields,
    position: EnemyScript,
    slot: usize,
    kind: Command,
) -> EnemyScript {
    match kind {
        Command::Threshold => {
            let hits = c.slot(Column::RoomArgument, slot);
            if hits >= position.word(0) {
                position.advance(4)
            } else {
                EnemyScript::jump(position.word(2))
            }
        }
        Command::Shatter => {
            c.sound(SoundChannel::Three, 15, 0x2e);
            for offset in [0, 2, 4, 6] {
                let parameter = position.word(offset);
                c.eproj(0xcefc, parameter);
            }
            position.advance(8)
        }
    }
}

//! Mother Brain glass hit counting, threshold progression and ordered shard bursts.
use super::super::access::*;
use super::super::content::Script;
use super::super::host::*;
use super::terrain_setup as terrain;

pub fn setup(c: &mut impl Host, slot: usize) -> u8 {
    mechanism!(c, slot, room_argument = 0);
    let block = mechanism!(c, slot, block);
    terrain::install(c, block, 0x8044);
    0
}
pub fn add_hits(c: &mut impl Host, slot: usize, amount: u16) {
    let hits = mechanism!(c, slot, room_argument).wrapping_add(amount);
    mechanism!(c, slot, room_argument = hits);
}
pub fn poll(c: &mut impl Host, slot: usize) {
    if matches!(mechanism!(c, slot, counter) & 0xf00, 0x100 | 0x200) {
        add_hits(c, slot, 1);
    }
    mechanism!(c, slot, counter = 0);
}
#[derive(Clone, Copy)]
pub enum Command {
    Threshold,
    Shatter,
}

pub fn command<H: Host>(
    c: &mut H,
    position: Script<H::Record>,
    slot: usize,
    kind: Command,
) -> Option<Script<H::Record>> {
    match kind {
        Command::Threshold => {
            let hits = mechanism!(c, slot, room_argument);
            if hits >= position.word(0) {
                Some(position.advance(4))
            } else {
                {
                    let target = position.word(2);
                    position.jump(target)
                }
            }
        }
        Command::Shatter => {
            c.sound(SoundChannel::Three, 15, 0x2e);
            for offset in [0, 2, 4, 6] {
                let parameter = position.word(offset);
                c.enemy_projectile(0xcefc, parameter);
            }
            Some(position.advance(8))
        }
    }
}

//! Tube rupture: terrain installation, input suspension and ordered effects.
#![forbid(unsafe_code)]

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
pub trait Terrain {
    fn install(&mut self, slot: usize, block: Block);
}
pub fn setup(c: &mut impl Terrain, slot: usize, kind: Setup) -> u8 {
    c.install(
        slot,
        if kind == Setup::Air {
            Block::Air
        } else {
            Block::Solid
        },
    );
    if kind == Setup::FourBlocks {
        for block in [
            Block::RightExtension,
            Block::BelowExtension,
            Block::BelowRightExtension,
        ] {
            c.install(slot, block);
        }
    }
    0
}
pub trait Wake {
    fn pressed(&self) -> bool;
    fn set_timer(&mut self, slot: usize);
    fn advance(&mut self, slot: usize);
}
pub fn poll(c: &mut impl Wake, slot: usize) {
    if c.pressed() {
        c.set_timer(slot);
        c.advance(slot);
        c.advance(slot);
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Crack,
    Shard(u8),
    Bubble(u8),
    Controls(bool),
}
pub trait Effects {
    fn water(&mut self);
    fn quake(&mut self);
    fn emit(&mut self, effect: Effect);
}
pub fn command(c: &mut impl Effects, kind: Command) {
    match kind {
        Command::Water => c.water(),
        Command::Crack => c.emit(Effect::Crack),
        Command::Quake => c.quake(),
        Command::Shatter => {
            for shard in 0..10 {
                c.emit(Effect::Shard(shard));
            }
            for bubble in 0..6 {
                c.emit(Effect::Bubble(bubble));
            }
        }
        Command::Lock => c.emit(Effect::Controls(false)),
        Command::Unlock => c.emit(Effect::Controls(true)),
    }
}

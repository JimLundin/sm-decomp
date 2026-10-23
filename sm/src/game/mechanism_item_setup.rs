//! Item placement and collision-trigger association.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Container {
    Visible,
    ShotBlock,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Graphic {
    EnergyTank,
    MissileTank,
    SuperMissileTank,
    PowerBombTank,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Initialize(Container, Option<Graphic>),
    Trigger,
}
pub trait Context {
    type Block: Copy + Eq;
    type SavedBlock;
    fn set_graphic(&mut self, slot: usize, graphic: Graphic);
    fn block(&self, slot: usize) -> Self::Block;
    fn clear_block(&mut self, slot: usize);
    fn install(&mut self, block: Self::Block, container: Container);
    fn level(&self, block: Self::Block) -> Self::SavedBlock;
    fn save_level(&mut self, slot: usize, level: Self::SavedBlock);
    fn loaded_count(&self) -> u16;
    fn set_loaded_count(&mut self, count: u16);
    fn frozen(&self) -> bool;
    fn release(&mut self, slot: usize);
    /// None preserves the legacy missing-association signal outside the slot array.
    fn signal_target(&mut self, target: Option<usize>);
}
pub fn setup(c: &mut impl Context, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Initialize(container, graphic) => {
            if let Some(graphic) = graphic {
                c.set_graphic(slot, graphic);
            }
            c.install(c.block(slot), container);
            if container == Container::ShotBlock {
                c.save_level(slot, c.level(c.block(slot)));
            }
            c.set_loaded_count(c.loaded_count().wrapping_add(1));
        }
        Setup::Trigger => {
            if c.frozen() {
                c.release(slot);
            } else {
                let block = c.block(slot);
                c.clear_block(slot);
                let found = (0..40).rev().find(|&found| c.block(found) == block);
                c.signal_target(found);
            }
        }
    }
    0
}

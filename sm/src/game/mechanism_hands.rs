//! Chozo hand contact guards, activation and nested mechanism creation.
#![forbid(unsafe_code)]

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShipScrollPair {
    First,
    Second,
}

pub trait Context {
    type Block: Copy;
    fn block_at(&self, x: u8, y: u8) -> Self::Block;
    fn install_trigger(&mut self, block: Self::Block, hand: Hand);
    fn selected_slot(&self) -> usize;
    fn set_acid_base(&mut self, level: u16);
    fn space_jump(&self) -> bool;
    fn boss_defeated(&mut self) -> bool;
    fn downward(&self) -> bool;
    fn matches_pose(&self, pose: GroundedPose) -> bool;
    fn mark_event(&mut self);
    fn signal_statue(&mut self);
    fn set_ship_scroll_pair(&mut self, pair: ShipScrollPair);
    fn block(&self, slot: usize) -> Self::Block;
    fn level(&self, block: Self::Block) -> u16;
    fn set_level(&mut self, block: Self::Block, value: u16);
    fn lock_player(&mut self);
    fn spawn_child(&mut self, hand: Hand);
    fn release(&mut self, slot: usize);
}

pub fn setup(c: &mut impl Context, slot: usize, hand: Hand) -> u8 {
    let enabled = match hand {
        Hand::LowerNorfair => c.space_jump(),
        Hand::WreckedShip => c.boss_defeated(),
    };
    if enabled
        && c.downward()
        && (c.matches_pose(GroundedPose::MorphRight)
            || c.matches_pose(GroundedPose::SpringRight)
            || c.matches_pose(GroundedPose::SpringLeft))
    {
        if hand == Hand::LowerNorfair {
            c.mark_event();
        }
        c.signal_statue();
        if hand == Hand::WreckedShip {
            c.set_ship_scroll_pair(ShipScrollPair::First);
            c.set_ship_scroll_pair(ShipScrollPair::Second);
        }
        let block = c.block(slot);
        let level = c.level(block);
        c.set_level(block, level & 0xfff);
        c.lock_player();
        c.spawn_child(hand);
    }
    // The child or another callback can reuse this captured slot.
    c.release(slot);
    1
}

pub fn prepare_ship_trigger(c: &mut impl Context, slot: usize) -> u8 {
    c.install_trigger(c.block(slot), Hand::WreckedShip);
    0
}
pub fn monitor_norfair(c: &mut impl Context) {
    let block = c.block_at(4, 8);
    if c.level(block) == 0xff {
        c.install_trigger(block, Hand::LowerNorfair);
        // The running PLM selection is read after installing the trigger.
        c.release(c.selected_slot());
    }
}
pub fn lower_acid(c: &mut impl Context) {
    c.set_acid_base(722);
}

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

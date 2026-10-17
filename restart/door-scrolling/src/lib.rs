//! Complete directional door-scrolling boundary from pinned recovered C bank 80.
//! The native IRQ caller owns scheduling and its completion-bit admission guard.
mod memory;
mod scrolling;

pub use memory::WRAM_SIZE;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Entry {
    FixDoorsMovingUp = 0,
    ScrollingSetup = 1,
    FollowTransition = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Handled,
    Refused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Routine {
    FixDoorsMovingUp = 0,
    ScrollingSetup = 1,
    FollowTransition = 2,
    SetupRight = 3,
    SetupLeft = 4,
    SetupDown = 5,
    SetupUp = 6,
    UpdatePreviousBlocks = 7,
    UpdateBgOffsets = 8,
    MoveRight = 9,
    MoveLeft = 10,
    MoveDown = 11,
    MoveUp = 12,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    Enter { routine: Routine, counter: u16 },
    Return { routine: Routine, result: u8, counter: u16 },
}

/// Services exchange the complete current WRAM synchronously. The native host
/// composes update_scroll_vars with the installed room-scrolling entry 1.
/// Observation is passive: no RAM synchronization or gameplay invocation.
/// An error after admission is an owned failure and prohibits C retry.
pub trait Host {
    fn calculate_layer2_x(&mut self, ram: &mut [u8]) -> Result<u8>;
    fn calculate_layer2_y(&mut self, ram: &mut [u8]) -> Result<u8>;
    fn calculate_blocks(&mut self, ram: &mut [u8]) -> Result<()>;
    fn update_scroll_vars(&mut self, ram: &mut [u8]) -> Result<()>;
    fn observe(&mut self, observation: Observation) -> Result<()>;
}

/// Execute one synchronous root. All four direction values (direction & 3)
/// are supported, including an already-set completion bit. Invalid WRAM extent
/// errors before observations or mutation. No state or borrow survives return.
pub fn run(entry: Entry, ram: &mut [u8], host: &mut impl Host) -> Result<Outcome> {
    let ram = memory::Wram::new(ram)?;
    let routine = match entry {
        Entry::FixDoorsMovingUp => Routine::FixDoorsMovingUp,
        Entry::ScrollingSetup => Routine::ScrollingSetup,
        Entry::FollowTransition => Routine::FollowTransition,
    };
    scrolling::Scrolling { ram, host }.call(routine)?;
    Ok(Outcome::Handled)
}

#[cfg(test)]
mod tests;

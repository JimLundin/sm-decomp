//! The seven-definition vertical movement kernel over canonical full WRAM.
//! Terrain collision is one synchronous external dependency, not Rust ownership.
#![forbid(unsafe_code)]

mod feedback;
mod memory;
mod solid_enemy;
mod vertical;

#[cfg(any(test, feature = "fixture-support"))]
mod fixtures;
#[cfg(feature = "fixture-support")]
pub use fixtures::{BodyFixture, body_fixtures};
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Entry {
    MoveUp(i32),
    MoveDown(i32),
    CheckSolidEnemy(i32),
    MoveUpNoColl(i32),
    MoveDownNoColl(i32),
    UpFeedback,
    DownFeedback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Output {
    Done,
    EnemyCollision { collision: u16, amount: i32 },
}

pub trait Terrain {
    /// Immutable admission established from incoming canonical state/entry.
    /// Must not mutate, request a native call, or produce an observation.
    fn available(&self) -> bool;

    /// Executes 94:9763 once synchronously, publishing and copying back full
    /// WRAM. A failure after this request is fatal; it must not become refusal.
    fn move_vertical(&mut self, ram: &mut [u8], amount: i32) -> Result<i32, String>;
}

/// `None` is an immutable no-request refusal. Errors after acceptance retain
/// the accepted effects for the caller's fatal path; no native retry is valid.
pub fn execute(entry: Entry, ram: &mut [u8], terrain: &mut impl Terrain) -> Result<Option<Output>, String> {
    use memory::*;
    if ram.len() != SIZE || matches!(entry, Entry::MoveUp(i32::MIN) | Entry::MoveUpNoColl(i32::MIN)) {
        return Ok(None);
    }
    match entry {
        Entry::MoveUp(amount) | Entry::MoveDown(amount) => {
            if !solid_enemy::list_supported(ram) {
                return Ok(None);
            }
            let upward = matches!(entry, Entry::MoveUp(_));
            let direction = if upward { 2 } else { 3 };
            let contact = solid_enemy::inspect(ram, if upward { -amount } else { amount }, direction);
            if !upward
                && (if contact.collision == 0 {
                    // The foreign result can make either feedback table reachable.
                    ram[MOVEMENT] >= 28
                } else {
                    !feedback::down_supported(ram, contact.collision)
                })
            {
                return Ok(None);
            }
            if contact.collision == 0 && !terrain.available() {
                return Ok(None);
            }
            vertical::move_root(ram, contact, upward, terrain)?;
        }
        Entry::CheckSolidEnemy(amount) => {
            if !solid_enemy::list_supported(ram) {
                return Ok(None);
            }
            let contact = solid_enemy::inspect(ram, amount, word(ram, DIRECTION));
            contact.apply(ram);
            return Ok(Some(Output::EnemyCollision { collision: contact.collision, amount: contact.amount }));
        }
        Entry::MoveUpNoColl(amount) => vertical::no_collision(ram, amount, true),
        Entry::MoveDownNoColl(amount) => vertical::no_collision(ram, amount, false),
        Entry::UpFeedback => feedback::up(ram),
        Entry::DownFeedback => {
            if !feedback::down_supported(ram, word(ram, COLLISION)) {
                return Ok(None);
            }
            feedback::down(ram)?;
        }
    }
    Ok(Some(Output::Done))
}

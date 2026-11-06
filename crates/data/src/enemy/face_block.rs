//! The Blue Brinstar face block's instruction lists, by what its AI starts them for (bank $A8
//! log, <https://patrickjohnston.org/bank/A8>, `$A8:E80C` to `$A8:E828`).

use crate::rom::RomAddress;

use super::program::{EnemyPrograms, ProgramError, Target};

/// The face block's AI bank.
const BANK: u8 = 0xa8;

/// The face block's lists as places in the loaded programs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaceBlockLists {
    /// Turning to look left, then sleeping (`$A8:E80C`).
    pub turning_left: Target,
    /// Turning to look right, then sleeping (`$A8:E81A`).
    pub turning_right: Target,
    /// Facing forward, sleeping (`$A8:E828`), which its init starts.
    pub facing_forward: Target,
}

impl FaceBlockLists {
    /// The face block's lists in `programs`.
    ///
    /// # Errors
    ///
    /// [`ProgramError::Unloaded`] for a list `programs` doesn't hold.
    pub fn resolve(programs: &EnemyPrograms) -> Result<Self, ProgramError> {
        let place = |offset: u16| {
            let at = RomAddress::new(BANK, offset);
            programs.target(at).ok_or(ProgramError::Unloaded(at))
        };
        Ok(Self {
            turning_left: place(0xe80c)?,
            turning_right: place(0xe81a)?,
            facing_forward: place(0xe828)?,
        })
    }
}

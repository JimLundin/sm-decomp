//! Spec: `EggHatches`, `PageThreeApproaches`. The egg waits for Samus, rocks and opens, and its
//! opening cues page three.

use types::discovery::Egg;

use crate::rules::Rules;
use crate::scene::{ShellPictures, SpriteRef};

/// What the egg did on a finished frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EggTurn {
    /// Nothing new: absent, still waiting, or already spent.
    Unchanged,
    /// The hatch shows the picture of this age: the hatch markers read it.
    Hatches(u16),
    /// The hatch animation's last hold ended: the shell is spent and page three is cued.
    Opens,
}

/// Spec: `EggHatches`, `PageThreeApproaches`. One finished frame of the egg. Waiting, it starts
/// hatching at age zero once Samus's whole-pixel x is below the approach threshold; hatching, it
/// advances one age, and becomes spent once the last hold ends.
pub fn advance(egg: &mut Egg, shell: &ShellPictures, rules: &Rules, samus_x: i32) -> EggTurn {
    match *egg {
        Egg::Waiting if samus_x < rules.approach_x => {
            *egg = Egg::Hatching { age: 0 };
            EggTurn::Hatches(0)
        }
        Egg::Absent | Egg::Waiting | Egg::Spent => EggTurn::Unchanged,
        Egg::Hatching { age } => {
            let age = age.saturating_add(1);
            if shell.hatch.holds(age) {
                *egg = Egg::Hatching { age };
                EggTurn::Hatches(age)
            } else {
                *egg = Egg::Spent;
                EggTurn::Opens
            }
        }
    }
}

/// Spec: `EggHatches`, `PageThreeApproaches`. The egg's picture: closed while it waits, the hatch
/// picture whose hold includes its age, then the spent shell; none while absent.
#[must_use]
pub fn picture(egg: Egg, shell: &ShellPictures) -> Option<SpriteRef> {
    match egg {
        Egg::Absent => None,
        Egg::Waiting => Some(shell.closed),
        Egg::Hatching { age } => shell.hatch.picture_at(age),
        Egg::Spent => Some(shell.spent),
    }
}

//! The values the HUD's rules read: the reference profile holds the spec's; another `Rules` is an
//! adjusted game.

use enum_map::{Enum, EnumMap};
use types::Screen;

use crate::inputs::FrameCount;

/// A boss that blanks the mini-map and turns it off (spec: BossBlanksTheMap).
///
/// Each but Mother Brain is met on entering its room while it lives, a step of its own outside
/// a frame of play; Mother Brain blanks it when her first form falls, during a frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum Boss {
    /// Kraid.
    Kraid,
    /// Draygon.
    Draygon,
    /// Crocomire.
    Crocomire,
    /// Phantoon.
    Phantoon,
    /// Mother Brain, her brain in the tank destroyed.
    MotherBrain,
    /// Ridley, in Norfair.
    RidleyInNorfair,
    /// Ridley, in Ceres.
    RidleyInCeres,
}

/// Spec: BossBlanksTheMap. The reference screens each boss counts as explored, across and down
/// from its room's top-left cell: the spec's table.
const fn reference_screens(boss: Boss) -> &'static [Screen] {
    match boss {
        Boss::Kraid | Boss::Draygon => &[
            Screen { x: 0, y: 0 },
            Screen { x: 1, y: 0 },
            Screen { x: 0, y: 1 },
            Screen { x: 1, y: 1 },
        ],
        Boss::Crocomire => &[
            Screen { x: 3, y: 0 },
            Screen { x: 4, y: 0 },
            Screen { x: 5, y: 0 },
        ],
        Boss::Phantoon => &[Screen { x: 0, y: 0 }],
        Boss::MotherBrain => &[
            Screen { x: 0, y: 0 },
            Screen { x: 1, y: 0 },
            Screen { x: 2, y: 0 },
            Screen { x: 3, y: 0 },
        ],
        Boss::RidleyInNorfair => &[Screen { x: 0, y: 0 }, Screen { x: 0, y: 1 }],
        Boss::RidleyInCeres => &[],
    }
}

/// A blink or a flash: so many frames one way, then as many the other, by the frame count.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Alternation {
    /// The frames each way.
    pub frames_each_way: u8,
}

impl Alternation {
    /// Whether `frame` lies in the first half of the cycle: `frame mod (2 × n) < n`, `n` the
    /// frames each way. With none each way, every frame is in the second half.
    #[must_use]
    pub fn first_half(self, frame: FrameCount) -> bool {
        let each_way = u16::from(self.frames_each_way);
        u16::from(frame.0)
            .checked_rem(each_way.saturating_add(each_way))
            .is_some_and(|phase| phase < each_way)
    }
}

/// Spec: OneShotFlash. The reference frames each way of the one-shot item's flash.
const REFERENCE_ONE_SHOT_FLASH_FRAMES: u8 = 16;

/// Spec: HerBlink. The reference frames each way of her cell's blink.
const REFERENCE_HER_BLINK_FRAMES: u8 = 8;

/// The values the HUD's rules read: every Parameter the spec lists.
///
/// They are the frames each way of the one-shot item's flash and of her cell's blink, and the
/// screens each boss explores. Its default is the reference profile, the spec's values; a caller
/// holds it and may adjust each.
#[derive(Clone, Debug)]
pub struct Rules {
    /// Spec: OneShotFlash. The one-shot item's flash.
    pub one_shot_flash: Alternation,
    /// Spec: HerBlink. Her mini-map cell's blink.
    pub her_blink: Alternation,
    /// Spec: BossBlanksTheMap. The screens of its room each boss counts as explored, across and
    /// down from the room's top-left cell.
    pub boss_screens: EnumMap<Boss, Vec<Screen>>,
}

impl Rules {
    /// The reference profile: the spec's values. The flash is 16 frames each way, the blink 8,
    /// and the bosses' screens the spec's table.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            one_shot_flash: Alternation {
                frames_each_way: REFERENCE_ONE_SHOT_FLASH_FRAMES,
            },
            her_blink: Alternation {
                frames_each_way: REFERENCE_HER_BLINK_FRAMES,
            },
            boss_screens: EnumMap::from_fn(|boss| reference_screens(boss).to_vec()),
        }
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::reference()
    }
}

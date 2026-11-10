//! The values the HUD's rules read: the reference profile holds the spec's; another `Rules` is an
//! adjusted game.

use clean_types::{Screen, by_member};
use enum_map::{Enum, EnumMap};

use crate::inputs::FrameCount;

/// A boss met on entering its room while it lives (spec: BossBlanksTheMap).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum RoomBoss {
    /// Kraid.
    Kraid,
    /// Draygon.
    Draygon,
    /// Crocomire.
    Crocomire,
    /// Phantoon.
    Phantoon,
    /// Ridley, in Norfair.
    RidleyInNorfair,
    /// Ridley, in Ceres.
    RidleyInCeres,
}

/// A boss that blanks the mini-map and turns it off (spec: BossBlanksTheMap).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum Boss {
    /// A boss met on entering its room, a step of its own.
    InItsRoom(RoomBoss),
    /// Mother Brain, when her first form falls during a frame (her brain in the tank destroyed).
    MotherBrain,
}

/// A screen `x` across and `y` down from a room's top-left cell.
const fn screen(x: u16, y: u16) -> Screen {
    Screen { x, y }
}

/// A row of the spec's table of the bosses' screens.
struct BossScreens {
    /// The boss.
    boss: Boss,
    /// The screens of its room it counts as explored.
    screens: &'static [Screen],
}

/// A room's top-left 2 by 2 screens: Kraid's and Draygon's.
const TWO_BY_TWO: &[Screen] = &[screen(0, 0), screen(1, 0), screen(0, 1), screen(1, 1)];

/// Spec: BossBlanksTheMap. The screens each boss counts as explored, the spec's table.
const VANILLA_BOSS_SCREENS: [BossScreens; 7] = [
    BossScreens {
        boss: Boss::InItsRoom(RoomBoss::Kraid),
        screens: TWO_BY_TWO,
    },
    BossScreens {
        boss: Boss::InItsRoom(RoomBoss::Draygon),
        screens: TWO_BY_TWO,
    },
    BossScreens {
        boss: Boss::InItsRoom(RoomBoss::Crocomire),
        screens: &[screen(3, 0), screen(4, 0), screen(5, 0)],
    },
    BossScreens {
        boss: Boss::InItsRoom(RoomBoss::Phantoon),
        screens: &[screen(0, 0)],
    },
    BossScreens {
        boss: Boss::MotherBrain,
        screens: &[screen(0, 0), screen(1, 0), screen(2, 0), screen(3, 0)],
    },
    BossScreens {
        boss: Boss::InItsRoom(RoomBoss::RidleyInNorfair),
        screens: &[screen(0, 0), screen(0, 1)],
    },
    BossScreens {
        boss: Boss::InItsRoom(RoomBoss::RidleyInCeres),
        screens: &[],
    },
];

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

/// Spec: OneShotFlash. The frames each way of the one-shot item's flash.
const VANILLA_ONE_SHOT_FLASH_FRAMES: u8 = 16;

/// Spec: HerBlink. The frames each way of her cell's blink.
const VANILLA_HER_BLINK_FRAMES: u8 = 8;

/// The values the HUD's rules read.
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
                frames_each_way: VANILLA_ONE_SHOT_FLASH_FRAMES,
            },
            her_blink: Alternation {
                frames_each_way: VANILLA_HER_BLINK_FRAMES,
            },
            boss_screens: by_member(VANILLA_BOSS_SCREENS, |row| row.boss)
                .map(|_, row| row.screens.to_vec()),
        }
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::reference()
    }
}

#[cfg(test)]
mod tests {
    use super::{Boss, VANILLA_BOSS_SCREENS};
    use crate::tests::names_each_member_once;

    #[test]
    fn the_boss_table_names_each_boss_once() {
        assert!(names_each_member_once::<Boss, _>(
            &VANILLA_BOSS_SCREENS,
            |row| row.boss
        ));
    }
}

//! The values the HUD's rules read: the reference profile holds the spec's; another `Rules` is an
//! adjusted game.

use clean_types::Pair;

use crate::pictures::Picture;

/// A boss that turns the mini-map off (spec: `BossBlanksTheMap`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Boss {
    /// Kraid, on entering his room while he lives.
    Kraid,
    /// Draygon, on entering her room while she lives.
    Draygon,
    /// Crocomire, on entering his room while he lives.
    Crocomire,
    /// Phantoon, on entering his room while he lives.
    Phantoon,
    /// Mother Brain, when her first form falls (her brain in the tank destroyed).
    MotherBrain,
    /// Ridley in Norfair, on entering his room while he lives.
    RidleyInNorfair,
    /// Ridley in Ceres, on entering his room while he lives.
    RidleyInCeres,
}

impl Boss {
    /// Every boss, in the order `Rules::boss_screens` holds their screens.
    pub const ALL: [Self; 7] = [
        Self::Kraid,
        Self::Draygon,
        Self::Crocomire,
        Self::Phantoon,
        Self::MotherBrain,
        Self::RidleyInNorfair,
        Self::RidleyInCeres,
    ];
}

/// A screen of a room: how many screens across and down from its top-left cell.
pub type Screen = Pair<u16>;

/// A screen `x` across and `y` down from a room's top-left cell.
const fn screen(x: u16, y: u16) -> Screen {
    Pair { x, y }
}

/// Kraid's and Draygon's screens: a room of 2 by 2.
const TWO_BY_TWO: &[Screen] = &[screen(0, 0), screen(1, 0), screen(0, 1), screen(1, 1)];

/// The spec's table of the screens each boss explores, in the order of `Boss::ALL` (spec:
/// `BossBlanksTheMap`).
const VANILLA_BOSS_SCREENS: [&[Screen]; Boss::ALL.len()] = [
    TWO_BY_TWO,
    TWO_BY_TWO,
    &[screen(3, 0), screen(4, 0), screen(5, 0)],
    &[screen(0, 0)],
    &[screen(0, 0), screen(1, 0), screen(2, 0), screen(3, 0)],
    &[screen(0, 0), screen(0, 1)],
    &[],
];

/// A blink or a flash: so many frames one way, then as many the other, by the frame count.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Alternation {
    /// The frames each way.
    pub frames_each_way: u8,
}

impl Alternation {
    /// Whether the frame count lies in the cycle's first half: `frame mod (2 × n) < n`. With no
    /// frames each way, every frame is in the second half.
    #[must_use]
    pub fn first_half(self, frame: u8) -> bool {
        let each_way = u16::from(self.frames_each_way);
        u16::from(frame)
            .checked_rem(each_way.saturating_add(each_way))
            .is_some_and(|phase| phase < each_way)
    }
}

/// The one-shot item's flash: 16 frames each way (spec: `OneShotFlash`).
const VANILLA_ONE_SHOT_FLASH_FRAMES: u8 = 16;

/// Her mini-map cell's blink: 8 frames each way (spec: `HerBlink`).
const VANILLA_HER_BLINK_FRAMES: u8 = 8;

/// The picture whose number's last nine bits mark her cell as a diagonal: `028` (spec:
/// `DiagonalPicture`).
const VANILLA_DIAGONAL_PICTURE: u16 = 0x028;

/// The values the HUD's rules read.
#[derive(Clone, Debug)]
pub struct Rules {
    /// Spec: OneShotFlash. The one-shot item's icon: plain for the first half of the cycle,
    /// highlighted for the second.
    pub one_shot_flash: Alternation,
    /// Spec: HerBlink. Her mini-map cell: in the blink colours for the first half of the cycle,
    /// as drawn for the second.
    pub her_blink: Alternation,
    /// Spec: DiagonalPicture. The picture whose number's last nine bits mark her cell as drawn
    /// as a diagonal.
    pub diagonal_picture: Picture,
    /// Spec: BossBlanksTheMap. The screens of its room each boss counts as explored, one entry
    /// per boss in the order of `Boss::ALL`.
    pub boss_screens: [Vec<Screen>; Boss::ALL.len()],
}

impl Rules {
    /// The reference profile: the spec's values. The flash is 16 frames each way, the blink 8,
    /// the diagonal picture `028`, and the bosses' screens the spec's table.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            one_shot_flash: Alternation {
                frames_each_way: VANILLA_ONE_SHOT_FLASH_FRAMES,
            },
            her_blink: Alternation {
                frames_each_way: VANILLA_HER_BLINK_FRAMES,
            },
            diagonal_picture: Picture::new(VANILLA_DIAGONAL_PICTURE),
            boss_screens: VANILLA_BOSS_SCREENS.map(<[Screen]>::to_vec),
        }
    }

    /// The screens of its room `boss` counts as explored.
    #[must_use]
    pub fn screens_of(&self, boss: Boss) -> &[Screen] {
        Boss::ALL
            .iter()
            .zip(&self.boss_screens)
            .find(|&(&each, _)| each == boss)
            .map_or(&[], |(_, screens)| screens.as_slice())
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::reference()
    }
}

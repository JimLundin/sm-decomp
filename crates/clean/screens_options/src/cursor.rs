//! Spec: `OptionsCursor`, `SpecialSettings`. Up and Down move a cursor round its menu's items.

use types::options::{OptionsItem, SpecialItem};

use crate::frame::{MenuAction, MenuSound, Pressed};

/// A menu's items in a ring: above the top is the bottom, below the bottom the top.
pub trait Ring: Copy {
    /// The item above this one.
    fn above(self) -> Self;
    /// The item below this one.
    fn below(self) -> Self;
}

/// Spec: `OptionsCursor`. Start game, English text, Japanese text, the controller settings, the
/// special settings.
impl Ring for OptionsItem {
    fn above(self) -> Self {
        match self {
            Self::StartGame => Self::SpecialSettings,
            Self::EnglishText => Self::StartGame,
            Self::JapaneseText => Self::EnglishText,
            Self::ControllerSettings => Self::JapaneseText,
            Self::SpecialSettings => Self::ControllerSettings,
        }
    }

    fn below(self) -> Self {
        match self {
            Self::StartGame => Self::EnglishText,
            Self::EnglishText => Self::JapaneseText,
            Self::JapaneseText => Self::ControllerSettings,
            Self::ControllerSettings => Self::SpecialSettings,
            Self::SpecialSettings => Self::StartGame,
        }
    }
}

/// Spec: `SpecialSettings`. Weapon-reset, moonwalk, end.
impl Ring for SpecialItem {
    fn above(self) -> Self {
        match self {
            Self::WeaponReset => Self::End,
            Self::Moonwalk => Self::WeaponReset,
            Self::End => Self::Moonwalk,
        }
    }

    fn below(self) -> Self {
        match self {
            Self::WeaponReset => Self::Moonwalk,
            Self::Moonwalk => Self::End,
            Self::End => Self::WeaponReset,
        }
    }
}

/// The cursor after Up or Down, asking for `CursorMoved` when it moves. Up wins over Down
/// (`OptionsCursor`'s C-up-beats-down).
pub fn moved<I: Ring>(cursor: I, pressed: Pressed, sounds: &mut Vec<MenuSound>) -> I {
    let moved = if pressed[MenuAction::Up] {
        cursor.above()
    } else if pressed[MenuAction::Down] {
        cursor.below()
    } else {
        return cursor;
    };
    sounds.push(MenuSound::CursorMoved);
    moved
}

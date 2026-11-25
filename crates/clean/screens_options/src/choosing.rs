//! Spec: `OptionsCursor`, `OptionsChoice`. The lit menu: the cursor moves first, then B goes back
//! to file select, or A or Start acts on the item the cursor is now on.

use types::options::{
    DissolveCursor, FileBegins, Language, OptionsItem, OptionsStage, Settings, WayOut,
};

use crate::cursor;
use crate::dissolve;
use crate::frame::{ChosenFile, MenuAction, MenuSound, Pressed};

/// Spec: `OptionsCursor`, `OptionsChoice`. One finished frame of the lit menu. B wins over A and
/// Start; a choice asks for `Selected`.
pub fn choose(
    cursor: OptionsItem,
    pressed: Pressed,
    file: ChosenFile,
    settings: &mut Settings,
    sounds: &mut Vec<MenuSound>,
) -> OptionsStage {
    let cursor = cursor::moved(cursor, pressed, sounds);
    if pressed[MenuAction::Back] {
        return OptionsStage::Returning(cursor);
    }
    if !pressed[MenuAction::Choose] {
        return OptionsStage::Choosing(cursor);
    }
    sounds.push(MenuSound::Selected);
    match cursor {
        OptionsItem::StartGame => start_game(file),
        OptionsItem::EnglishText => pick_language(settings, Language::English),
        OptionsItem::JapaneseText => pick_language(settings, Language::Japanese),
        OptionsItem::ControllerSettings => dissolve::start(DissolveCursor::ControllerSettings),
        OptionsItem::SpecialSettings => dissolve::start(DissolveCursor::SpecialSettings),
    }
}

/// Spec: `OptionsChoice`. Start game: a file that begins with the intro or the escape leaves
/// through the dark, keeping how it begins; one that goes straight on in play hands on at once.
const fn start_game(file: ChosenFile) -> OptionsStage {
    match file {
        ChosenFile::Begins(begins) => leaving(begins),
        ChosenFile::InPlay => OptionsStage::HandedOn(WayOut::PlayContinues),
    }
}

/// Spec: `OptionsChoice`. Leaving for the game, the cursor still on start game.
const fn leaving(begins: FileBegins) -> OptionsStage {
    OptionsStage::Leaving {
        cursor: OptionsItem::StartGame,
        begins,
    }
}

/// Spec: `OptionsChoice`. English or Japanese text: the text in that language, the cursor back on
/// start game.
const fn pick_language(settings: &mut Settings, language: Language) -> OptionsStage {
    settings.language = language;
    OptionsStage::Choosing(OptionsItem::StartGame)
}

//! The catalogue: one behaviour per entry of the intro's spec, with its kind, its switch and how
//! it is checked.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch, by_member};

/// A behaviour of the spec, by its entry id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// Picture scripts step through their pictures.
    ScriptSteps,
    /// The colours dim and brighten every fourth frame.
    PicturesDissolve,
    /// A dissolve ends: the text cleared, or the caret's colours shown.
    DissolveEnds,
    /// The baby rises as the picture settles.
    BabyRises,
    /// The baby is gone as the text returns.
    BabyLeaves,
    /// Page 5 begins on the baby's signal.
    PageFiveBegins,
    /// A letter appears, clicks every other turn, and the caret moves on.
    LetterIsTyped,
    /// A new letter glows through its colour sets.
    LetterGlows,
    /// The caret runs its steady or blinking script.
    CaretBlinks,
    /// A page's end: wait for a press, or start the fade.
    PageEnds,
    /// A press turns to page 6.
    PressTurnsThePage,
    /// The picture darkens a level every `d + 1` frames.
    PictureDarkens,
    /// The intro ends once the picture is blanked.
    IntroEnds,
    /// The intro starts dark, asks for its first tune and lends her equipment.
    IntroBegins,
    /// The opening and page 1 brighten or darken a level every 2 frames.
    SlowFade,
    /// The opening's waits, brightening, holds, tunes and darkening, in order.
    OpeningUnfolds,
    /// Page 1's eyes, caret, picture and tune made ready in the dark.
    PageOneIsMadeReady,
    /// Page 1 types while its picture brightens.
    PageOneBegins,
    /// The press after page 1 begins the fight on Zebes.
    BattleBegins,
    /// The fight and the text exchange their colours as the picture dissolves each way.
    BattleDissolves,
    /// Page 2 starts typing as the fight fades away.
    PageTwoBegins,
    /// Page 2 ends with a blinking caret and a wait for a press.
    PageTwoEnds,
}

/// A catalogue entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its entry id.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// The number of the spec's entries.
const ENTRIES: usize = 22;

/// The catalogue's rows, one per spec entry, in the spec's order.
const ROWS: [Behaviour; ENTRIES] = [
    Behaviour {
        id: BehaviourId::ScriptSteps,
        summary: "a script shows one picture at a time; when its hold runs out it moves on to its \
                  next step, going back or repeating where the step says, or handing the scene a \
                  signal",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PicturesDissolve,
        summary: "in a dissolve, every fourth frame each colour of one set dims by its step and \
                  each of the other brightens by its step, and the screen shows the new colours; \
                  after 128 frames the dissolve ends",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::DissolveEnds,
        summary: "when a dissolve's count runs past 0 it ends: to the scene, the text is cleared; \
                  to the text, the caret takes its colours",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BabyRises,
        summary: "as the picture dissolves to the scene, on each frame that leaves the dissolve's \
                  count a multiple of 4 the picture moves down a pixel and the baby rises a \
                  pixel, until the picture's height reaches 8",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BabyLeaves,
        summary: "once the dissolve back to the text reaches its last count, the baby is gone",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PageFiveBegins,
        summary: "the baby's script signals page 5: on the next frame the page's script is placed \
                  and the picture starts back toward the text, its text colours black and every \
                  other colour at its full",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LetterIsTyped,
        summary: "each letter of a page appears as its step begins, glowing; the caret moves to \
                  where the next letter goes; every other letter clicks, spaces silent",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LetterGlows,
        summary: "a new letter glows through its colour sets: set 0 in the frame it appears, then \
                  sets 1, 2 and 3 five frames apart; it keeps set 3",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CaretBlinks,
        summary: "the caret runs its steady script while a page types and its blinking script \
                  once the page waits; it shows whenever its script's step is the caret's picture",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PageEnds,
        summary: "at the end of page 1 and of page 5 the game waits for a press; at the end of \
                  page 6 the page holds, and the picture starts to fade out",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PressTurnsThePage,
        summary: "while the game waits, a press clears the text, page 6 begins, Samus's eyes \
                  start to stare, and the music is asked to stop",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PictureDarkens,
        summary: "the picture darkens one level every d + 1 frames; at level 1 the next step \
                  blanks it",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::IntroEnds,
        summary: "once the picture is black and blanked, the intro is over: everything of the \
                  scene is cleared, the equipment lent for the played scenes is returned, and the \
                  next frame is the flight's",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::IntroBegins,
        summary: "the intro's first frame clears the scene, keeps the screen dark, asks for the \
                  first tune and lends the equipment the played scenes use",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SlowFade,
        summary: "in the opening and as page 1 appears, the picture brightens or darkens one \
                  level every 2 frames; brightening stops at 15, darkening blanks the screen as \
                  it reaches 0",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::OpeningUnfolds,
        summary: "the opening waits for the first tune to load, brightens, holds 60 frames, asks \
                  the first tune to play, holds 200 frames, asks for the second tune, waits for \
                  it to load, holds 240 frames and darkens",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PageOneIsMadeReady,
        summary: "in the dark, the text is cleared, the picture stands at 8, Samus's eyes start \
                  to blink, the caret waits at its home, and the pages' tune is asked for",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PageOneBegins,
        summary: "once the pages' tune has loaded, page 1 starts to type and the picture slowly \
                  brightens; once it is full the text goes on typing",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BattleBegins,
        summary: "the press after page 1 begins the fight and readies the dissolve from the text",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BattleDissolves,
        summary: "the flashback and text exchange their colours as the picture dissolves each way",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PageTwoBegins,
        summary: "after the explosions around Mother Brain, page 2 starts typing as the fight \
                  fades away",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PageTwoEnds,
        summary: "page 2 ends with a blinking caret and a wait for the press that starts the \
                  baby's discovery",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];

/// The catalogue: each spec entry's behaviour, by its id.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    by_member(ROWS, |row| row.id)
}

#[cfg(test)]
mod tests {
    use super::{ROWS, catalogue};

    /// The rows name each entry once: the catalogue holds each row under its own id.
    #[test]
    fn each_entry_has_one_row() {
        for (id, row) in catalogue() {
            assert_eq!(row.id, id);
            assert_eq!(ROWS.iter().filter(|other| other.id == id).count(), 1);
        }
    }
}

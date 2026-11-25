//! The intro's scenes in its own format (docs/architecture/assets.md).
//!
//! Page 1's and the last pages' picture scripts, the colours the last pages' dissolves head for
//! and move, the caret's colours and its home.
//!
//! A script is a timeline of steps, each showing a picture, a letter or nothing for its frames,
//! or going back, repeating a stretch, or handing the scene a cue. The pictures are named by
//! what they show; how they look is the platform's.

pub use types::intro::{StepAt, TextPlace};
pub use types::{Colour, Palette};

/// A picture a script shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Picture {
    /// Samus's eyes, open.
    EyesOpen,
    /// Samus's eyes, half open.
    EyesHalfOpen,
    /// Samus's eyes, closed.
    EyesClosed,
    /// Samus's eyes, staring.
    EyesStaring,
    /// The baby, its first frame.
    BabyFirst,
    /// The baby, its second frame.
    BabySecond,
    /// The baby, its third frame.
    BabyThird,
    /// The text's caret.
    Caret,
}

/// What a step shows while it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shows {
    /// Nothing.
    Nothing,
    /// A picture.
    Picture(Picture),
    /// A letter of a page, typed at its place as the step begins.
    Letter {
        /// The letter; a space is a letter too.
        glyph: char,
        /// Its place on the text layer.
        at: TextPlace,
    },
}

/// A cue a script hands the scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    /// The baby's: page 5 begins.
    PageFive,
    /// Page 5's end: the game waits for a press.
    WaitForPress,
    /// Page 1's end: the game waits for the press that starts the battle on Zebes.
    WaitForPressThenFight,
    /// The caret starts to blink.
    CaretBlinks,
    /// Page 6's end: the picture fades out.
    Finish,
}

/// One step of a script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Show this for `hold` frames.
    Show {
        /// What shows.
        shows: Shows,
        /// For how many frames.
        hold: u16,
    },
    /// Go back to this step.
    Back(usize),
    /// Play the stretch from step `from` to here `times` times in all, then go on.
    Repeat {
        /// The stretch's first step.
        from: usize,
        /// The times it plays in all.
        times: u16,
    },
    /// Hand the scene this cue and go on.
    Cue(Cue),
    /// The script ends.
    End,
}

/// A dissolve's colours, by index: those that dim to black and those that brighten from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DissolveSets {
    /// The colours that dim.
    pub dims: Vec<u8>,
    /// The colours that brighten.
    pub brightens: Vec<u8>,
}

/// The intro's scenes: page 1 and the last pages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scene {
    /// Page 1's letters, places and holds, and its cue.
    pub page_one: Vec<Step>,
    /// Page 5's letters, places and holds, and its cues.
    pub page_five: Vec<Step>,
    /// Page 6's.
    pub page_six: Vec<Step>,
    /// Samus's eyes as they blink, through page 5.
    pub eyes_blinking: Vec<Step>,
    /// Samus's eyes from page 6 on.
    pub eyes_staring: Vec<Step>,
    /// The baby being examined.
    pub baby: Vec<Step>,
    /// The caret while a page is typed.
    pub caret_steady: Vec<Step>,
    /// The caret while a page waits.
    pub caret_blinking: Vec<Step>,
    /// The scene's 256 colours at their full.
    pub full: Palette,
    /// The dissolve from page 4's text to the picture.
    pub to_scene: DissolveSets,
    /// The dissolve from the picture to page 5's text.
    pub to_text: DissolveSets,
    /// The text's colours, which start black as page 5 begins.
    pub text: Vec<u8>,
    /// The colours the caret takes once the text is back, by index.
    pub caret_colours: Vec<(u8, Colour)>,
    /// The caret's place as a page begins, px: x and y.
    pub caret_home: [u16; 2],
}

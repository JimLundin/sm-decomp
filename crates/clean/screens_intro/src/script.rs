//! Picture scripts (`ScriptSteps`): the one mechanism behind every scripted picture of the
//! intro (the pages, Samus's eyes, the baby and the caret). A script shows one picture at a time
//! and holds it for its frames; when the hold runs out it moves on to its next step, going back
//! or repeating where the step says, or handing the scene a signal.

use types::intro::{Progress, StepAt, TextPlace};

/// The name content gives a picture: a look of the eyes, a frame of the baby, the caret. What it
/// looks like is the platform's to draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureName(pub String);

/// A letter of a page, at its place on the text layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Letter {
    /// The letter; a space is typed like any other, silently.
    pub glyph: char,
    /// Its place.
    pub at: TextPlace,
}

/// What a step shows while it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shows {
    /// Nothing: the script's picture is hidden.
    Nothing,
    /// A picture.
    Image(PictureName),
    /// A page's letter, typed as the step begins (`LetterIsTyped`).
    Text(Letter),
}

/// A signal a script hands the scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    /// The baby's: page 5 begins (`PageFiveBegins`).
    PageFive,
    /// Page 5's end: the game waits for a press, the caret blinking (`PageEnds`, `CaretBlinks`).
    WaitForPress,
    /// Page 1's end: the game waits for the press that starts the battle on Zebes, the caret
    /// blinking (`PageEnds`, `CaretBlinks`).
    WaitForPressThenFight,
    /// The caret starts to blink (`CaretBlinks`).
    CaretBlinks,
    /// Page 6's end: the picture starts to fade out (`PageEnds`).
    Finish,
}

/// One step of a script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Show this, holding it for these frames.
    Show {
        /// What shows.
        shows: Shows,
        /// For how many frames.
        hold: u16,
    },
    /// Go back to this step, in the same frame.
    Back(usize),
    /// Repeat from this step, this many times in all; then go on past it.
    Repeat {
        /// The step the stretch starts at.
        from: usize,
        /// The times the stretch plays in all.
        times: u16,
    },
    /// Hand the scene this signal and go on, in the same frame.
    Hand(Signal),
    /// The end: the script is removed.
    End,
}

/// A script: its steps, in order, numbered from 0.
pub type Script = Vec<Step>;

/// What a script did as it ran this frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ran {
    /// Where it has got to, or none if it ended and is removed.
    pub progress: Option<Progress>,
    /// The step that began this frame, if one did.
    pub began: Option<usize>,
    /// The signals it handed the scene, in order.
    pub signals: Vec<Signal>,
}

/// `ScriptSteps`: one frame of a script. While its hold is over 1 it counts down; once its hold
/// runs out it walks on from the step after its current one (its first, from its start),
/// following backs, repeats and signals in the same frame, until a step that shows something,
/// which begins and holds, or the end. A step past the script's last is its end. A hold of 0,
/// which the spec doesn't give, runs out as a hold of 1 does.
#[must_use]
pub fn run(progress: Progress, script: &[Step]) -> Ran {
    if progress.hold > 1 {
        return Ran {
            progress: Some(Progress {
                hold: progress.hold.wrapping_sub(1),
                ..progress
            }),
            began: None,
            signals: Vec::new(),
        };
    }
    let mut at = after(progress.step);
    let mut repeat_left = progress.repeat_left;
    let mut signals = Vec::new();
    loop {
        match script.get(at) {
            Some(Step::Show { hold, .. }) => {
                let progress = Progress {
                    step: StepAt::Step(at),
                    hold: *hold,
                    repeat_left,
                };
                return Ran {
                    progress: Some(progress),
                    began: Some(at),
                    signals,
                };
            }
            Some(Step::Back(to)) => at = *to,
            Some(Step::Repeat { from, times }) => {
                (at, repeat_left) = repeat(at, *from, repeat_left.unwrap_or(*times));
            }
            Some(Step::Hand(signal)) => {
                signals.push(*signal);
                at = at.wrapping_add(1);
            }
            Some(Step::End) | None => {
                return Ran {
                    progress: None,
                    began: None,
                    signals,
                };
            }
        }
    }
}

/// The step after this one: from its start, the first.
const fn after(step: StepAt) -> usize {
    match step {
        StepAt::Start => 0,
        StepAt::Step(at) => at.wrapping_add(1),
    }
}

/// `ScriptSteps`: a repeat at step `at` with `left` plays still to go. Over 1, one is used and
/// the stretch plays again from `from`; at 1 the count is cleared and the script goes on past it.
const fn repeat(at: usize, from: usize, left: u16) -> (usize, Option<u16>) {
    if left > 1 {
        (from, Some(left.wrapping_sub(1)))
    } else {
        (at.wrapping_add(1), None)
    }
}

/// What a script's current step shows: nothing from its start or on a step that shows nothing.
#[must_use]
pub fn showing(progress: Progress, script: &[Step]) -> &Shows {
    let current = match progress.step {
        StepAt::Start => None,
        StepAt::Step(at) => script.get(at),
    };
    match current {
        Some(Step::Show { shows, .. }) => shows,
        Some(Step::Back(_) | Step::Repeat { .. } | Step::Hand(_) | Step::End) | None => {
            &Shows::Nothing
        }
    }
}

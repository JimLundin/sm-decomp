//! The screens-intro model, built from its behaviour spec alone: the intro's opening and page 1,
//! and its last pages.
//!
//! The screen stays dark while the first tune loads, slowly brightens on the opening's words,
//! holds through the first tune and the second, and slowly darkens (`opening`); page 1 is made
//! ready in the dark and typed while the picture brightens, then waits for the press that starts
//! the battle on Zebes (`page_one`), a part not yet modelled. At the end, page 4's text dissolves
//! into a picture of the baby, which rises as the picture settles; the picture dissolves back to
//! the text and page 5 is typed letter by letter; a press turns to page 6, and after a pause the
//! picture fades to black and the intro ends.
//!
//! The caller holds the intro's state (`types::intro::IntroState`) and runs `step` once a frame
//! with the frame's press and audio's reply, the scene's content and the rules; the `Report`
//! says what to show, what to ask of audio and what happened. The scene is built from four
//! mechanisms: picture scripts (`script`), the dissolves (`dissolve`), the brightness fades
//! (`fade`) and the baby's rise (`baby`).

mod baby;
mod catalogue;
mod content;
mod dissolve;
mod fade;
mod frame;
mod language;
mod music;
mod opening;
mod page_one;
mod pages;
mod report;
mod rules;
mod scene;
mod script;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use content::{Dissolve, DissolveSets, IndexedColour, IntroContent};
pub use frame::{FrameInputs, step};
pub use language::{Language, Unsupported, plays};
pub use music::{MusicRequest, TrackReply, Tune};
pub use report::{ButtonSetting, Equipment, IntroEvent, Recoloured, Report, ShownPicture};
pub use rules::Rules;
pub use script::{Letter, PictureName, Script, Shows, Signal, Step};

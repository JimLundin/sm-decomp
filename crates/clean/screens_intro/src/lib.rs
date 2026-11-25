//! The screens-intro model, built from its behaviour spec alone: the intro's last pages.
//!
//! Page 4's text dissolves into a picture of the baby, which rises as the picture settles; the
//! picture dissolves back to the text and page 5 is typed letter by letter; a press turns to
//! page 6, and after a pause the picture fades to black and the intro ends.
//!
//! The caller holds the intro's state (`types::intro::IntroState`) and runs `step` once a frame
//! with the frame's press, the scene's content and the rules; the `Report` says what to show and
//! what happened. The scene is built from four mechanisms: picture scripts (`script`), the
//! dissolves (`dissolve`), the brightness fade (`fade`) and the baby's rise (`baby`).

mod baby;
mod catalogue;
mod content;
mod dissolve;
mod fade;
mod frame;
mod language;
mod pages;
mod report;
mod rules;
mod script;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use content::{Dissolve, DissolveSets, IndexedColour, IntroContent};
pub use frame::step;
pub use language::{Language, Unsupported, plays};
pub use report::{IntroEvent, Recoloured, Report, ShownPicture};
pub use rules::Rules;
pub use script::{Letter, PictureName, Script, Shows, Signal, Step};

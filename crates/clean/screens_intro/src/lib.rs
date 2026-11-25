//! The screens-intro model, built from its behaviour spec alone: the intro's opening, page 1, the
//! flashback to Zebes and page 2, and its last pages.
//!
//! The screen stays dark while the first tune loads, slowly brightens on the opening's words,
//! holds through the first tune and the second, and slowly darkens (`opening`); page 1 is made
//! ready in the dark and typed while the picture brightens, then waits for a press
//! (`page_one`). The press asks for the encounter on Zebes and the text dissolves into the
//! fight; once the encounter signals its return, page 2 types as the fight dissolves back to the
//! text, and waits for the press that starts the baby's discovery, a part not yet specced
//! (`flashback`). At the end, page 4's text dissolves into a picture of the baby, which rises as
//! the picture settles; the picture dissolves back to the text and page 5 is typed letter by
//! letter; a press turns to page 6, and after a pause the picture fades to black and the intro
//! ends.
//!
//! The caller holds the intro's state (`types::intro::IntroState`) and runs each frame with the
//! frame's press, audio's reply and the encounter's signal, the scene's content and the rules;
//! the `Report` says what to show, what to ask of audio and of the encounter, and what happened.
//! A caller that already has the encounter's reply runs the frame in one call, `step`. A caller
//! that plays the flashback's room runs it in two parts with room play between them:
//! `begin_frame` steps the stage and hands back the encounter's start or the hide of Samus and
//! her shots, then `FrameAwaitingReply::finish` takes the encounter's reply and runs the
//! scripts. Both calls share the same rules and give the same frame (`frame`).
//!
//! The parts between page 2's wait and page 4's turn (the baby's discovery, page 3, the baby
//! delivered and page 4) are not modelled yet. Since the caller can build every part of the
//! state, the late pages can be entered with the scene already at their starting point: a
//! state built at page 4's turn to the scene (`ToScene`) plays on through pages 5 and 6 to the
//! intro's end.
//!
//! The scene is built from four mechanisms: picture scripts (`script`), the dissolves
//! (`dissolve`), the brightness fades (`fade`) and the baby's rise (`baby`).

mod baby;
mod catalogue;
mod content;
mod dissolve;
mod fade;
mod flashback;
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
pub use flashback::FightReply;
pub use frame::{FrameAwaitingReply, FrameInputs, begin_frame, step};
pub use language::{Language, Unsupported, plays};
pub use music::{MusicRequest, TrackReply, Tune};
pub use report::{
    ButtonSetting, Equipment, FlashbackRequest, IntroEvent, PictureLayout, Recoloured, Report,
    ShownPicture, TextTiles,
};
pub use rules::Rules;
pub use script::{Letter, PictureName, Script, Shows, Signal, Step};

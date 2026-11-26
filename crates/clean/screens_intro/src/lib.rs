//! The screens-intro model, built from its behaviour spec alone: the intro from its start to the
//! flight to Ceres.
//!
//! The screen stays dark while the first tune loads, slowly brightens on the opening's words,
//! holds through the first tune and the second, and slowly darkens (`opening`); page 1 is made
//! ready in the dark and typed while the picture brightens, then waits for a press
//! (`page_one`). The press asks for the encounter on Zebes and the text dissolves into the
//! fight; once the encounter signals its return, page 2 types as the fight dissolves back to the
//! text, and waits for a press (`flashback`). The press brings up the room where Samus finds the
//! baby and asks for the played discovery; once the egg cues the return, page 3 types as the
//! discovery dissolves away (`discovery`). The next press brings up the delivery, its picture
//! panning as the baby is carried into view; the baby's signal brings page 4 back as the
//! delivery fades; the press after page 4 dissolves into a picture of the baby being examined,
//! which rises as the picture settles (`delivery`, `baby`). The picture dissolves back to the
//! text and page 5 is typed letter by letter; a press turns to page 6, and after a pause the
//! picture fades to black and the intro ends.
//!
//! The caller holds the intro's state (`types::intro::IntroState`) and runs each frame with the
//! frame's press, audio's reply and the encounters' cues, the scene's content and the rules; the
//! `Report` says what to show, what to ask of audio and of the encounters, and what happened. A
//! caller that already has the encounters' replies runs the frame in one call, `step`. A caller
//! that plays a room runs it in two parts with room play between them: `begin_frame` steps the
//! stage and hands back the encounters' requests, then `FrameAwaitingReply::finish` takes their
//! replies and runs the scripts. A cut between the two parts is held as a `RoomTurnAwaited`,
//! resumed once; its `RoomTurnCheckpoint` (in `types::intro`) is saved with the state and
//! restores it. All of them give the same frame (`frame`).
//!
//! The scene is built from four mechanisms: picture scripts (`script`), the dissolves
//! (`dissolve`, begun by the passages between text and picture, `passage`), the brightness fades
//! (`fade`) and the moving pictures, the delivery's pan and the examination's rise (`baby`).

mod baby;
mod catalogue;
mod content;
mod delivery;
mod discovery;
mod dissolve;
mod fade;
mod flashback;
mod frame;
mod language;
mod music;
mod opening;
mod page_one;
mod pages;
mod passage;
mod report;
mod rules;
mod scene;
mod script;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use content::{Dissolve, DissolveSets, IndexedColour, IntroContent};
pub use discovery::DiscoveryReply;
pub use flashback::FightReply;
pub use frame::{FrameAwaitingReply, FrameInputs, RoomTurnAwaited, begin_frame, step};
pub use language::{Language, Unsupported, plays};
pub use music::{MusicRequest, TrackReply, Tune};
pub use report::{
    ButtonSetting, DiscoveryRequest, Equipment, FlashbackRequest, IntroEvent, PictureLayout,
    Recoloured, Report, ScenePicture, ShownPicture, TextTiles,
};
pub use rules::Rules;
pub use script::{Letter, PictureName, Script, Shows, Signal, Step};

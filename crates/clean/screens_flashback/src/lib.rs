//! The screens-flashback model, built from its behaviour spec alone: Mother Brain in the Zebes
//! flashback.
//!
//! The encounter starts as page 1 gives way to the room. Mother Brain breathes while missiles
//! cross the room and rings form and drift across it; one ring can push Samus. Enough
//! missile hits defeat her: bursts pulse around her and the room background shakes until page 2
//! is cued, and she and the bursts vanish as the picture dissolves back to the text.
//!
//! The scene calls [`admit_start`] during its setup and [`beat`] once per scene work frame, in
//! its encounter phase. Each beat reports its [`Effects`] to their owners; video reads the
//! background Y from the State and the pictures from [`pictures`].

mod brain;
mod bursts;
mod catalogue;
mod clip;
mod contacts;
mod encounter;
mod pictures;
mod ranks;
mod rings;
mod rules;

pub use brain::{HurtColours, IntroRequest};
pub use catalogue::{Behaviour, BehaviourId, behaviour, catalogue};
pub use contacts::{ReturnDissolve, RoomPlay, ShotCandidate, ShotId, ShotKind};
pub use encounter::{Effects, admit_start, beat};
pub use pictures::{Art, ClipImage, Drawing, ImagePalette, pictures};
pub use rings::RingKnockback;
pub use rules::{ContactLine, ContactRules, HitThreshold, Rules, ShakeHold, ShakeTravel};

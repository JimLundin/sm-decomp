//! The screens-discovery model, built from its behaviour spec alone.
//!
//! In the intro's second played scene Samus approaches an egg. It rocks, cracks and scatters
//! shell fragments; the baby rises through the shell, sheds droplets that splash on the floor,
//! hovers above her, then follows her, crying twice. The spent shell cues page three, and the
//! intro's finish removes the encounter.
//!
//! [`Encounter::new`] takes the caller's [`DiscoveryScene`](scene::DiscoveryScene); every
//! behaviour of the encounter is `Fixed`, so its numbers stay inside the model.
//! [`Encounter::step`] carries a [`DiscoveryState`](types::discovery::DiscoveryState) through
//! one frame and reports the ordered sound offers and the page-three cue;
//! [`Encounter::drawings`] gives the pictures to draw, front first.

mod baby;
pub mod catalogue;
mod drawing;
mod egg;
mod encounter;
mod pulse;
mod rules;
pub mod scene;
mod shell;
mod slime;

pub use drawing::{Drawing, Figure};
pub use encounter::{Cut, Encounter, IntroRequest, ReadyForPageThree, Report};

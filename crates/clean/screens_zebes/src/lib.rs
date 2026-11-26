//! The screens-zebes model, built from its behaviour spec alone: the flight to Zebes (F4),
//! from the station explosion's blackout to the cut that asks to load the Landing Site.
//!
//! Each finished frame, [`step`] runs the rule of the stage the frame begins in, in the order
//! the player sees them: the mosaic reveal (`GunshipAppears`), the planet and stars
//! (`PlanetAppears`), the caption (`PlanetIsNamed`), the ship's flight
//! (`ShipApproachesPlanet`), the still picture (`PlanetHolds`), the slide
//! (`PlanetAndStarsSlide`) and the hand-off (`LandingSiteLoadBegins`). It publishes the picture
//! and sends each request to its owner: pages to E2, colour effects to R4, the plane to E4,
//! the sprites to E1, and the hand-off to Samus's health owner and the scheduler (T1).
//!
//! The State and the imported scene are `types::zebes`'s; the flight is adjusted through
//! [`Rules`].

mod approach;
mod caption;
pub mod catalogue;
mod flight;
mod handoff;
mod hold;
pub mod plane;
pub mod report;
mod reveal;
pub mod rules;
mod slide;

pub use catalogue::{Behaviour, BehaviourId, behaviour};
pub use flight::{FrameAdvance, step};
pub use plane::{PlaneMatrix, PlaneTransform};
pub use report::{
    BeginLandingSiteLoad, ColourEffectRequest, FrameReport, Handoff, PublishedPicture, Requests,
    RestoreFullEnergy, SpriteDraw, StartingPalette,
};
pub use rules::Rules;

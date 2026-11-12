//! The crate root uses its child, and the child names the root's type.
mod collide;
use collide::Mover;

/// Her.
pub struct Samus { mover: Mover }

//! Her as the station's hitters read her: his tail and the escape's steam alike.

use types::{Pair, Px, Radius};

/// What a hitter reads of her on a run: her position, her body and her protection, each as the
/// frame starts, before she moves this frame (samus-movement, samus-health).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Her {
    /// Her position, in whole pixels, where the last frame left her (samus-movement).
    pub position: Pair<Px>,
    /// The half-width and half-height of the body she started the frame in, from her
    /// movement's content.
    pub body: Radius,
    /// Her protection as the frame starts, in frames (samus-health).
    pub protection: u16,
}

impl Her {
    /// `TailHits`, `PuffTouches`: whether her protection had run out as the frame started; only
    /// then can a hitter hit her.
    pub(crate) const fn unprotected(&self) -> bool {
        self.protection == 0
    }
}

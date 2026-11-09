//! The items the HUD can select (§1.1).

/// An item the HUD can select: one of its five icons (§1.1). Not the pickups other domains name
/// items; only these five have an icon to highlight.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectableItem {
    /// Missiles.
    Missiles,
    /// Super missiles.
    SuperMissiles,
    /// Power bombs.
    PowerBombs,
    /// The grapple beam.
    GrappleBeam,
    /// The x-ray scope.
    XRayScope,
}

impl SelectableItem {
    /// Spec: CyclingItems. The HUD's order, left to right, the order Select cycles in (§1.1);
    /// "none" comes before the first.
    pub const ORDER: [Self; 5] = [
        Self::Missiles,
        Self::SuperMissiles,
        Self::PowerBombs,
        Self::GrappleBeam,
        Self::XRayScope,
    ];
}

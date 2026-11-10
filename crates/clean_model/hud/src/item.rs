//! The items the HUD can select (§1.1).

use enum_map::Enum;

/// A kind of ammunition the HUD counts, in the HUD's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum Ammo {
    /// Missiles.
    Missiles,
    /// Super missiles.
    SuperMissiles,
    /// Power bombs.
    PowerBombs,
}

/// An item the HUD can select: one of its five icons (§1.1). Not the pickups other domains name
/// items; only these five have an icon to highlight.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum SelectableItem {
    /// An ammunition, selected while she has some.
    Ammo(Ammo),
    /// The grapple beam.
    GrappleBeam,
    /// The x-ray scope.
    XRayScope,
}

impl SelectableItem {
    /// Spec: CyclingItems. The HUD's order, left to right, the order Select cycles in (§1.1);
    /// "none" comes before the first.
    pub const ORDER: [Self; 5] = [
        Self::Ammo(Ammo::Missiles),
        Self::Ammo(Ammo::SuperMissiles),
        Self::Ammo(Ammo::PowerBombs),
        Self::GrappleBeam,
        Self::XRayScope,
    ];
}

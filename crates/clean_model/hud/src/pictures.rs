//! The HUD's pictures (§1.4): content the importer supplies by name, which the caller gives each
//! step. The spec gives no picture numbers; it says only which picture each thing is drawn with.
//! Each is a picture with its flips, the shared `Look`.

use clean_types::Look;

use crate::item::{Ammo, SelectableItem};
use crate::tile::{BLANK_COLOURS, Tile};

/// Each item's icon: two rows, the top first, of three pictures for the missiles and two for
/// every other item, each with its flips (§1.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Icons {
    /// The missiles' icon.
    pub missiles: [[Look; 3]; 2],
    /// The super missiles' icon.
    pub super_missiles: [[Look; 2]; 2],
    /// The power bombs' icon.
    pub power_bombs: [[Look; 2]; 2],
    /// The grapple beam's icon.
    pub grapple_beam: [[Look; 2]; 2],
    /// The x-ray scope's icon.
    pub x_ray_scope: [[Look; 2]; 2],
}

impl Icons {
    /// The rows of `item`'s icon, the top first, each left to right.
    #[must_use]
    pub fn of(&self, item: SelectableItem) -> [&[Look]; 2] {
        match item {
            SelectableItem::Ammo(Ammo::Missiles) => rows(&self.missiles),
            SelectableItem::Ammo(Ammo::SuperMissiles) => rows(&self.super_missiles),
            SelectableItem::Ammo(Ammo::PowerBombs) => rows(&self.power_bombs),
            SelectableItem::GrappleBeam => rows(&self.grapple_beam),
            SelectableItem::XRayScope => rows(&self.x_ray_scope),
        }
    }
}

/// Rows of pictures, the top first, each as a row whatever its width.
pub fn rows<const ACROSS: usize, const DOWN: usize>(
    looks: &[[Look; ACROSS]; DOWN],
) -> [&[Look]; DOWN] {
    looks.each_ref().map(<[Look; ACROSS]>::as_slice)
}

/// The HUD's pictures, by the names §1.4 gives them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HudPictures {
    /// The blank.
    pub blank: Look,
    /// The digits 0 to 9: digit `d`'s picture is the `d`-th, from 0.
    pub digits: [Look; 10],
    /// A full energy tank.
    pub full_tank: Look,
    /// An empty energy tank.
    pub empty_tank: Look,
    /// The reserve sign (it reads AUTO): three rows, the top first, of two pictures each.
    pub reserve_sign: [[Look; 2]; 3],
    /// The items' icons.
    pub icons: Icons,
    /// The map's blank: a mini-map cell that shows nothing of her map.
    pub map_blank: Look,
    /// The diagonal: a map cell drawn as a diagonal (spec: DiagonalPicture).
    pub diagonal: Look,
}

impl HudPictures {
    /// Spec: SelectionHighlight. The blank cell: the blank's picture, unflipped, in the blank's
    /// colours.
    #[must_use]
    pub const fn blank_cell(&self) -> Tile {
        Tile::new(Look::unflipped(self.blank.picture), BLANK_COLOURS)
    }
}

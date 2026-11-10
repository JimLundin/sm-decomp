//! The HUD's pictures (§1.4): content the importer supplies by name, which the caller gives each
//! step. The spec gives no picture numbers; it says only which picture each thing is drawn with.

use crate::item::{Ammo, SelectableItem};
use crate::tile::{BLANK_COLOURS, Picture, Tile};

/// Each item's icon: two rows, the top first, of three pictures for the missiles and two for
/// every other item, each with its flips (§1.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Icons {
    /// The missiles' icon.
    pub missiles: [[Picture; 3]; 2],
    /// The super missiles' icon.
    pub super_missiles: [[Picture; 2]; 2],
    /// The power bombs' icon.
    pub power_bombs: [[Picture; 2]; 2],
    /// The grapple beam's icon.
    pub grapple_beam: [[Picture; 2]; 2],
    /// The x-ray scope's icon.
    pub x_ray_scope: [[Picture; 2]; 2],
}

impl Icons {
    /// The rows of `item`'s icon, the top first, each left to right.
    #[must_use]
    pub fn of(&self, item: SelectableItem) -> [&[Picture]; 2] {
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
    pictures: &[[Picture; ACROSS]; DOWN],
) -> [&[Picture]; DOWN] {
    pictures.each_ref().map(<[Picture; ACROSS]>::as_slice)
}

/// The HUD's pictures, by the names §1.4 gives them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HudPictures {
    /// The blank.
    pub blank: Picture,
    /// The digits 0 to 9: digit `d`'s picture is the `d`-th, from 0.
    pub digits: [Picture; 10],
    /// A full energy tank.
    pub full_tank: Picture,
    /// An empty energy tank.
    pub empty_tank: Picture,
    /// The reserve sign (it reads AUTO): three rows, the top first, of two pictures each.
    pub reserve_sign: [[Picture; 2]; 3],
    /// The items' icons.
    pub icons: Icons,
    /// The map's blank: a mini-map cell that shows nothing of her map.
    pub map_blank: Picture,
    /// The diagonal: a map cell drawn as a diagonal (spec: DiagonalPicture).
    pub diagonal: Picture,
}

impl HudPictures {
    /// Spec: SelectionHighlight. The blank cell: the blank's picture in the blank's colours.
    #[must_use]
    pub const fn blank_cell(&self) -> Tile {
        self.blank.in_colours(BLANK_COLOURS)
    }
}

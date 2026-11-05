//! The `runtime` crate's side of the intent check (D16): the exact sprites read back from
//! OAM, and the intent sprites placed at a chosen view width.

use crate::data::SpriteEntry;
use crate::game::OamEntry;
use crate::game::view::View;

/// A sprite as it appears on screen: X, Y and tile.
pub type Placed = (i32, i32, u16);

/// The sprites the exact OAM shows, in OAM order.
#[must_use]
pub fn from_oam(oam: &[OamEntry]) -> Vec<Placed> {
    oam.iter()
        .filter(|entry| entry.y != OamEntry::PARKED)
        .map(|entry| {
            let x = i32::from(entry.x);
            let signed = if x >= 0x100 { x.saturating_sub(0x200) } else { x };
            (signed, i32::from(entry.y), entry.tile)
        })
        .collect()
}

/// The sprites a view `width` pixels wide shows, centred on the original 256 px.
#[must_use]
pub fn at_width(view: &View, width: i32, with_margin: bool) -> Vec<Placed> {
    let margin = width.saturating_sub(0x100) / 2;
    let left = margin.saturating_neg();
    let right = 0x100_i32.saturating_add(margin);
    let margin_entries = if with_margin { view.margin.as_slice() } else { &[] };
    view.sprites
        .iter()
        .chain(margin_entries)
        .filter(|entry| visible(entry, left, right))
        .map(|entry| (entry.x, entry.y, entry.tile))
        .collect()
}

/// At native width, the encoder's 9-bit range is what limits a sprite.
fn visible(entry: &SpriteEntry, left: i32, right: i32) -> bool {
    let native = left == 0 && right == 0x100;
    if native { (-0x100..0x100).contains(&entry.x) } else { (left..right).contains(&entry.x) }
}

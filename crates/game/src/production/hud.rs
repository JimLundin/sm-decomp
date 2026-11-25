//! The HUD's carried State and the encoding of its cells.

use canonical::model::Pinned;
use interface_hud::{HudState, Place};

/// The actual complete HUD drawing cannot supply an encoded cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilemapFault {
    /// A cell within the complete HUD extent is absent.
    MissingCell,
}

/// Encode every actual HUD cell in native row order, including its flips and colour set.
/// HUD cells always have foreground priority in this output format.
///
/// # Errors
/// An absent cell rejects the complete output instead of filling or omitting it.
pub fn tilemap(state: &HudState) -> Result<Vec<u8>, TilemapFault> {
    let mut bytes = Vec::with_capacity(192);
    for y in 0_i32..3 {
        for x in 0_i32..32 {
            let tile = state
                .grid
                .get(Place { x, y })
                .ok_or(TilemapFault::MissingCell)?;
            let flips = match tile.look.flip {
                interface_hud::Flip::None => 0,
                interface_hud::Flip::Across => 0x4000,
                interface_hud::Flip::Down => 0x8000,
                interface_hud::Flip::Both => 0xc000,
            };
            let value = tile.look.picture.index()
                | (u16::from(tile.colours.number()) << 10)
                | 0x2000
                | flips;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok(bytes)
}

/// The HUD's complete State, carried between whole frames of play.
pub struct HudPin;
impl Pinned for HudPin {
    type State = HudState;
    const NAME: &'static str = "production-frame-hud";
    const SHAPE: u64 = 0xE8F9_C0CF_E5D1_2326;
    // The escape countdown's place is a pair of 1/256ths of a pixel, x from 128 to 220 whole
    // pixels and y from 48 to 128: 128 whole pixels lies in both.
    const TRACE_U16_SAMPLES: &'static [(&'static str, u16)] = &[("Pair", 128 * 256)];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<HudPin>(), Ok(()));
    }
}

//! Publication of the flyby's plane and low-byte art updates.

use crate::state::video::{Mode7, Mode7Select, Scroll};
use data::assets::ridley_flyby::Tiles;

/// The PPU registers carried by the flyby display consumer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Plane {
    /// The physical matrix and pivot retain their most recent active values.
    pub transform: Mode7,
    /// The ten-bit background scroll as published to the PPU.
    pub scroll: Scroll,
}

impl Plane {
    /// Publish the next drawing transform. The room view clears scroll and overflow.
    pub fn publish(&mut self, picture: Option<([i16; 4], [i16; 2], [i16; 2])>) {
        if let Some((matrix, centre, [x, y])) = picture {
            self.transform = Mode7 {
                matrix,
                centre,
                select: Mode7Select(0x80),
            };
            self.scroll = Scroll {
                x: u16::from_le_bytes(x.to_le_bytes()) & 0x03ff,
                y: u16::from_le_bytes(y.to_le_bytes()) & 0x03ff,
            };
        } else {
            self.transform.select = Mode7Select(0);
            self.scroll = Scroll::default();
        }
    }
}

/// Apply a batch of contiguous low-byte uploads, keeping each word's high byte.
///
/// # Errors
/// Returns `None` before writing if any destination falls outside the plane memory.
pub fn publish_tiles(vram: &mut [u16], uploads: &[Tiles]) -> Option<()> {
    let span = |upload: &Tiles| {
        let start = usize::from(upload.start);
        start..start.saturating_add(upload.low.len())
    };
    if uploads
        .iter()
        .any(|upload| vram.get(span(upload)).is_none())
    {
        return None;
    }
    for upload in uploads {
        for (target, &value) in vram.get_mut(span(upload))?.iter_mut().zip(&upload.low) {
            let [_, upper] = target.to_le_bytes();
            *target = u16::from_le_bytes([value, upper]);
        }
    }
    Some(())
}

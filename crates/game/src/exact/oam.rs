//! The sprite buffer: each frame's sprite draws as the original's OAM entries.
//!
//! The entries follow the order of the draws. Drawing starts from an empty buffer, and once it
//! ends every entry nothing drew is moved off screen.
//!
//! A piece's place is reduced to what an entry holds: X to nine bits (the low table's byte and
//! the high table's ninth bit), Y to eight. So a piece far off screen wraps back into view
//! where the original's would.

use crate::intent::{Placing, ScreenPoint, SpriteDraw, SpritePiece};
use crate::state::video::{NmiBatch, OAM_HIGH_BYTES, OAM_SPRITES, Oam, OamEntry, SpriteCount};
use data::spritemap::{SpriteSize, Spritemap};

/// The Y of a hidden sprite: line 240, below the picture. Only Y moves, so a hidden entry
/// keeps the tile and attributes the buffer last held.
pub const HIDDEN_Y: u8 = 240;

impl Oam {
    /// Clear the first entry's coordinates as a direct OAM word write.
    pub(crate) const fn clear_first_position(&mut self) {
        if let Some(first) = self.low.first_mut() {
            first.x = 0;
            first.y = 0;
        }
    }
}

impl NmiBatch {
    /// Bank `$80` log, clear high OAM `$80:8B1A`: every sprite's X ninth bit and size bit.
    pub const fn clear_high_oam(&mut self) {
        self.oam.high = [0; OAM_HIGH_BYTES];
    }

    /// Bank `$82` log, main game loop `$82:8957`: drawing starts again from the first entry.
    pub const fn restart_sprites(&mut self) {
        self.sprites = SpriteCount(0);
    }

    /// Hide every entry past those drawn this frame and restart the count.
    ///
    /// Bank `$80` log, finalise OAM `$80:896E`. A count at or past the table's end hides
    /// nothing.
    pub fn finalise_oam(&mut self) {
        let drawn = usize::from(self.sprites.0);
        if let Some(unused) = self.oam.low.get_mut(drawn..) {
            for entry in unused {
                entry.y = HIDDEN_Y;
            }
        }
        self.restart_sprites();
    }

    /// Write `draw`'s pieces at the next entries, in order.
    pub fn draw_sprites(&mut self, draw: &SpriteDraw) {
        for piece in &draw.pieces {
            self.draw_piece(piece);
        }
    }

    /// Add cinematic pieces until the OAM table fills, leaving the cursor at the end.
    pub(crate) fn draw_capped_sprites(&mut self, draw: &SpriteDraw) {
        let start = self.sprites.0;
        let count = OAM_SPRITES
            .saturating_sub(usize::from(start))
            .min(draw.pieces.len());
        for piece in draw.pieces.iter().take(count) {
            self.draw_piece(piece);
        }
        self.sprites = SpriteCount(start.saturating_add(u16::try_from(count).unwrap_or_default()));
    }

    fn draw_piece(&mut self, piece: &SpritePiece) {
        match piece.placing {
            Placing::Mapped(size) => self.write_mapped(piece, size),
            Placing::Lone => self.write_lone(piece),
        }
    }

    /// Append a room-effect picture whose origin can be up to 128 pixels beyond the display.
    /// A piece above or below the 256-line span keeps its slot, X and attributes, with Y hidden.
    pub fn draw_room_effect(&mut self, picture: &Spritemap, origin: ScreenPoint) {
        let mut draw = SpriteDraw::default();
        draw.place(picture, origin);
        for piece in &mut draw.pieces {
            if u8::try_from(piece.at.y).is_err() {
                piece.at.y = i32::from(HIDDEN_Y);
            }
        }
        self.draw_sprites(&draw);
    }

    /// A mapped piece: its X ninth bit and size bit go into the high table, and the count
    /// wraps from the last entry to the first.
    ///
    /// Bank `$81` log, add spritemap to OAM `$81:89AE` and `$81:8AB8`: the high table's bits
    /// are set, never cleared (clear high OAM cleared them for the frame), and the OAM index
    /// wraps at `$200` bytes.
    fn write_mapped(&mut self, piece: &SpritePiece, size: SpriteSize) {
        let index = usize::from(self.sprites.0) % OAM_SPRITES;
        let [_, x_high, ..] = piece.at.x.to_le_bytes();
        if let Some(entry) = self.oam.low.get_mut(index) {
            *entry = low_entry(piece);
        }
        let bits = (x_high & 1) | (u8::from(size == SpriteSize::Large) << 1);
        if let Some(byte) = self.oam.high.get_mut(index / 4) {
            *byte |= bits << ((index % 4).wrapping_mul(2));
        }
        let next = index.wrapping_add(1) % OAM_SPRITES;
        self.sprites = SpriteCount(u16::try_from(next).unwrap_or_default());
    }

    /// A lone sprite: the low table's four bytes only, so its X ninth bit is dropped and its
    /// size is the high table's (small, as the frame cleared it), and the count moves on
    /// without wrapping: once the table is full a lone sprite is lost.
    ///
    /// Bank `$90` log, `$90:868D`'s chest sprite and `$90:8AC5`'s atmospheric sprite.
    // open: past the table's end the original writes into the high table (`$0570`); here
    // the sprite is dropped.
    fn write_lone(&mut self, piece: &SpritePiece) {
        if let Some(entry) = self.oam.low.get_mut(usize::from(self.sprites.0)) {
            *entry = low_entry(piece);
        }
        self.sprites = SpriteCount(self.sprites.0.saturating_add(1));
    }
}

/// A piece's low-table entry: the low bytes of its X and Y, and its tile and attributes word.
fn low_entry(piece: &SpritePiece) -> OamEntry {
    let [x, ..] = piece.at.x.to_le_bytes();
    let [y, ..] = piece.at.y.to_le_bytes();
    let [tile, attributes] = piece.look.word().to_le_bytes();
    OamEntry {
        x,
        y,
        tile,
        attributes,
    }
}

#[cfg(test)]
mod tests;

//! The PPU's line-rendering registers ($2100, $2105-$2114, $211A-$2120 and
//! $2123-$2133) as a typed model the native renderer owns.
//!
//! A write decodes exactly as the SNES PPU latches it: the shared scroll latch
//! (BGnHOFS/BGnVOFS), the Mode 7 write latch (M7A-M7Y and M7HOFS/M7VOFS) and
//! the colour data channel selects. VRAM, CGRAM and OAM ports, the counters and
//! the beam-timed mosaic start line are outside this model.
use super::{Affine, Background, ColorMath, Routing, Window, Windows};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Layer {
    pub background: Background,
    pub big_tiles: bool,
    pub mosaic: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Registers {
    pub mode: u8,
    pub forced_blank: bool,
    pub layers: [Layer; 4],
    pub mosaic_size: u8,
    pub windows: Windows,
    pub routing: Routing,
    pub affine: Affine,
    pub color: ColorMath,
    pub direct_color: bool,
    /// The low byte each BGnHOFS/BGnVOFS write leaves for the next one.
    pub scroll_latch: u8,
    /// BGnHOFS's own copy of the latch, which supplies the low three bits.
    pub horizontal_scroll_latch: u8,
    /// The low byte each Mode 7 matrix or scroll write leaves for the next one.
    pub affine_latch: u8,
}

/// The values a line's rasterization reads, as the shared raster core takes them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineRegisters {
    pub mode: u8,
    pub forced_blank: bool,
    pub backgrounds: [Background; 3],
    pub windows: Windows,
    pub routing: Routing,
    pub affine: Affine,
    pub color: ColorMath,
}

fn bit(value: u8, mask: u8) -> bool {
    value & mask != 0
}

impl Registers {
    /// Apply one CPU or DMA write to $21xx, `address` being its low byte.
    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x00 => {
                self.color.brightness = value & 15;
                self.forced_blank = bit(value, 0x80);
            }
            0x05 => {
                self.mode = value & 7;
                for (index, layer) in self.layers.iter_mut().enumerate() {
                    layer.big_tiles = bit(value, 0x10 << index);
                }
            }
            0x06 => {
                for (index, layer) in self.layers.iter_mut().enumerate() {
                    layer.mosaic = bit(value, 1 << index);
                }
                self.mosaic_size = (value >> 4) + 1;
            }
            0x07..=0x0a => {
                let background = &mut self.layers[usize::from(address - 7)].background;
                background.wider = bit(value, 1);
                background.higher = bit(value, 2);
                background.map_words = u16::from(value & 0xfc) << 8;
            }
            0x0b | 0x0c => {
                let first = usize::from(address - 0x0b) * 2;
                self.layers[first].background.tile_words = u16::from(value & 15) << 12;
                self.layers[first + 1].background.tile_words = u16::from(value & 0xf0) << 8;
            }
            0x0d..=0x14 => {
                let layer = usize::from(address - 0x0d) / 2;
                let horizontal = address & 1 == 1;
                if layer == 0 {
                    // BG1's scroll registers are also M7HOFS and M7VOFS.
                    let slot = if horizontal { 6 } else { 7 };
                    self.affine.matrix[slot] = self.affine_word(value) & 0x1fff;
                }
                let high = u16::from(value) << 8;
                let background = &mut self.layers[layer].background;
                if horizontal {
                    let low = (self.scroll_latch & 0xf8) | (self.horizontal_scroll_latch & 7);
                    background.horizontal_scroll = (high | u16::from(low)) & 0x3ff;
                    self.horizontal_scroll_latch = value;
                } else {
                    background.vertical_scroll = (high | u16::from(self.scroll_latch)) & 0x3ff;
                }
                self.scroll_latch = value;
            }
            0x1a => {
                self.affine.large_field = bit(value, 0x80);
                self.affine.character_fill = bit(value, 0x40);
                self.affine.vertical_flip = bit(value, 2);
                self.affine.horizontal_flip = bit(value, 1);
            }
            0x1b..=0x1e => {
                self.affine.matrix[usize::from(address - 0x1b)] = self.affine_word(value);
            }
            0x1f | 0x20 => {
                self.affine.matrix[usize::from(address - 0x1b)] = self.affine_word(value) & 0x1fff;
            }
            0x23..=0x25 => {
                let first = usize::from(address - 0x23) * 2;
                for (offset, nibble) in [value & 15, value >> 4].into_iter().enumerate() {
                    let window = &mut self.windows.layers[first + offset];
                    window.first_inverted = bit(nibble, 1);
                    window.first_enabled = bit(nibble, 2);
                    window.second_inverted = bit(nibble, 4);
                    window.second_enabled = bit(nibble, 8);
                }
                let shift = u32::from(address - 0x23) * 8;
                self.windows.native_flags =
                    (self.windows.native_flags & !(0xff << shift)) | (u32::from(value) << shift);
            }
            0x26 => self.windows.first[0] = value,
            0x27 => self.windows.first[1] = value,
            0x28 => self.windows.second[0] = value,
            0x29 => self.windows.second[1] = value,
            0x2a | 0x2b => {
                let first = usize::from(address - 0x2a) * 4;
                let count = if address == 0x2a { 4 } else { 2 };
                for (index, window) in self.windows.layers[first..first + count]
                    .iter_mut()
                    .enumerate()
                {
                    window.logic = (value >> (index * 2)) & 3;
                }
            }
            0x2c | 0x2d => self.routing.enabled[usize::from(address - 0x2c)] = value,
            0x2e | 0x2f => self.routing.windowed[usize::from(address - 0x2e)] = value,
            0x30 => {
                self.direct_color = bit(value, 1);
                self.color.add_subscreen = bit(value, 2);
                self.color.prevent = (value & 0x30) >> 4;
                self.color.clip = (value & 0xc0) >> 6;
            }
            0x31 => {
                self.color.subtract = bit(value, 0x80);
                self.color.half = bit(value, 0x40);
                for (index, enabled) in self.color.enabled.iter_mut().enumerate() {
                    *enabled = bit(value, 1 << index);
                }
            }
            0x32 => {
                // Each set channel bit selects a fixed colour channel: B, G, R.
                for (channel, select) in [(2, 0x80), (1, 0x40), (0, 0x20)] {
                    if bit(value, select) {
                        self.color.fixed[channel] = value & 0x1f;
                    }
                }
            }
            _ => {}
        }
    }

    fn affine_word(&mut self, value: u8) -> i16 {
        let word = i16::from_le_bytes([self.affine_latch, value]);
        self.affine_latch = value;
        word
    }

    pub fn line(&self) -> LineRegisters {
        LineRegisters {
            mode: self.mode,
            forced_blank: self.forced_blank,
            backgrounds: core::array::from_fn(|index| self.layers[index].background),
            windows: self.windows,
            routing: self.routing,
            affine: self.affine,
            color: self.color,
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        let window = Window {
            first_enabled: false,
            first_inverted: false,
            second_enabled: false,
            second_inverted: false,
            logic: 0,
        };
        Self {
            mode: 0,
            forced_blank: true,
            layers: [Layer::default(); 4],
            mosaic_size: 1,
            windows: Windows {
                first: [0; 2],
                second: [0; 2],
                native_flags: 0,
                layers: [window; 6],
            },
            routing: Routing {
                enabled: [0; 2],
                windowed: [0; 2],
            },
            affine: Affine {
                matrix: [0; 8],
                horizontal_flip: false,
                vertical_flip: false,
                large_field: false,
                character_fill: false,
            },
            color: ColorMath {
                clip: 0,
                prevent: 0,
                enabled: [false; 6],
                add_subscreen: false,
                subtract: false,
                half: false,
                fixed: [0; 3],
                brightness: 0,
            },
            direct_color: false,
            scroll_latch: 0,
            horizontal_scroll_latch: 0,
            affine_latch: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(writes: &[(u8, u8)]) -> Registers {
        let mut registers = Registers::default();
        for &(address, value) in writes {
            registers.write(address, value);
        }
        registers
    }

    #[test]
    fn scroll_pairs_share_one_latch() {
        // BG2HOFS low then high: the high write takes the latched low byte.
        let registers = written(&[(0x0f, 0x34), (0x0f, 0x02)]);
        assert_eq!(registers.layers[1].background.horizontal_scroll, 0x234);
        // BG3VOFS's low byte comes from the previous write to any scroll register.
        let registers = written(&[(0x0d, 0x56), (0x12, 0x01)]);
        assert_eq!(registers.layers[2].background.vertical_scroll, 0x156);
        // Horizontal scroll keeps its own low three bits from the last HOFS write.
        let registers = written(&[(0x0d, 0x07), (0x0e, 0xf8), (0x0f, 0x00)]);
        assert_eq!(registers.layers[1].background.horizontal_scroll, 0x0ff);
    }

    #[test]
    fn mode7_registers_share_their_own_latch() {
        let registers = written(&[(0x1b, 0x00), (0x1b, 0x01), (0x0d, 0xff), (0x0d, 0x1f)]);
        assert_eq!(registers.affine.matrix[0], 0x100);
        assert_eq!(registers.affine.matrix[6], 0x1fff);
        // M7X/M7Y keep 13 bits.
        let registers = written(&[(0x1f, 0xff), (0x1f, 0xff)]);
        assert_eq!(registers.affine.matrix[4], 0x1fff);
    }

    #[test]
    fn colour_data_sets_only_selected_channels() {
        let registers = written(&[(0x32, 0xe5), (0x32, 0x43)]);
        assert_eq!(registers.color.fixed, [5, 3, 5]);
    }

    #[test]
    fn window_selects_decode_two_layers_each() {
        let registers = written(&[(0x24, 0xa6), (0x2a, 0b1110_0100), (0x2b, 0b0110)]);
        let layers = registers.windows.layers;
        assert!(layers[2].first_enabled && layers[2].second_inverted);
        assert!(layers[3].first_enabled && layers[3].second_enabled);
        assert_eq!(registers.windows.native_flags, 0xa600);
        let logic: Vec<u8> = layers.iter().map(|window| window.logic).collect();
        assert_eq!(logic, [0, 1, 2, 3, 2, 1]);
    }

    #[test]
    fn screen_and_mode_registers() {
        let registers = written(&[(0x00, 0x8f), (0x05, 0x09), (0x07, 0x53), (0x0b, 0x21)]);
        assert!(registers.forced_blank);
        assert_eq!(registers.color.brightness, 15);
        assert_eq!(registers.mode, 1);
        let background = registers.layers[0].background;
        assert_eq!(
            (background.map_words, background.wider, background.higher),
            (0x5000, true, true)
        );
        assert_eq!(background.tile_words, 0x1000);
        assert_eq!(registers.layers[1].background.tile_words, 0x2000);
    }
}

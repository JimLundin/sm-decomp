//! One visible line of the frame from the PPU's register and memory state: the
//! replay's video output (D9). Sprites are evaluated first; then a line with
//! background mosaic or Mode 7 EXTBG is composed per pixel and any other line
//! takes the native line path.
use super::composed::Composer;
use super::registers::Registers;
use super::{
    Bgra, BrightnessTables, IndexedSource, LineConfig, Mosaic, Overflow, RasterFault, SpriteInput,
    SpriteLine, WIDTH, evaluate_sprites, render_line,
};

/// Draw beam line `line` (1-224 or 1-239, the frame's row `line - 1`) into
/// `output`. `overflow` is the frame's sticky sprite overflow so far; the
/// result is the overflow after this line.
pub fn draw_line(
    source: &impl IndexedSource,
    registers: &Registers,
    sprites: &SpriteInput<'_>,
    line: u16,
    overflow: Overflow,
    output: &mut [Bgra; WIDTH],
) -> Result<Overflow, RasterFault> {
    let y = i32::from(line);
    let mut sprite_line = SpriteLine::cleared(overflow);
    if registers.forced_blank {
        *output = [[0; 4]; WIDTH];
        return Ok(sprite_line.overflow);
    }
    let (has_tiles, overflow) =
        evaluate_sprites(source, sprites, y - 1, &mut sprite_line.pixels, overflow)?;
    sprite_line.has_tiles = has_tiles;
    sprite_line.overflow = overflow;
    let brightness = BrightnessTables::for_level(registers.color.brightness);
    if registers.composes_per_pixel() {
        Composer::new(source, registers, sprites.even_frame, y)?.draw(
            &sprite_line,
            brightness.borrowed(),
            y,
            output,
        )?;
    } else {
        let line = registers.line();
        let config = LineConfig {
            mode: line.mode,
            forced_blank: line.forced_blank,
            backgrounds: line.backgrounds,
            windows: line.windows,
            routing: line.routing,
            affine: line.affine,
            // Mosaic lines are composed per pixel, so the native mosaic is off.
            mosaic: Mosaic {
                native_enabled: 0,
                background0_enabled: registers.layers[0].mosaic,
                size: registers.mosaic_size,
                start_line: registers.mosaic_start_line,
                modulo: &[0; 256],
                left: 0,
            },
            color: line.color,
            brightness: brightness.borrowed(),
            has_sprites: has_tiles,
        };
        render_line(source, &config, &sprite_line, y, output)?;
    }
    Ok(overflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::memory::OwnedMemory;

    struct Memory(OwnedMemory);

    impl IndexedSource for Memory {
        fn word(&self, index: usize) -> Result<u16, RasterFault> {
            self.0
                .vram
                .get(index)
                .copied()
                .ok_or(RasterFault::Word(index))
        }
        fn palette(&self, index: u8) -> Result<u16, RasterFault> {
            Ok(self.0.cgram[usize::from(index)])
        }
    }

    fn sprites(memory: &OwnedMemory) -> SpriteInput<'_> {
        SpriteInput {
            words: &memory.oam,
            extension: &memory.high_oam,
            tile_words: [0, 0x1000],
            size: 0,
            rotation: None,
            interlace: false,
            even_frame: false,
        }
    }

    /// A Mode 1 screen whose BG1 tile 1 is colour 1 of palette 0, mapped at
    /// $0400 with tiles at $2000, every sprite off screen.
    fn screen(writes: &[(u8, u8)]) -> (Registers, Memory) {
        let mut memory = OwnedMemory::default();
        memory.vram[0x400..0x800].fill(1);
        for row in 0..8 {
            memory.vram[0x2010 + row] = 0x00ff;
        }
        memory.cgram[0] = 0x7c00;
        memory.cgram[1] = 0x001f;
        memory.oam.fill(0xf000);
        let mut registers = Registers::default();
        for &(address, value) in [(0x00, 0x0f), (0x05, 0x09), (0x07, 0x04), (0x0b, 0x02)]
            .iter()
            .chain([(0x2c, 0x01)].iter())
            .chain(writes)
        {
            registers.write(address, value);
        }
        (registers, Memory(memory))
    }

    fn draw(registers: &Registers, memory: &Memory) -> [Bgra; WIDTH] {
        let mut output = [[1; 4]; WIDTH];
        let overflow = Overflow {
            range: false,
            time: false,
        };
        draw_line(
            memory,
            registers,
            &sprites(&memory.0),
            10,
            overflow,
            &mut output,
        )
        .unwrap();
        output
    }

    #[test]
    fn forced_blank_lines_are_black() {
        let (registers, memory) = screen(&[(0x00, 0x8f)]);
        assert_eq!(draw(&registers, &memory), [[0; 4]; WIDTH]);
    }

    #[test]
    fn mosaic_lines_draw_the_same_opaque_background() {
        let (registers, memory) = screen(&[]);
        let native = draw(&registers, &memory);
        assert_eq!(native[0], [0, 0, 255, 0]);
        // A uniform background is unchanged by a 4-pixel mosaic.
        let (registers, memory) = screen(&[(0x06, 0x31)]);
        assert!(registers.composes_per_pixel());
        assert_eq!(draw(&registers, &memory), native);
    }

    #[test]
    fn mosaic_repeats_each_blocks_first_pixel() {
        let (registers, mut memory) = screen(&[(0x06, 0x31)]);
        // Only column 0 of each tile is opaque.
        for row in 0..8 {
            memory.0.vram[0x2010 + row] = 0x0080;
        }
        let output = draw(&registers, &memory);
        let red = [0, 0, 255, 0];
        let blue = [255, 0, 0, 0];
        assert_eq!(output[..8], [red, red, red, red, blue, blue, blue, blue]);
    }
}

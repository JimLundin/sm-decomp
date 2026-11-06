//! The NMI's video commit points (D16, K4): the only places video memory and the committed
//! registers change. Each copies a part of the batch the main strand built during the frame.

use crate::state::video::{
    Mode7, SCROLL_BITS, Scroll, VRAM_WORDS, Video, VramSource, VramStep, VramTransfer,
};

/// VRAM's word address is 15 bits; the PPU ignores the top bit.
const VRAM_ADDRESS_MASK: usize = VRAM_WORDS - 1;

impl Video {
    /// Bank `$80` log, update OAM and CGRAM `$80:933A`: DMA the OAM buffer to OAM and the
    /// whole palette buffer to CGRAM.
    pub const fn upload_oam_and_cgram(&mut self) {
        self.memory.oam = self.batch.oam;
        self.memory.cgram = self.batch.cgram;
    }

    /// Bank `$80` log, update IO registers `$80:91EE`: the register mirrors to the PPU.
    ///
    /// Each scroll register keeps the ten bits the PPU has of the mirror's word (the record's
    /// BG1HOFS holds 0 where the mirror holds 400h), the three COLDATA writes each set the
    /// components they select, and the mode 7 registers are written only in mode 7.
    // open: `$80:91EE` also tests `REG_BGMODE_FAKE`'s mode for the mode 7 registers.
    pub fn commit_registers(&mut self) {
        let mirrors = self.batch.registers;
        let fixed = self
            .batch
            .fixed_colour
            .iter()
            .fold(self.committed.colour_math.fixed, |colour, &write| {
                colour.with(write)
            });
        let mode7 = if mirrors.mode.is_mode7() {
            mirrors.mode7
        } else {
            Mode7 {
                select: mirrors.mode7.select,
                ..self.committed.mode7
            }
        };
        let mut committed = mirrors;
        committed.scroll = mirrors.scroll.map(|Scroll { x, y }| Scroll {
            x: x & SCROLL_BITS,
            y: y & SCROLL_BITS,
        });
        committed.colour_math.fixed = fixed;
        committed.mode7 = mode7;
        self.committed = committed;
    }

    /// Bank `$80` log, process VRAM write table `$80:8C83`: each queued upload DMAs to VRAM
    /// in queue order, and the queue empties.
    pub fn process_vram_writes(&mut self) {
        for transfer in std::mem::take(&mut self.batch.vram) {
            write_vram(&mut self.memory.vram, &transfer);
        }
    }
}

/// One upload. The address wraps within VRAM as the PPU's does.
fn write_vram(vram: &mut [u16], transfer: &VramTransfer) {
    let words = match &transfer.source {
        VramSource::Words(words) => words,
        // open: ROM graphics wait for B3's content; the upload writes nothing until then.
        VramSource::Content(_) => return,
    };
    let stride = match transfer.step {
        VramStep::Row => 1,
        VramStep::Column => 32,
    };
    let start = usize::from(transfer.to.0);
    for (index, &word) in words.iter().enumerate() {
        let address = start.wrapping_add(index.wrapping_mul(stride)) & VRAM_ADDRESS_MASK;
        if let Some(slot) = vram.get_mut(address) {
            *slot = word;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::video::{
        Brightness, Colour, ColourWrite, FixedColour, Registers, Scroll, VRAM_WORDS, Video,
        VramSource, VramStep, VramTransfer, VramWord,
    };

    fn transfer(to: u16, step: VramStep, source: VramSource) -> VramTransfer {
        VramTransfer {
            to: VramWord(to),
            step,
            source,
        }
    }

    #[test]
    fn the_queue_writes_in_order_and_empties() {
        let mut video = Video::default();
        video.batch.vram = vec![
            transfer(0x10, VramStep::Row, VramSource::Words(vec![1, 2, 3])),
            transfer(0x11, VramStep::Row, VramSource::Words(vec![9])),
        ];
        video.process_vram_writes();
        assert_eq!(video.memory.vram[0x10..0x13], [1, 9, 3]);
        assert_eq!(video.batch.vram, []);
    }

    #[test]
    fn a_column_upload_steps_a_row_of_the_tilemap_and_wraps() {
        let mut video = Video::default();
        let last = u16::try_from(VRAM_WORDS - 32).unwrap();
        video.batch.vram = vec![transfer(
            last,
            VramStep::Column,
            VramSource::Words(vec![7, 8]),
        )];
        video.process_vram_writes();
        assert_eq!(video.memory.vram[VRAM_WORDS - 32], 7);
        assert_eq!(video.memory.vram[0], 8);
    }

    #[test]
    fn content_uploads_write_nothing_yet() {
        let mut video = Video::default();
        video.batch.vram = vec![transfer(0, VramStep::Row, VramSource::Content(16))];
        video.process_vram_writes();
        assert!(video.memory.vram.iter().all(|&word| word == 0));
        assert_eq!(video.batch.vram, []);
    }

    #[test]
    fn the_palette_buffer_and_mirrors_commit() {
        let mut video = Video::default();
        video.batch.cgram[5] = Colour(0x7FFF);
        video.batch.registers = Registers {
            brightness: Brightness(15),
            ..Registers::default()
        };
        video.upload_oam_and_cgram();
        video.commit_registers();
        assert_eq!(video.memory.cgram[5], Colour(0x7FFF));
        assert_eq!(video.committed.brightness, Brightness(15));
    }

    #[test]
    fn the_scroll_keeps_ten_bits_and_coldata_sets_what_it_selects() {
        let mut video = Video::default();
        video.batch.registers.scroll[0] = Scroll {
            x: 0x0400,
            y: 0x1200,
        };
        video.batch.fixed_colour = [ColourWrite(0x26), ColourWrite(0x42), ColourWrite(0x00)];
        video.commit_registers();
        assert_eq!(video.committed.scroll[0], Scroll { x: 0, y: 0x200 });
        assert_eq!(
            video.committed.colour_math.fixed,
            FixedColour {
                red: 6,
                green: 2,
                blue: 0
            }
        );
    }
}
